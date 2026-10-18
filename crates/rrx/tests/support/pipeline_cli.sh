#!/bin/sh
base=${0%/*}
if [ -f "$base/pause" ]; then
  : > "$base/entered"
  while [ ! -f "$base/release" ]; do /bin/sleep 0.01; done
fi
/usr/bin/env > "$base/environment"
printf '%s\n' "$*" >> "$base/calls"
case "$*" in
  --version) /bin/cat "$base/version" ;;
  'exec --help'|--help) /bin/cat "$base/help" ;;
  '--setting-sources  --help')
    /bin/cat "$base/help"
    read -r status < "$base/settings-status"
    exit "$status" ;;
  'login status'|'auth status')
    read -r status < "$base/login"
    if [ -f "$base/mutate" ]; then printf '\n# changed during Q\n' >> "$0"; fi
    exit "$status" ;;
  *)
    /bin/cat > "$base/stdin"
    /bin/cat "$base/exec-output"
    read -r status < "$base/exec-status"
    exit "$status" ;;
esac
