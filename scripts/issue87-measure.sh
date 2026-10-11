#!/usr/bin/env bash
# TEMPORARY Issue #87 measurement. Removed before the fix PR is proposed.
set -u
out=$1
mkdir -p "$out"
uname -a; sysctl -n hw.ncpu 2>/dev/null || nproc; id -u
TARGETS=(ca4i_whole_registry_counts_actual_held_plan_across_two_pages sc10_four_projects_close_independently sc7_codex_ordinary_failure_installs_no_settlement)
run() {
  local name=$1; shift
  local start=$(date +%s)
  cargo test --locked -p rrx --lib -- "$@" >"$out/$name.log" 2>&1
  local code=$?
  echo "RUN $name exit=$code seconds=$(( $(date +%s) - start )) args=$*" | tee -a "$out/summary.txt"
  grep -E '^test result:' "$out/$name.log" | tee -a "$out/summary.txt"
  for t in "${TARGETS[@]}"; do
    grep -E "^test .*$t \.\.\. " "$out/$name.log" | sed "s/^/  $name: /" | tee -a "$out/summary.txt"
  done
  grep -E '^    [a-z0-9_:]+$' "$out/$name.log" | sed "s/^/  FAILED $name: /" | tee -a "$out/summary.txt"
}
for i in 1 2 3; do run "parallel-default-$i"; done
for i in 1 2 3; do run "parallel-t12-$i" --test-threads=12; done
for i in 1 2 3; do run "serial-targets-$i" --test-threads=1 "${TARGETS[@]}"; done
echo "===== failure sections ====="
for f in "$out"/*.log; do
  if grep -q '^failures:' "$f"; then
    echo "##### $(basename "$f")"
    sed -n '/^failures:$/,$p' "$f" | grep -vE '^test .* \.\.\. ok$' | head -400
  fi
done
echo "===== summary ====="
cat "$out/summary.txt"
exit 0
