schema = 1
heading = Cairn — persistent project memory
lede = Cairn is shared durable project memory.
mcp_lede = Cairn is shared durable project memory.

[rule context]
block = Read Cairn context before re-deriving the project.
mcp = Call `cairn_context` before re-deriving the project.

[rule search]
block = Search Cairn before repeating an investigation.
mcp = Call `cairn_search` before repeating an investigation.

[rule record]
block = Before finish, call cairn_remember for durable user choices/failures even if code agrees. For project findings keep relevant counts, qualifiers, IDs, constraints, status, and lasting policies (read-only); omit temporary requests/completion reports. Separate intent from implementation; add user_report capture_attestation. Apply recall with task-relevant counts and qualifiers. Skip source summaries/routine calls.
mcp = Before finish, call cairn_remember for durable user choices/failures even if code agrees. For project findings keep relevant counts, qualifiers, IDs, constraints, status, and lasting policies (read-only); omit temporary requests/completion reports. Separate intent from implementation; add user_report capture_attestation. Apply recall with task-relevant counts and qualifiers. Skip source summaries/routine calls.

[rule scope]
block = Project scope spans branches; branch is branch-specific; session is scratch.
mcp = Project scope spans branches; branch is branch-specific; session is scratch.

[rule evidence]
block = Never invent evidence observation IDs.
mcp = Never invent evidence observation IDs.

[rule secrets]
block = Never store secrets, credentials, raw prompts, or unbounded output.
mcp = Never store secrets, credentials, raw prompts, or unbounded output.

[rule lifecycle]
block = Lifecycle is automatic; do not hand-roll sessions, checkpoints, or handoffs.
mcp = Use existing sessions; generic MCP cannot start sessions. Never invent keys.

[rule depth]
block = For deeper workflows, use the Cairn Skill.
mcp = For deeper workflows, call `cairn_handoff` for what the last session left you.

[rule subject]
block = Use topic/value keys specific enough to state the whole durable fact.
mcp = Use `topic_key`/`value_key` specific enough to state the whole durable fact.

[rule evidence_over_importance]
block = Attach evidence; do not assert importance.
mcp = Attach evidence; do not assert importance.

[rule corroboration]
block = If Cairn names corroboration for the same claim, reinforce it.
mcp = If Cairn names corroboration for the same claim, reinforce it.

[rule outcome]
block = Record a pattern's outcome, including when it did not apply.
mcp = Record a pattern's outcome, including when it did not apply.
