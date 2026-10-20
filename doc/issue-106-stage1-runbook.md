# Stage 1 daemon environment

`rrx daemon start` builds the `serve` environment itself using `env_clear()` and
the fixed list in `cli::daemon::environment`, in both execution modes. Configure
needed paths and tool settings using those explicit names. Arbitrary prefixes,
Git environment overlays and credential names are not inherited.

For direct `rrx serve`, the operator supplies an allowlisted environment before
exec. Both modes additionally refuse token-like variable names before opening
state. This auxiliary refusal is not proof that values are secret-free.

The owner's residual acceptance and Stage 1 isolation conditions in design §12
still apply, including no same-user processes with credentials in arguments or
environments. Proxy URLs and other allowed settings must not contain credentials.

When a run reports `test_cleanup_failed`, restore the private cleanup parent's
permissions or resolve the filesystem/Git failure, then use the existing
`recover --then abandon-run` flow with the current cursor. Pending cleanup survives
restart and runs without a later test adoption. Do not delete unconfirmed-death
TMPDIRs manually as part of this recovery.
