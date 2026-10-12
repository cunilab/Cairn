# M1/M2 regression protocol v9

V9 repeats the 15 Codex cases from the unchanged 30-case corpus on frozen
product source `2f1e043d2516060a3e3d61de5c571a382ee78a1d`, retaining v8's
repository pins, actor/model configuration, prompts, arm order,
300-second budgets, runtime checks, claim denominators, and exit thresholds.
Use a new database and isolated output directory. Preserve the failed v8
quota-limited attempt; rerun the whole Codex subset rather than selecting cases.
The earlier local debug binaries are no longer present. Use the verified macOS
ARM release archive from candidate workflow `37448879539` for every v9 arm and
the disposable v9 server; freeze its executable hashes before any actor runs.
This is a disclosed build change on the same frozen source, not the same v8
debug binaries. M1 uses that same verified native archive.

Frozen native executable SHA-256 values:

| Executable | SHA-256 |
| --- | --- |
| cairn | `9568bf13656d4095ffa15591ee9b0cb05878c42179cd3cf37bb60c369f12dc67` |
| cairnd | `c8b905ed5fdce61c9c7628e945bd9da723c474a068639e429426de22beeb1bce` |
| cairn-server | `284e69b8471b177bfb62b30163d37a7a4496b13b0b2c1acd6d47744c84f06ea0` |

The user explicitly requested skipping Claude. Its 15 original cases remain
marked skipped. A passing Codex subset cannot establish the original two-agent
M1/M2 exit criteria. This is a disclosed regression, not an independent holdout.

## Independent scoring exception

The user authorized an independent calibrated semantic judge while preserving
Laya results. Claude scoring is also skipped. Before v9 actor invocation, an
isolated Codex CLI `0.160.0` judge configured as `gpt-6.1-sol` matched all 30
frozen binary calibration fixtures (24 development and six sealed), with zero
unknown labels and zero observed tool calls. Expected labels were withheld.
Reused fixture screening is not a fresh accuracy estimate.

Freeze that profile: fresh ephemeral process/home per packet, existing auth
only, user configuration/rules ignored, read-only sandbox, shell/plugins/apps/
multi-agent disabled, web search disabled, no project MCP configuration. Use the
unchanged rubric and complete protected packet inventory from v8. The later
v9 protocol supersedes the earlier requirement to use the Claude judge only;
all other predicates remain unchanged. Preserve negative and unknown labels.
Only failed or malformed judge attempts may be retried with the first preserved.

Actor and judge share a model family; process/configuration isolation supplies
independence from the actor session, not model-family independence. Configured
Codex model identity is recorded but not provider-attested. Report this limit.

## M1 candidate journey

Separately run a Codex earlier/later journey against the fresh trusted HTTPS
deployment using the verified macOS ARM archive from candidate workflow
`37448879539`, immutable server/web digests, and a distinct isolated home/project.
Preserve identities, raw traces and hook payloads privately; publish content-free
metadata. Ordinary setup plus the disclosed absolute MCP candidate selection
from v8 remains instrumented evidence; it does not certify portable setup under
every host shell. Claude M1 journeys remain skipped by user instruction.
