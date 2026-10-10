# Architecture decision records

These records explain consequential M1/M2 decisions and the evidence that led
to them. They are reconstructed on 2026-10-09 from the implementation, preserved
evaluation reports, independent reviews and user instructions. An accepted
decision is a direction, not a claim that implementation or milestone evidence
has passed. [Architecture](../architecture.md) describes current behavior;
[candidate evidence](../../evals/m1-m2/results.md) owns measured results.

| Record | Decision | Status |
| --- | --- | --- |
| [0001](0001-native-session-identity.md) | Bind verified native calls to framework session identity | Accepted; implemented in V13 |
| [0002](0002-milestone-evidence.md) | Preserve full milestone gates and failed evidence | Accepted; exits remain open |
| [0003](0003-task-bound-project-recall.md) | Require a task query before project findings leave Cairn | Accepted; implemented at `f44448eb` |
| [0004](0004-extractive-semantic-selection.md) | Select exact excerpts inside records before working recall | Accepted; mechanical checks passed, Groq qualification failed |
| [0005](0005-configured-inference-and-deployment.md) | Configure inference separately from deployment | Accepted; temporary Groq tested, no production provider selected |
| [0006](0006-local-embedding-experiment.md) | Test local statement retrieval before replacing the selector | Experiment complete; useful retrieval and condition preservation failed |
| [0007](0007-agentmemory-workflow-reassessment.md) | Reassess the Agentmemory workflow with Cairn's requested scopes | Review complete; implementation recommendations proposed |
| [0008](0008-independent-finding-capture.md) | Capture separately supported findings through existing canonical writes | Implemented; structural checks passed, automatic-capture screen failed |
| [0009](0009-bounded-native-finalization-checkpoint.md) | Give native turns one bounded opportunity to capture or decline | Implemented; mechanical checks passed, native journey pending |

## Maintaining the records

Record a consequential decision when it is made: context, decision, alternatives,
consequences and evidence. Link existing results instead of copying raw logs or
secrets. Update implementation and validation status when supported by evidence.
If a decision changes, add a successor and mark the original superseded;
preserve failed approaches and their measured outcomes.
