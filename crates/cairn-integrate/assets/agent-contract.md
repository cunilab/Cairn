schema = 1
heading = Cairn — persistent project memory
lede = Cairn is durable, project-scoped memory for this repository, shared by agents.
mcp_lede = Cairn is durable, project-scoped memory for this repository, shared by agents.

[rule context]
block = Read the Cairn context you were given before re-deriving the project.
mcp = Call `cairn_context` first and use what it returns before re-deriving the project.

[rule search]
block = Search Cairn memory before repeating an investigation you may already have done.
mcp = Call `cairn_search` before repeating an investigation you may already have done.

[rule record]
block = Before finishing, call cairn_remember for durable user choices/failures even if code agrees. Keep requirements, counts, qualifiers, IDs; add user_report capture_attestation. Skip source summaries/routine calls.
mcp = Before finishing, call cairn_remember for durable user choices/failures even if code agrees. Keep requirements, counts, qualifiers, IDs; add user_report capture_attestation. Skip source summaries/routine calls.

[rule scope]
block = Use project scope for decisions valid across branches, branch for branch-specific facts, and session only for scratch state.
mcp = Use project scope for decisions valid across branches, branch for branch-specific facts, and session only for scratch state.

[rule evidence]
block = Never invent an evidence observation identifier.
mcp = Never invent an evidence observation identifier.

[rule secrets]
block = Never put secrets, credentials, raw prompts or unbounded output into memory.
mcp = Never put secrets, credentials, raw prompts or unbounded output into memory.

[rule lifecycle]
block = Session boundaries, checkpoints and handoffs are automatic here. Do not hand-roll them.
mcp = Open and close sessions with `cairn_session`; this client has no automatic lifecycle.

[rule depth]
block = For deeper workflows, use the Cairn Skill.
mcp = For deeper workflows, call `cairn_handoff` for what the last session left you.

[rule subject]
block = Give a durable project fact a topic key and a value key specific enough to state the whole claim.
mcp = Give a durable project fact a `topic_key` and a `value_key` specific enough to state the whole claim.

[rule evidence_over_importance]
block = Attach evidence instead of asserting importance.
mcp = Attach evidence instead of asserting importance.

[rule corroboration]
block = If Cairn names a corroborating member and it is the same claim, reinforce it.
mcp = If Cairn names a corroborating member and it is the same claim, reinforce it.

[rule outcome]
block = Record a pattern's outcome, including when it did not apply.
mcp = Record a pattern's outcome, including when it did not apply.
