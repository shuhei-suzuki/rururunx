# Approval Broker and Review Engine Design

**Status:** Draft
**Scope:** MVP approvals and cross-agent review

## 1. Goal

Reduce routine human interruptions without disabling safety controls.

The broker converts native agent permission requests into a provider-neutral decision flow.

## 2. Decision pipeline

```text
Native Executor
      ↓
Normalize Approval Request
      ↓
Deterministic Policy
      ↓ unresolved
Reviewer Set (1 / 2 / 3+ agents)
      ↓
Decision Aggregation
      ↓ unresolved / policy requires human
Human
```

## 3. Normalized Approval Request

Suggested fields:

- request ID
- task ID
- executor agent/session
- action category
- raw requested command/action
- normalized command/action
- working directory
- repository
- branch
- environment classification
- reversible/destructive hints
- network/external-write hints
- relevant diff/state context
- project policy context
- timestamp
- minimal relevant context-pack/repository references
- context bundle version

## 4. Deterministic policy

Policy should decide obvious cases without an LLM.

Possible decisions:

- ALLOW
- DENY
- REVIEW
- HUMAN

Examples:

- known read/test commands → ALLOW
- force push to protected main → DENY
- dependency installation → REVIEW
- production deployment → HUMAN

Actual policies are project-configurable.

## 5. Reviewer selection

Reviewer Set may contain 1, 2, 3, or more agents.

Selection constraints may include:

- exclude executor
- require different provider
- require capability
- reviewer preference/order
- cost/latency class in future versions

Two reviewers must be fully supported.

Example:

```yaml
approval:
  reviewers: [codex, gemini]
  completion: all
  exclude_executor: true
```

## 6. Reviewer result

Each reviewer returns:

```text
APPROVE | DENY | ESCALATE
confidence
risk
reason
evidence
```

A reviewer is decision-only and must not execute the requested action.

## 7. Aggregation

The broker supports at least:

### all

All selected reviewers must provide a compatible non-escalated decision.

### quorum

N-of-M required.

### any

One sufficient decision, intended only for low-risk configured workflows.

Conflicting APPROVE/DENY outcomes should default to escalation unless policy explicitly defines another safe outcome.

An ESCALATE result should normally propagate to human unless policy allows a configured additional reviewer stage.

## 8. Human escalation

Human view must include:

- requested action
- executor
- deterministic policy result
- reviewer results
- reasons/confidence
- relevant branch/worktree/risk context

The user should not need to reconstruct context from multiple terminals.

## 9. Cross-agent mapping

Mappings are configurable, not fixed:

```text
Claude → Codex
Codex → Claude
Grok → Gemini
Gemini → local reviewer
```

The system may choose reviewer pools instead of one fixed mapping.

## 10. Review Engine relationship

The same reviewer-set abstraction is reused for development reviews, but permission approval and adversarial code review are separate request types with separate prompts/contracts.

A two-reviewer development review and a two-reviewer permission decision use the same scheduling/aggregation primitives but different schemas.

## 11. Approval context minimization

Permission review should not receive an executor's entire session history.

The broker asks Context Efficiency for a minimal Approval Bundle containing only what is needed to judge the operation, such as:

- normalized operation
- worktree/branch/environment
- relevant project policy
- affected paths/resources
- minimal diff/state context
- reversibility/destructive hints
- required evidence for the policy class

A reviewer can request context expansion when necessary.

Multiple independent approval reviewers receive equivalent factual bundles. Their conclusions are not shared until aggregation.

For recurring structurally identical permission requests, deterministic policy should be preferred over repeated LLM review when project policy safely permits it.

## 12. Audit

Every approval path records:

- request
- policy decision
- reviewers selected
- each reviewer result
- aggregation result
- human result when present
- final action
- timestamps

## 13. Failure handling

Reviewer timeout/failure must not silently become approval.

Configured outcomes may be:

- use fallback reviewer
- add another reviewer
- escalate human
- deny

Default for unresolved safety-sensitive action: escalate rather than approve.
