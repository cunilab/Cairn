# Choosing the right memory scope

Scope decides where a project memory applies; authorization decides who can read it.
Choose the narrowest scope that is still correct.

| Scope | Use when | Example |
|---|---|---|
| `branch` | It is true of this line of work but not of the project | the API shape this branch is moving towards |
| `project` | It is true of the repository generally | the release process, a naming convention |
| `session` | It is scratch state for the current session only | a reproduction step you are still refining |

## How to choose

Ask what would be wrong if a session on a different branch retrieved it. If the answer is
"nothing, it is still true", the scope is `project`. If the answer is "it would mislead them",
narrow it.

Too narrow is recoverable — someone re-records it. Too wide is not: a project-scoped memory
that was only ever true on one branch quietly misleads every future session.
