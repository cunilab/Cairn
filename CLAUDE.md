<!-- cairn:managed:begin id=agent-contract schema=1 content=5290af7b6943 -->
## Cairn — persistent project memory

Cairn is durable, project-scoped memory for this repository, shared by every agent working on it.
1. Read the Cairn context you were given before re-deriving the project.
2. Search Cairn memory before repeating an investigation you may already have done.
3. When a user supplies a new durable decision or failed approach absent from source, call `cairn_remember` before finishing. Record other durable findings, never routine tool calls.
4. Use project scope for decisions valid across branches, branch for branch-specific facts, and session only for scratch state.
5. Never invent an evidence observation identifier.
6. Never put secrets, credentials, raw prompts or unbounded output into memory.
7. Session boundaries, checkpoints and handoffs are automatic here. Do not hand-roll them.
8. For deeper workflows, use the Cairn Skill.
9. Give a durable project fact a topic key and a value key specific enough to state the whole claim.
10. Attach evidence instead of asserting importance.
11. If Cairn names a corroborating member and it is the same claim, reinforce it.
12. Record a pattern's outcome, including when it did not apply.
<!-- cairn:managed:end id=agent-contract -->
