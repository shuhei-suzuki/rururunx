#!/usr/bin/env bash
# Run from a clean macOS repository root: bash scripts/local-ci.sh
# The two rururunx-local-ci-* named volumes are standing caches, not disposable
# test resources. /tmp and /var/tmp preserve the image's world-writable directory
# permissions when Docker initializes them, allowing the host UID to use them.
set -u
set -o pipefail

host_only=false
allow_partial=false
for arg in "$@"; do
    case "$arg" in
        --host-only) host_only=true ;;
        --allow-partial) allow_partial=true ;;
        *) printf 'Usage: bash scripts/local-ci.sh [--host-only [--allow-partial]]\n' >&2; exit 2 ;;
    esac
done
if $allow_partial && ! $host_only; then
    printf '%s\n' '--allow-partial requires --host-only' >&2
    exit 2
fi

repo=$(git rev-parse --show-toplevel) || exit 1
if [[ $(pwd -P) != "$repo" ]]; then
    printf '%s\n' 'Run from the repository toplevel.' >&2
    exit 1
fi
head=$(git rev-parse --verify HEAD) || exit 1
clean=$(git --no-optional-locks status --porcelain --untracked-files=all) || exit 1
if [[ -n "$clean" ]]; then
    printf '%s\n' 'Refusing to verify a dirty worktree (including untracked files).' >&2
    exit 1
fi

log_dir="$repo/.rrx/local-ci/$head"
summary="$repo/.rrx/local-ci/$head.txt"
mkdir -p "$log_dir/linux" || exit 1
: > "$summary" || exit 1
failed=0
printf 'HEAD %s\n' "$head" | tee -a "$summary" || exit 1

record() {
    printf '%s %s %ss HEAD %s\n' "$1" "$2" "$3" "$head" | tee -a "$summary" || failed=1
}

run_step() {
    local name=$1 start=$SECONDS status
    shift
    printf 'Running %s (log: %s/%s.log)\n' "$name" "$log_dir" "$name"
    if "$@" > "$log_dir/$name.log" 2>&1; then
        status=PASS
    else
        status=FAIL
        failed=1
    fi
    record "$status" "$name" "$((SECONDS - start))"
}

# Invoked indirectly by run_step.
# shellcheck disable=SC2329
metadata() {
    local rc=0
    printf 'HEAD %s\n' "$head"
    date -u '+Date %Y-%m-%dT%H:%M:%SZ' || rc=1
    uname -sm || rc=1
    env -u GITHUB_TOKEN -u GH_TOKEN -u GH_ENTERPRISE_TOKEN -u GITHUB_ENTERPRISE_TOKEN rustc --version || rc=1
    env -u GITHUB_TOKEN -u GH_TOKEN -u GH_ENTERPRISE_TOKEN -u GITHUB_ENTERPRISE_TOKEN cargo --version || rc=1
    docker --version || printf '%s\n' 'Docker version unavailable; Linux requires Docker.'
    return "$rc"
}

run_step metadata metadata
# Record versions and date in both the retained metadata log and the summary.
cat "$log_dir/metadata.log" | tee -a "$summary" || failed=1

# Invoked indirectly by run_step.
# shellcheck disable=SC2329
host_cargo() {
    env -u GITHUB_TOKEN -u GH_TOKEN -u GH_ENTERPRISE_TOKEN -u GITHUB_ENTERPRISE_TOKEN cargo "$@"
}

if [[ $(uname -s) == Darwin ]]; then
    run_step host-fmt host_cargo fmt --all -- --check
    run_step host-clippy host_cargo clippy --locked --workspace --all-targets -- -D warnings
    run_step host-test host_cargo test --locked --workspace
    run_step host-release host_cargo build --locked --release --workspace
else
    for name in host-fmt host-clippy host-test host-release; do
        record 'NOT RUN' "$name" 0
    done
    printf '%s\n' 'The host checks require macOS.' >&2
    failed=1
fi

if $host_only; then
    for name in linux-setup linux-fmt linux-clippy linux-test; do
        record 'NOT RUN' "$name" 0
    done
    $allow_partial || failed=1
else
    # Delete only our previous result markers; missing fresh markers cannot pass.
    for name in linux-setup linux-fmt linux-clippy linux-test; do
        rm -f "$log_dir/linux/$name.result" || exit 1
    done
    rm -f "$log_dir/container.cid" || exit 1
    # The quoted script expands variables inside the container, not on the host.
    # shellcheck disable=SC2016
    run_step linux-container docker run --rm \
        --label rururunx-local-ci=1 \
        --cidfile "$log_dir/container.cid" \
        --user "$(id -u):$(id -g)" \
        --mount "type=bind,src=$repo,dst=/src,readonly" \
        --mount "type=bind,src=$log_dir/linux,dst=/results" \
        --mount type=volume,src=rururunx-local-ci-cargo,dst=/tmp \
        --mount type=volume,src=rururunx-local-ci-target,dst=/var/tmp \
        --workdir /src \
        --env HOME=/tmp --env USER=local-ci --env LOGNAME=local-ci \
        --env CARGO_HOME=/tmp/cargo --env RUSTUP_HOME=/tmp/rustup \
        --env CARGO_TARGET_DIR=/var/tmp/target --env CARGO_INCREMENTAL=0 \
        rust:1.91.1 bash -c '
            set -u
            failed=0
            run_step() {
                local name=$1 start=$SECONDS status
                shift
                if "$@" > "/results/$name.log" 2>&1; then
                    status=PASS
                else
                    status=FAIL
                    failed=1
                fi
                printf "%s %s\n" "$status" "$((SECONDS - start))" > "/results/$name.result" || failed=1
            }
            # The stock minimal toolchain lacks fmt/clippy. Install them as the
            # host UID into the standing cache, never into the root-owned image.
            run_step linux-setup rustup toolchain install 1.91.1 --profile minimal --component rustfmt --component clippy --no-self-update
            run_step linux-fmt cargo fmt --all -- --check
            run_step linux-clippy cargo clippy --locked --workspace --all-targets -- -D warnings
            run_step linux-test cargo test --locked --workspace
            exit "$failed"
        '
    for name in linux-setup linux-fmt linux-clippy linux-test; do
        status='NOT RUN'
        duration=0
        if [[ -f "$log_dir/linux/$name.result" ]]; then
            result='' seconds='' extra=''
            read -r result seconds extra < "$log_dir/linux/$name.result"
            if [[ ( "$result" == PASS || "$result" == FAIL ) && "$seconds" =~ ^[0-9]+$ && -z "$extra" ]]; then
                status=$result
                duration=$seconds
            fi
        fi
        [[ "$status" == PASS ]] || failed=1
        record "$status" "$name" "$duration"
    done
fi

# Invoked indirectly by run_step.
# shellcheck disable=SC2329
source_unchanged() {
    local current state
    current=$(git rev-parse --verify HEAD) || return 1
    state=$(git --no-optional-locks status --porcelain --untracked-files=all) || return 1
    [[ "$current" == "$head" && -z "$state" ]]
}
run_step source-unchanged source_unchanged
printf 'Summary: %s\nStep logs: %s\n' "$summary" "$log_dir"
exit "$failed"
