# Stage 1 daemon environment

Run the rrx CLI itself, including daemon start, status and recovery, from a
credential-free environment. No same-user process, even a short-lived CLI, may
carry credentials in arguments or environments. The owner's residual acceptance
and isolation conditions in design §12 still apply. Self-targeted process-info
rules do not block all reads of other processes; parent libproc task info and
sysctl argument/environment reads remain exposed.

`rrx daemon start` uses `env_clear()` and `cli::daemon::environment(mode)`.
In `cli_pipeline` it inherits only HOME, USER, LOGNAME, PATH, TMPDIR, LANG,
LC_ALL, LC_CTYPE, LC_MESSAGES, TERM, XDG_CONFIG_HOME, XDG_DATA_HOME,
XDG_STATE_HOME and XDG_CACHE_HOME. Proxy and Node settings are excluded.
`managed_native` keeps the current fixed list. Arbitrary prefixes and credential
names are excluded in both modes. Configure credentials with each provider CLI's
login/file-based authentication (for example the configured Codex auth.json),
outside rrxd's arguments/environment; do not add API-key names to the allowlist.

Managed native no longer inherits GROK_*, CODEX_* except CODEX_HOME, BUN_*,
OPENSSL_*, TZ, COLORTERM, LC_* except LC_ALL/LC_CTYPE/LC_MESSAGES, or
XDG_RUNTIME_DIR. Move needed tool settings to explicit configuration. The owner
will confirm this compatibility impact separately; it is not recorded as accepted.

For direct `rrx serve`, supply an allowlisted credential-free environment before
exec. Both modes refuse token-like names before opening state. AWS_ is matched
after ASCII uppercasing the name. Refusal reports names without values, and does
not prove that allowed values are secret-free. Proxy URLs and other allowed
managed-native settings must contain no embedded credentials.

Goal/run status `pending_cleanup` lists each pending run/seq/attempt with its
last failure class and rrx-derived path. `checkout_remove` includes filesystem
or registration mismatch failures; `tmpdir_remove` identifies TMPDIR removal.
`cleanup_sweep` identifies an inventory/read failure and reports the derived
checkout path; it uses the same per-obligation backoff. If recording is unavailable,
the daemon logs only the fixed failure class and continues with other runs.
Cleanup never holds, abandons or reopens a run, and never changes cursor versions.
The daemon retries each obligation after an exponential delay (2 to 256 seconds).
Restore permissions on the reported private path and let the sweep retry; no
recovery command is required. Inspect mismatched Git administrative registrations
before repairing them; only the exact checkout's registration may be removed.
Never run Project-wide worktree prune to resolve one obligation. Do not manually
delete resources without confirmed process death. Immutable flags, ACLs and deep
trees can also prevent deletion; inspect and repair only the reported private
directory, never follow replacement links or apply recursive metadata changes to
files that may be hardlinked outside it. Cleanup failure affects disk usage, not
the recorded test result. Pending terminal cleanup survives owner restart; active
running/held runs require the normal full epoch takeover first.
