# Plan: <title, same as spec>
Source: [spec.md](spec.md), [intent.md](intent.md). Author: <name>. Status: <draft | approved>.
Approved by: <name, role>. Date: <YYYY-MM-DD>.
Scope: <all of spec | slice N: R<a>, R<b>, … ; deferred: R<x>, … (to which later plan)>

## Decisions made while planning
| Topic | Decision | Rejected alternatives (why) |
|---|---|---|
| <library / approach left open by spec> | <choice> | <alt (reason)> |

## Files that change
| Path | New / changed / deleted | Purpose | Requirements |
|---|---|---|---|
| `<path>` | new | <what it holds> | R<n> |

## Order of work
Each step can be verified on its own. Run its proof before you move on.

1. **<Step name>** (R<n>, R<m>)
   - Do: <concrete changes>
   - Proof: `<command>` → <expected result>; <test names>; <visual check>
2. **<Step name>** (R<n>)
   - Do: <…>
   - Proof: <…>

## Risks
| Risk | Where | Mitigation / early check |
|---|---|---|
| <what could break> | step <n> | <how it is de-risked> |

Riskiest step: <n>, because <…>.

## Proof (whole change)
- Build: `<command>`, which must report <healthy output>
- Tests: `<command>`, all green, nothing skipped; new tests: <list>
- Lint: `<command>`, zero warnings
- Manual or visual: <screenshot or scenario checks, against which mock>

## Requirement coverage
| R | Step(s) |
|---|---|
| R1 | 1, 3 |

## Deviations
<Empty at approval. Add an entry whenever the implementation diverges, in the same commit as
the code that diverges: what changed, why, and which steps it affects.>
