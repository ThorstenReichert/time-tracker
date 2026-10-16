# Spec: <title, same as intent>
Source: [intent.md](intent.md). Author: <name>. Status: <draft | ready for owner review | approved>.
Date: <YYYY-MM-DD>. Skills applied: <skill@version, … | none>.

Requirements and design in one document. Requirements are numbered `R<n>`, open questions
`OQ-<n>`, areas of concern `AC-<n>`, so the plan and the reviews can cite them.

## Summary of decisions
| Topic | Decision |
|---|---|
| <decision left open by the intent> | <what was chosen, in one line> |

---

## 1. Requirements

### <Capability group>
- **R1 <Name>.** <Testable behaviour. Defaults, limits, edge cases, error behaviour.>
- **R2 <Name>.** <…>

### Exclusions (out of scope)
From intent: <…>.
Decided in this spec: <…>.

---

## 2. Design

### Architecture
<Components, responsibilities, dependency direction, how they talk to each other. A diagram or
tree if useful.>

### Data model
<Entities, keys, the constraints that enforce requirements (cite R<n>).>

### Interfaces
<APIs, commands, events, external systems and how they are called. Cite R<n>.>

### Security and privacy
<Data classification, authn/z, logging rules, network exposure, the policy skills applied.>

### Testing approach
<What gets tested at which level. Property tests or worked examples for critical logic.
Detailed in Stage 4.>

---

## 3. Constraint traceability
| Constraint / success criterion (intent.md) | Satisfied by |
|---|---|
| <constraint> | <R<n>, design section> |

## 4. Open questions from intent
| # | Intent question | Resolution |
|---|---|---|
| 1 | <question> | **Answered:** <decision> (R<n>) / **Carried forward as OQ-<n>** |

Carried forward:
- **OQ-1 <topic>.** <Question, and when or how it will be resolved.>

## 5. Areas of concern
- **AC-1 <title>.** Owner: <name or role>. <Risk or conflict.> *Mitigation:* <…> / *Accepted*
  because <…>.
