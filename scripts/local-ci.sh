#!/usr/bin/env bash
# Run from a clean macOS repository root: bash scripts/local-ci.sh
# The three rururunx-local-ci-* named volumes are standing project caches.
# Use only reviewed commits; host code runs with the user's privileges.
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

# Preserve path newlines with a sentinel, then remove only the output delimiter.
repo=$(git rev-parse --show-toplevel && printf '.') || exit 1
repo=${repo%$'\n.'}
physical_pwd=$(pwd -P && printf '.') || exit 1
physical_pwd=${physical_pwd%$'\n.'}
for path in "$repo" "$physical_pwd"; do
    case "$path" in
        *','*|*'"'*|*$'\n'*)
            printf '%s\n' 'Repository path must not contain a comma, double quote or newline (Docker mount CSV).' >&2
            exit 1 ;;
    esac
done
if [[ "$physical_pwd" != "$repo" ]]; then
    printf '%s\n' 'Run from the repository toplevel.' >&2
    exit 1
fi
head=$(git rev-parse --verify HEAD) || exit 1
clean=$(git --no-optional-locks status --porcelain --untracked-files=all) || exit 1
if [[ -n "$clean" ]]; then
    printf '%s\n' 'Refusing to verify a dirty worktree (including untracked files).' >&2
    exit 1
fi

mkdir -p "$repo/.rrx/local-ci/$head" || exit 1
log_dir=$(mktemp -d "$repo/.rrx/local-ci/$head/run.XXXXXX") || exit 1
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

if main_object=$(git show-ref --verify --hash refs/remotes/origin/main 2> "$log_dir/main-tip.log") &&
    main_tip=$(git rev-parse --verify --end-of-options "$main_object^{commit}" 2>> "$log_dir/main-tip.log"); then
    printf 'origin/main %s\n' "$main_tip" | tee -a "$log_dir/main-tip.log" "$summary" || failed=1
    run_step main-contained git merge-base --is-ancestor "$main_tip" "$head"
else
    record 'NOT RUN' main-contained 0
    failed=1
fi

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
    for name in linux-archive linux-cache-setup linux-image linux-container linux-setup linux-components linux-fmt linux-clippy linux-test; do
        record 'NOT RUN' "$name" 0
    done
    $allow_partial || failed=1
else
    # Only HEAD content enters the container, never the worktree or ignored files.
    run_step linux-archive git archive --format=tar --output="$log_dir/source.tar" "$head"
    # Fresh volumes otherwise have root-owned mountpoints. This helper has no
    # host binds or source code and changes only the cache root permissions.
    run_step linux-cache-setup docker run --rm \
        --label rururunx-local-ci=1 \
        --cidfile "$log_dir/cache-container.cid" \
        --user 0:0 \
        --tmpfs /root:rw,exec,mode=700 \
        --tmpfs /tmp:rw,exec,mode=1777 \
        --env HOME=/root --env TMPDIR=/tmp \
        --mount type=volume,src=rururunx-local-ci-registry,dst=/cache/registry \
        --mount type=volume,src=rururunx-local-ci-git,dst=/cache/git \
        --mount type=volume,src=rururunx-local-ci-target,dst=/cache/target \
        rust:1.91.1 chmod 1777 /cache/registry /cache/git /cache/target

    # Invoked indirectly by run_step, after Docker has made the image available.
    # shellcheck disable=SC2329
    image_metadata() {
        local identity
        identity=$(docker image inspect --format '{{index .RepoDigests 0}}' rust:1.91.1 2>/dev/null) || identity=''
        if [[ -z "$identity" || "$identity" == '<no value>' ]]; then
            identity=$(docker image inspect --format '{{.Id}}' rust:1.91.1) || return 1
        fi
        [[ -n "$identity" ]] || return 1
        printf 'Linux image %s\n' "$identity"
    }
    run_step linux-image image_metadata
    cat "$log_dir/linux-image.log" | tee -a "$log_dir/metadata.log" "$summary" || failed=1
    # Init forwards signals/reaps children; flock returns the step script's status.
    # The quoted script expands variables inside the container, not on the host.
    # shellcheck disable=SC2016
    run_step linux-container docker run --rm --init \
        --label rururunx-local-ci=1 \
        --cidfile "$log_dir/container.cid" \
        --user "$(id -u):$(id -g)" \
        --mount "type=bind,src=$log_dir/source.tar,dst=/source.tar,readonly" \
        --mount "type=bind,src=$log_dir/linux,dst=/results" \
        --tmpfs /src:rw,exec,mode=1777 \
        --tmpfs /home/local-ci:rw,exec,mode=1777 \
        --tmpfs /tmp:rw,exec,mode=1777 \
        --tmpfs /cargo:rw,exec,mode=1777 \
        --mount type=volume,src=rururunx-local-ci-registry,dst=/cargo/registry \
        --mount type=volume,src=rururunx-local-ci-git,dst=/cargo/git \
        --mount type=volume,src=rururunx-local-ci-target,dst=/target \
        --workdir /src \
        --env HOME=/home/local-ci --env TMPDIR=/tmp --env USER=local-ci --env LOGNAME=local-ci \
        --env CARGO_HOME=/cargo \
        --env CARGO_TARGET_DIR=/target/target --env CARGO_INCREMENTAL=0 \
        rust:1.91.1 flock --exclusive --close /target/.local-ci.lock bash -c '
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
            # The parent flock holds the shared-cache lock for this whole script,
            # closing its descriptor in this shell and all checked processes.
            # --touch refreshes source fingerprints after the preceding run.
            run_step linux-setup tar --extract --touch --file /source.tar --directory /src
            run_step linux-components rustup component add rustfmt clippy
            run_step linux-fmt cargo fmt --all -- --check
            run_step linux-clippy cargo clippy --locked --workspace --all-targets -- -D warnings
            run_step linux-test cargo test --locked --workspace
            exit "$failed"
        '
    # Accept exactly one newline-terminated ASCII marker, with no other bytes.
    read_linux_result() {
        local line bytes result seconds
        IFS= read -r line < "$1" || return 1
        [[ "$line" =~ ^(PASS|FAIL)\ ([0-9]+)$ ]] || return 1
        result=${BASH_REMATCH[1]}
        seconds=${BASH_REMATCH[2]}
        bytes=$(LC_ALL=C wc -c < "$1") || return 1
        [[ "$bytes" -eq $((${#line} + 1)) ]] || return 1
        status=$result
        duration=$seconds
    }
    for name in linux-setup linux-components linux-fmt linux-clippy linux-test; do
        status='NOT RUN'
        duration=0
        if [[ -f "$log_dir/linux/$name.result" ]]; then
            read_linux_result "$log_dir/linux/$name.result" || :
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
