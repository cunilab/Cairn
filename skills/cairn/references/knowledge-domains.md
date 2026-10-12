# Knowledge domains: project, personal, team

Cairn holds three kinds of durable knowledge. They are not scopes. A **scope** (`project`,
`branch`, `session`) says where a project memory applies; a **domain** says
whose knowledge it is and how far it travels. The two are independent, and nothing you do here
changes a scope.

| Domain | Whose | Travels to | Authored by |
|---|---|---|---|
| `project` | this repository | everyone with access to the project | you, directly — the default |
| `personal` | your account | every project and machine you sign in from | you, directly |
| `team` | everyone on the server | every account, regardless of project membership | server proposal path; made authoritative only by a human administrator |

## Recording personal knowledge

Use `domain: "personal"` on `cairn_remember` `action: "create"` when what you learned is true
of *you* rather than of this repository — a habit, a preference, a lesson that would apply on
your next project too.

```json
{ "cwd": "/path/to/repository", "action": "create", "domain": "personal", "type": "convention",
  "content": "Read the failing test before the implementation, not after" }
```

If it is true of *this* repository, it is project memory and the domain field does not belong
on it. "The retry backoff here is exponential" is project knowledge. "I always check the
backoff before assuming a timeout" is personal.

Personal records are immutable once written. The one permitted change is forgetting them —
`action: "forget"` with `domain: "personal"` — which clears the content and leaves the record
as a tombstone, so the machine that already synchronized it learns it is gone.

## Team knowledge, and the line you cannot cross

**No MCP action makes team guidance authoritative.** The current five MCP tools do not
author team guidance: `domain: "team"` is refused on `create`, and `promote` is not a
supported action. Existing proposals remain absent from recall until a human administrator
ratifies them through web **Governance**. That screen also supports retirement.
The old `cairn team propose` and `cairn team ratify` CLI commands are not available.

## What these two domains will not accept

Personal and team records are stripped of anything that identifies where they came from. A
write is refused — locally when you make it, and again at the server when it arrives — if it
carries:

- an absolute path, a home-directory reference, a drive-letter path or a `file://` URL
- a URL with credentials in it, or an environment-variable assignment
- a long run that looks like an encoded secret
- a token that names the project you are working in
- a shell command invocation

The refusal names which of those it tripped and never quotes your content back. Rewrite the
claim so it stands on its own: "clear the build cache when a stale artifact is suspected" says
the same useful thing as a sentence naming a path under your home directory, and it is true on
every machine rather than one.

## Reading them back

`cairn_search` returns three sibling arrays — `results` for project memory, `personal` and
`team` — never merged, and `total` counts project results alone. Narrow with `domains`.

`cairn_context` includes both domains last, after every project section, capped at a small
share of the budget, and excludes them entirely at `depth: "minimum"`. You do not need to
manage that: it is arranged so that personal and team knowledge can never take space this
project's own context would have used.
