schema = 1
heading = Cairn — persistent project memory
lede = Cairn is shared durable project memory.
mcp_lede = Cairn is shared durable project memory.

[rule context]
block = Read delivered continuity; call cairn_context with short task keywords for relevant findings.
mcp = Call cairn_context with short task keywords; an unqueried briefing provides continuity only.

[rule search]
block = Search Cairn with task keywords before repeating an investigation.
mcp = Call cairn_search with task keywords before repeating an investigation.

[rule record]
block = Before finish, use cairn_remember action=capture for durable project findings. Each item keeps its conditions, counts and own support. Separate user intent from inspected implementation; skip routine summaries.
mcp = Before finish, use cairn_remember action=capture for durable project findings. Each item keeps its conditions, counts and own support. Separate user intent from inspected implementation; skip routine summaries.

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
