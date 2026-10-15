# Traceability

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

## Documentation mapping matrix

The sixteen documents are target destinations, not documents created or approved by this assessment. DOC-01 is represented by this Phase 0 assessment; the immutable PDF remains BASE-001. Every RES ID is mapped at least once; duplication is intentional where concerns cross document boundaries.

| Document | Responsibility | Extraction IDs | Open-item inputs |
| --- | --- | --- | --- |
| DOC-01 Research Baseline & Assessment | Provenance, extraction and assessment | RES-001–063 | All registers |
| DOC-02 Product Requirements | Personas, scope, FR/NFRs and outcomes | RES-009–015,020–025,027–032,036–039,045,047–048 | GAP-001,013,019; CON-001/002/009 |
| DOC-03 Product Architecture | User capabilities and product boundaries | RES-002,009–025,031 | GAP-001,009,021 |
| DOC-04 Platform/System Architecture | Runtime topology and deployment | RES-002–003,015–019,024–025,034–035,060 | GAP-005,007,018; DEC-010/011 |
| DOC-05 Platform Design | Interfaces and lifecycle contracts | RES-016–019,025–026,030,046,049,051,053–061 | GAP-002–007,017 |
| DOC-06 Engineering Requirements | Measurable correctness/safety/performance | RES-012,014–015,020,027–030,036–042,044–048,052,056,061–062 | All Critical/High gaps |
| DOC-07 Engineering Design | Algorithms and component designs | RES-010–012,016–019,021,026,030,040,044,046,049–061 | VAL-012–027; GAP-002–012 |
| DOC-08 Rust Concepts & Patterns | Design-linked concepts and exercises | RES-017,020,024,044,050–062 | GAP-002,011–012,022 |
| DOC-09 Benchmark, Testing & Setup | Reproducible workloads and acceptance | RES-014,027,029,032–039,042,044–048,055–057,059,062 | GAP-013,022; DEC-005 |
| DOC-10 Market Data & Financial Semantics | Feeds, time, books and query semantics | RES-010–012,021,025,028,030,040–042,046,049 | GAP-006,008–010,014,018 |
| DOC-11 Data Model & Storage | Schemas, persistence and recovery | RES-002–004,007,016–019,026,030,034,040,046,049,051–052,058 | GAP-005,007–010,017 |
| DOC-12 Performance & Hardware | CPU, memory, network and profiling | RES-006,017,020,034–039,042,044,050,052,062 | VAL-008/009/022; GAP-012–013 |
| DOC-13 Reliability, Security & Operations | Failure behavior, controls and runbooks | RES-007–008,015,018,020–021,025,030–031,034–035,053 | GAP-004–006,014–018,021 |
| DOC-14 Architecture Decision Records | Approved choices and alternatives | RES-003,006–008,014,017,024–026,030–031,034–035,040,046,051–062 | DEC-001–014; all conflicts |
| DOC-15 Roadmap & Work Breakdown | Dependency-ordered milestones | RES-008,023–025,031,043–049 | CON-009; implementation gate |
| DOC-16 Traceability & Validation | Requirement/design/test/evidence links | RES-001–063 | All registers; EXT-01–03 |

Future Rust chapters should use a consistent structure: implementation problem → required concepts/patterns → isolated runnable example → invariants → correctness tests → benchmark → approved design links. Candidate topics include restricted producer/consumer handles, ownership transfer, immutable Arc snapshots, atomics, cache layout, traits/Send/Sync, bounded pools and safe retirement. Unsafe code remains a proposed technique, not a default requirement.

## Traceability and identifier conventions

```mermaid
flowchart LR
    R[Research extraction] --> V[Validation, gap and decision]
    V --> D[Target document]
    D --> Q[Future requirement and design]
    Q --> T[Test and evidence]
    T --> R
```

| Prefix | Register | Current range |
| --- | --- | --- |
| BASE | Original research baseline | BASE-001 |
| RES | [Extracted source items](research-extraction.md) | RES-001–063 |
| VAL | [Technical validation](technical-validation.md) | VAL-001–028 |
| GAP | [Open gaps](assumptions-and-gaps.md) | GAP-001–022 |
| CON | [Conflicts and ambiguities](assumptions-and-gaps.md#conflicts-and-ambiguities) | CON-001–012 |
| DEC | [Pending decisions](decision-register.md) | DEC-001–014 |
| DOC | Target documentation destinations above | DOC-01–16 |
| EXT | [External evidence](research-baseline.md#evidence-register-and-maintenance) | EXT-01–03 |

IDs are stable and never reused. Use physical PDF references such as `BASE-001 pp. 29–32`. Later requirements, designs, tests and evidence must retain originating RES IDs and reciprocal links. The DOC destinations describe responsibilities; they do not certify that the downstream draft pages are complete. No future requirement, test result or approval record is invented by this mapping.

The [validation framework](validation-framework.md#identity-and-revision-rules) specifies EXP, TST, RUN, EVD and VDEC identities and experiment-scoped hypotheses. The [coverage plan](../10-validation-lab/validation-coverage.md) registers EXP-001–EXP-003 and EXP-005–EXP-030; EXP-004 remains unallocated. Stage 5 registers TST-001–TST-090 with reciprocal experiment, hypothesis and applicable validation references in the [test catalog](../10-validation-lab/test-register.md). RUN, EVD and VDEC registries remain empty. VDEC identifies a validation conclusion and remains distinct from the architecture DEC/ADR records above.

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
