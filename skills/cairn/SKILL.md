---
name: cairn
description: Use when resuming work in a Cairn-connected repository, investigating prior decisions or failures, recording durable findings, or diagnosing memory and session problems.
metadata:
  cairn_skill_schema: 1
  cairn_skill_revision: 66ffc104c4ce
---

# Cairn

Cairn is durable memory for this repository, shared by every agent working on it, so what one
session learns the next one already knows.

Most of it is project memory. Two smaller domains are not scoped to the project at all:
**personal** knowledge follows your account across every project and machine, and **team**
guidance is a server-wide default every account sees. Both are deliberately stripped of
anything that identifies where they came from — see
[knowledge-domains](references/knowledge-domains.md).

Use this Skill when you are resuming someone else's work, deciding whether something is worth
recording, choosing a scope, or working out why Cairn is reporting a problem.

At the start of work, read any Cairn context and search before repeating an investigation.
Before finishing work, record a durable decision, failed approach, procedure, or non-obvious
fact that a later session would otherwise have to rediscover. Use project scope when it applies
across branches, branch scope for branch-specific facts, and session only for scratch state.
Keep user-supplied durable decisions, failure reports, and their identifiers even when the
current implementation already agrees. For durable project findings, preserve task-relevant
requirements, counts, qualifiers, identifiers, constraints, and status. Keep lasting policies,
including read-only restrictions, but not temporary task requests or completion reports. Separate
remembered intent from current implementation. State choices as decisions and trials as observations,
not as implemented or validated behavior. For a user decision or incident, record that finding
concisely; cite source locators without appending an implementation summary. Skip routine tool
calls and plain source summaries. Never send secrets, raw prompts, transcripts, diffs, or
unbounded output.
When recording project knowledge for ordinary reuse, include a bounded
`capture_attestation` with basis `user_report` or `inspected_source`; the server derives the
actor. This records accountability and never objective verification. Use specific topic and
value keys and never invent an observation identifier. If nothing durable was learned, do
not create a memory. Ordinary search uses reuse eligibility. Use archival `inspect` only for
deliberate auditing, verification, supersession, or correction, never as a fallback when
reuse search returns nothing.

## When to reach for which reference

| Situation | Reference |
|---|---|
| A session is starting and work already exists | [resuming-work](references/resuming-work.md) |
| You are about to investigate something | [searching-first](references/searching-first.md) |
| You learned something worth keeping | [recording-knowledge](references/recording-knowledge.md) |
| You are recording and must pick a scope | [choosing-scope](references/choosing-scope.md) |
| Session identity or repository binding is unclear | [sessions-and-tasks](references/sessions-and-tasks.md) |
| Cairn reports a problem | [diagnosing-cairn](references/diagnosing-cairn.md) |
| What you learned is about you, or about the whole team, rather than this repository | [knowledge-domains](references/knowledge-domains.md) |

If this repository also supplies a Cairn instruction block, follow it when it is more
specific than this Skill.
