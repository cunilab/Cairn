# Sessions and repository binding

## Sessions

A Cairn session is one agent's working session on one repository. Where the integration
supports it, sessions open and close automatically and you should not manage them by hand.
Where it does not, open and close them explicitly through Cairn's tools.

Several sessions can be active on one repository at once — two agents, or two worktrees.
That is normal and supported.

## When Cairn reports an ambiguous session

Cairn refuses to guess which session a request belongs to. It returns an ambiguous-session
error naming the candidates. Resolve it by passing the session identifier you are working in,
not by picking one arbitrarily.

## Repository binding

Run `cairn setup` in the authorized Git repository to bind it to the server project
whose remote matches. Setup requires existing membership and cannot grant access.
Current memory scopes are `project`, `branch`, and `session`; task binding is not
part of the current MCP interface.
