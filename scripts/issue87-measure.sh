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
  local tmo=()
  if [ -n "${ISSUE87_TIMEOUT:-}" ]; then
    if command -v timeout >/dev/null; then tmo=(timeout "$ISSUE87_TIMEOUT"); elif command -v gtimeout >/dev/null; then tmo=(gtimeout "$ISSUE87_TIMEOUT"); fi
  fi
  ${tmo[@]+"${tmo[@]}"} cargo test --locked -p rrx --lib -- "$@" >"$out/$name.log" 2>&1
  # (an ISSUE87_GIT_GATE_DELAY_MS prefix on the run call is inherited here)
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
  grep -E 'sweep stage now|observe_drivers step|service sweep body|bootstrap initialize timed out|rpc response .* timed out' "$out/$name.log" | head -60 | sed "s/^/  STALL $name: /" | tee -a "$out/summary.txt"
  grep -E 'WATCHDOG|ISSUE87 SAMPLE' "$out/$name.log" | head -1500 | sed "s/^/  WATCH $name: /" | tee -a "$out/summary.txt"
}
if [ "${ISSUE87_PLAN:-full}" = control ]; then
  CA4I=(ca4i_whole_registry_counts_actual_held_plan_across_two_pages)
  for d in 0 500 1000; do
    ISSUE87_GIT_GATE_DELAY_MS=$d run "control-ca4i-serial-delay$d" --test-threads=1 "${CA4I[@]}"
  done
elif [ "${ISSUE87_PLAN:-full}" = control2 ]; then
  SC7=(sc7_codex_ordinary_failure_installs_no_settlement)
  for d in 0 10000; do
    ISSUE87_TERMINAL_DELAY_MS=$d run "control-sc7-serial-delay$d" --test-threads=1 "${SC7[@]}"
  done
elif [ "${ISSUE87_PLAN:-full}" = control3 ]; then
  SC10=(sc10_four_projects_close_independently)
  ISSUE87_SC10_RELEASE_DELAY_MS=0 run "control-sc10-serial-delay0" --test-threads=1 "${SC10[@]}"
  ISSUE87_SC10_RELEASE_DELAY_MS=35000 run "control-sc10-serial-delay35000" --test-threads=1 "${SC10[@]}"
  ISSUE87_SC10_RELEASE_DELAY_MS=35000 ISSUE87_SC10_BOOTSTRAP_PER_TASK=1 run "control-sc10-serial-delay35000-bootstrap-per-task" --test-threads=1 "${SC10[@]}"
  ISSUE87_SC10_RELEASE_DELAY_MS=25000 run "control-sc10-serial-delay25000" --test-threads=1 "${SC10[@]}"
elif [ "${ISSUE87_PLAN:-full}" = verify ]; then
  # Controls and mutants at the D1-D3 head (expected results in the design doc).
  CA4I=(ca4i_whole_registry_counts_actual_held_plan_across_two_pages)
  SC7C=(sc7_codex_ordinary_failure_installs_no_settlement)
  SC7L=(sc7_claude_ordinary_failure_installs_no_settlement)
  SC10=(sc10_four_projects_close_independently)
  run "plain-targets-serial" --test-threads=1 "${CA4I[@]}" "${SC7C[@]}" "${SC7L[@]}" "${SC10[@]}"
  ISSUE87_GIT_GATE_DELAY_MS=1000 run "C87-1-ca4i-gate1000" --test-threads=1 "${CA4I[@]}"
  ISSUE87_TIMEOUT=900 ISSUE87_GIT_GATE_HANG_NTH=10 run "C87-2-ca4i-lease10-never-ends" --test-threads=1 "${CA4I[@]}"
  ISSUE87_TERMINAL_DELAY_MS=10000 run "C87-3-sc7-codex-terminal10000" --test-threads=1 "${SC7C[@]}"
  ISSUE87_TERMINAL_DELAY_MS=10000 run "C87-4-sc7-claude-terminal10000" --test-threads=1 "${SC7L[@]}"
  ISSUE87_SC10_RELEASE_DELAY_MS=35000 run "C87-6-sc10-release35000" --test-threads=1 "${SC10[@]}"
  ISSUE87_MUTANT=m1 ISSUE87_GIT_GATE_DELAY_MS=1000 run "m1-ca4i-fixed120-gate1000" --test-threads=1 "${CA4I[@]}"
  if command -v timeout >/dev/null || command -v gtimeout >/dev/null; then
    ISSUE87_TIMEOUT=600 ISSUE87_MUTANT=m2 ISSUE87_GIT_GATE_HANG_NTH=10 run "m2-ca4i-unbounded-lease10" --test-threads=1 "${CA4I[@]}"
  else
    echo "RUN m2 skipped: no timeout tool" | tee -a "$out/summary.txt"
  fi
  ISSUE87_MUTANT=m3 ISSUE87_TERMINAL_DELAY_MS=10000 run "m3-sc7-codex-from-finished-terminal10000" --test-threads=1 "${SC7C[@]}"
  ISSUE87_MUTANT=m4 run "m4-sc7-settlement-turn-skips" --test-threads=1 "${SC7C[@]}" "${SC7L[@]}"
  ISSUE87_MUTANT=m5 run "m5-sc7-binding-turn-skips-unbound-poll" --test-threads=1 "${SC7C[@]}" "${SC7L[@]}"
  ISSUE87_MUTANT=m6 ISSUE87_SC10_RELEASE_DELAY_MS=35000 run "m6-sc10-shared-bootstrap-release35000" --test-threads=1 "${SC10[@]}"
  ISSUE87_MUTANT=m7 run "m7-sc10-no-continuation" --test-threads=1 "${SC10[@]}"
elif [ "${ISSUE87_PLAN:-full}" = t12x3 ]; then
  for i in 1 2 3; do run "parallel-t12-$i" --test-threads=12; done
elif [ "${ISSUE87_PLAN:-full}" = t24 ]; then
  for i in 1 2; do run "parallel-t24-$i" --test-threads=24; done
elif [ "${ISSUE87_PLAN:-full}" = t12x2 ]; then
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
