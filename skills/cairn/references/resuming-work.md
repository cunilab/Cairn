# Resuming existing work

Cairn writes handoffs when supported lifecycle hooks run. Automatic compaction and
session-end coverage depends on the installed agent's capabilities and observed health.
Generic MCP clients use explicit session and handoff calls.

## Do this first

1. Read the handoff Cairn delivered in your context. It can include captured activity and
   recorded continuity; it may omit work that was never captured or recorded.
2. Check the current branch state and continue from the recorded next step when present.
   Use the handoff to avoid repeating established investigation.
3. Where the handoff names a failure, check whether it still reproduces before assuming it does.
4. Once the current task is known, call `cairn_context` with a short `query` naming
   that task. An unqueried briefing supplies continuity and memory availability,
   not project findings. Do this without asking the human to manage recall. Never
   send the raw task prompt or transcript as the query. Use `cairn_search` for
   narrower follow-up questions when needed; preserve the returned qualifications.

## What a handoff is not

It is not a task list you must finish, and it is not authoritative about intent. It is an
account of what happened. If it conflicts with what you observe now, what you observe wins —
and that conflict is itself worth recording.

## When there is no handoff

A first session may have none, and missing lifecycle delivery can also leave no handoff.
Check integration health when a handoff was expected. Search memory, then proceed.
