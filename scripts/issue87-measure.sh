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
  # Re-executed child test binaries print their own "test result" lines;
  # only the last one belongs to the harness run.
  grep -E '^test result:' "$out/$name.log" | tail -1 | tee -a "$out/summary.txt"
  grep -nE 'error(\[|:)|signal|SIG[A-Z]+|abort|process didn.t exit|^failures:' "$out/$name.log" | head -20 | sed "s/^/  DIAG $name: /" | tee -a "$out/summary.txt"
  tail -8 "$out/$name.log" | sed "s/^/  TAIL $name: /" | tee -a "$out/summary.txt"
  for t in "${TARGETS[@]}"; do
    grep -E "^test .*$t \.\.\. " "$out/$name.log" | sed "s/^/  $name: /" | tee -a "$out/summary.txt"
  done
  grep -E '^    [a-z0-9_:]+$' "$out/$name.log" | sed "s/^/  FAILED $name: /" | tee -a "$out/summary.txt"
}
if [ "${ISSUE87_PLAN:-full}" = t12x2 ]; then
  for i in 1 2 3 4; do run "parallel-t12-$i" --test-threads=12; done
else
  for i in 1 2 3; do run "parallel-default-$i"; done
  for i in 1 2 3; do run "parallel-t12-$i" --test-threads=12; done
  for i in 1 2 3; do run "serial-targets-$i" --test-threads=1 "${TARGETS[@]}"; done
fi
echo "===== failure sections ====="
for f in "$out"/*.log; do
  if grep -q '^failures:' "$f"; then
    echo "##### $(basename "$f")"
    sed -n '/^failures:$/,$p' "$f" | grep -vE '^test .* \.\.\. ok$' | grep -vE 'startup admission waited|drive returned after|^actual retained Driver outcome: Task Driver cancelled' | head -1500
  fi
done
echo "===== summary ====="
cat "$out/summary.txt"
exit 0
