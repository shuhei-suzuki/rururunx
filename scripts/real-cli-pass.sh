#!/bin/bash
# Owner only: run from a clean repository toplevel in a fresh terminal.
set -euo pipefail
umask 077

die() { printf '%s\n' "$*" >&2; exit 1; }

[[ $(uname -s) == Darwin ]] || die 'This harness requires macOS.'
root=$(git rev-parse --show-toplevel 2>/dev/null) || die 'Not a Git worktree.'
[[ $(pwd -P) == "$root" ]] || die 'Run from the repository toplevel.'
[[ -z $(git status --porcelain=v1 --untracked-files=all) ]] || die 'Worktree is dirty.'
head=$(git rev-parse HEAD)

blocked=()
while IFS= read -r name; do
    case "$name" in
        *[Tt][Oo][Kk][Ee][Nn]*|*[Ss][Ee][Cc][Rr][Ee][Tt]*|*[Pp][Aa][Ss][Ss][Ww][Oo][Rr][Dd]*|*[Aa][Pp][Ii]_[Kk][Ee][Yy]*)
            blocked+=("$name") ;;
    esac
done < <(compgen -e)
if (( ${#blocked[@]} )); then
    printf 'Refusing token-like environment variable names:\n' >&2
    printf '%s\n' "${blocked[@]}" >&2
    exit 1
fi

printf 'Other same-user processes (PID and executable name only):\n'
ps -U "$(id -u)" -o pid=,comm= | awk '
    { pid=$1; sub(/^[[:space:]]*[0-9]+[[:space:]]+/, ""); name=$0; sub(/^.*\//,"",name);
      if(name=="claude" || name=="codex" || name=="node" || name=="Codex") print pid, name }
'
printf 'Stop other Claude/Codex sessions and ALL credential-holding same-user processes.\n'
printf 'This pass can read other same-user process arguments and environment.\n'
printf 'Confirm this fresh terminal meets that condition; type yes to run once: '
IFS= read -r answer
[[ "$answer" == yes ]] || die 'Cancelled.'

codex=$(command -v codex) || die 'codex is not on PATH.'
claude=$(command -v claude) || die 'claude is not on PATH.'
[[ "$codex" == /* && -x "$codex" ]] || die 'codex must resolve to an absolute executable.'
[[ "$claude" == /* && -x "$claude" ]] || die 'claude must resolve to an absolute executable.'
out="$root/.rrx/real-cli/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$root/.rrx/real-cli"
mkdir "$out" || die 'Output directory already exists.'
printf '%s\n' "$head" > "$out/target-head.txt"
# Pins are retained privately, never echoed; Q must accept the exact same versions.
"$codex" --version > "$out/codex-version.txt" 2> "$out/codex-version-stderr.log" || die 'codex version probe failed.'
"$claude" --version > "$out/claude-version.txt" 2> "$out/claude-version-stderr.log" || die 'claude version probe failed.'
codex_version=$(cat "$out/codex-version.txt")
claude_version=$(cat "$out/claude-version.txt")

printf 'Building the library test harness (offline).\n'
cargo test --offline -p rrx --lib --no-run > "$out/build.log" 2>&1 || die 'Build failed; inspect the private build.log.'
[[ $(git rev-parse HEAD) == "$head" && -z $(git status --porcelain=v1 --untracked-files=all) ]] || die 'HEAD or worktree changed during build.'
test_name=state::execution::pipeline::tests::real_cli::real_cli_pass
status=0
RRX_REAL_CLI_PASS=1 \
RRX_REAL_CODEX="$codex" \
RRX_REAL_CLAUDE="$claude" \
RRX_REAL_CODEX_VERSION="$codex_version" \
RRX_REAL_CLAUDE_VERSION="$claude_version" \
RRX_REAL_CODEX_MODEL="${RRX_REAL_CODEX_MODEL:-gpt-6.1-sol}" \
RRX_REAL_CODEX_EFFORT="${RRX_REAL_CODEX_EFFORT:-medium}" \
RRX_REAL_CLAUDE_MODEL="${RRX_REAL_CLAUDE_MODEL:-claude-opus-5-5}" \
RRX_REAL_CODEX_HOME="${RRX_REAL_CODEX_HOME:-$HOME/.codex}" \
RRX_REAL_CLI_OUT="$out" \
cargo test --offline -p rrx --lib "$test_name" -- --ignored --exact --test-threads=1 > "$out/run.log" 2>&1 || status=$?
printf 'Summary: %s/summary.json\n' "$out"
if [[ ! -f "$out/summary.json" ]]; then
    die 'No summary was produced; inspect the private run.log. No retry was started.'
fi
exit "$status"
