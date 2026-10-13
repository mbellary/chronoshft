# Decision register

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

No entry below is an approved ADR. “Source-established” records research direction only. Recommendations are assessment proposals requiring the project's normal review process, not permission requests for completing this deliverable.

| ID | Status / proposed owner | Decision and preserved baseline | Assessment recommendation / alternatives | Linked records |
| --- | --- | --- | --- | --- |
| DEC-001 | Unresolved / Product | Financial TSDB vs edge database vs full trading engine | Define financial data/analytics boundary; keep execution capabilities separately scoped | RES-006–009,021,024; GAP-001; CON-001 |
| DEC-002 | Unresolved / Product | MVP feeds/features and staged delivery | Select first venue/feed and precise capability set; reconcile Stage 3 features with MVP | RES-010–013,025,043; CON-009 |
| DEC-003 | Proposed / Systems | Ring, double buffer and snapshot ownership | Compare restricted SPSC transfer, immutable batch registry and explicit buffer pool; retain unsafe proposals only with proof | RES-017,051,054,058,060–061; GAP-002/003; CON-004 |
| DEC-004 | Unresolved / Systems | Publication, flush and freshness policies | Separate visible-at, persisted-at and recyclable-at; choose thresholds and low-volume timer owner | RES-026,057,059; CON-003/007 |
| DEC-005 | Unresolved / Performance | Performance profile and measurement contract | Preserve MVP and EPYC profiles independently; qualify all figures until measured | RES-027,029,036–039,045,047–048; GAP-013; CON-002/008 |
| DEC-006 | Unresolved / Ingestion | Full ring and slow consumers | Evaluate block/backpressure, drop with accounting, disconnect/replay or bounded admission | RES-053; GAP-004; CON-005 |
| DEC-007 | Source-established direction; details unresolved / Domain | Nanosecond representation | Retain ns resolution; decide epoch/timezone/schema/sequence/overflow semantics separately | RES-040–042,051; GAP-006/014; CON-006/011 |
| DEC-008 | Unresolved / Storage | Durable acknowledgment and recovery | Evaluate volatile mode, WAL or source-replay durability; approve failure/commit model | RES-018,030; GAP-005 |
| DEC-009 | Proposed / Domain+Storage | Micro-sort, corrections and bitemporality | Define lateness bounds and knowledge versions; micro-sort alone cannot handle all corrections | RES-012,030,046,049; GAP-010 |
| DEC-010 | Source-established direction / Architecture | FDAP and single-node Rust MVP | Preserve foundation; pin dependencies and specify extension boundaries | RES-002,024; GAP-011 |
| DEC-011 | Proposed / Performance+Operations | EPYC reference, DPDK and core allocation | Correct population claim; compare ordinary sockets and bypass per adapter; measure locality and interference | RES-034–035; VAL-008/009/025; CON-010 |
| DEC-012 | Unresolved / Product | Open-core/proprietary component boundary | Select license, feature boundary and distribution policy independently of market claims | RES-008,022,031; CON-012 |
| DEC-013 | Proposed / Client Engineering | Flight/Python transport and API | Preserve Flight direction; define copy boundaries, types, nullability and client compatibility | RES-013–014,039,048; VAL-004 |
| DEC-014 | Unresolved / Domain | Financial schemas, SQL and order-book scope | Define precision, query semantics, instruments and venue reconstruction rules | RES-010–012,021,028,051; GAP-008/009 |

Decision closure requires an ADR with: source IDs; exact question; considered alternatives; rationale; product/technical approval record; validation evidence or explicit follow-up; affected documents; superseded records; effective version. Validating an implementation must not retroactively rewrite the original research claim.

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
