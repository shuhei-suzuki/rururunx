#!/bin/zsh
# Issue #87: the D1-D4 fix on main, macOS parallel rrx lib x10 (default threads).
# Run from the rururunx checkout:  zsh mac-x10.sh /path/to/d1-d4.patch
set -u
patch=$1
ulimit -n 10240
git fetch origin main && git switch --detach origin/main && git apply --index "$patch" || exit 1
git log -1 --format='base %H' > issue87-mac.txt
# D4 changes the bundled SQLite build flags; rebuild it explicitly.
cargo clean -p libsqlite3-sys || exit 1
cargo test --locked -p rrx --lib --no-run || exit 1
for i in $(seq 1 10); do
  cargo test --locked -p rrx --lib > issue87-mac-$i.log 2>&1
  echo "run $i: $(grep -E '^test result:' issue87-mac-$i.log | tail -1)" | tee -a issue87-mac.txt
  grep -E 'ca4i_whole_registry_counts_actual_held_plan_across_two_pages|sc10_four_projects_close_independently|sc7_(codex|claude)_ordinary_failure_installs_no_settlement' issue87-mac-$i.log | grep -E '\.\.\. ' | tee -a issue87-mac.txt
  grep -E '^    [a-z0-9_:]+$' issue87-mac-$i.log | sed 's/^/  FAILED /' | tee -a issue87-mac.txt
done
git apply -R --index "$patch"
echo "summary: issue87-mac.txt"
