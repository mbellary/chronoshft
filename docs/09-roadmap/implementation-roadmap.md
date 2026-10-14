# Implementation roadmap

!!! info "Proposed implementation plan · 2026-10-14"
    Grounded in BASE-001 physical pp. 16–17 and 26–28 (RES-024/025/043–049), the [Phase 0 assessment](../00-governance/research-assessment.md) and companion Product, Architecture, Engineering, Benchmarking and Operations drafts. Documentation is populated; implementation, decision approval and release evidence remain pending.

## Delivery intent

Build a single-node Rust financial time-series database on the Flight, DataFusion, Arrow and Parquet foundation. Deliver recent-data capture and SQL first, historical continuity second, then the complete financial research workflow. The [MVP delivery plan](mvp.md) preserves the full proposed capability set; the [milestone registry](milestone-register.md) records dependencies and exit evidence.

The source proposes months 1–3 for the hot path, 4–6 for persistence/hybrid queries and 7–9 for Flight/Python and financial extensions (pp. 26–28; RES-043). These are research estimates. No start date, staffing plan or delivery commitment is approved. Plan dates only after scope, contracts, owners and workload acceptance are ratified.

## Dependency-ordered work

| Phase / milestone | Deliverables | Entry dependency | Exit evidence |
| --- | --- | --- | --- |
| Definition — RM-01 | Financial product boundary, feed/version matrix, full MVP scope and deferred-feature dispositions | Existing research and Product drafts | DEC-001/002 approval records and requirement baseline |
| Contract baseline — RM-02 | Pinned platform/dependencies; event/time/financial schemas; ownership, visibility, overload, durability and client contracts | RM-01 | Reviewed contracts and disposition of required DEC-003/004/006–010/013/014 choices |
| Validation foundation — RM-03 | Deterministic feed fixtures, independent query/domain oracles, crash scenarios, workload harness and measurement contract | RM-02 | Reproducible build and requirement-linked validation inventory; adopted DEC-005 profiles |
| Slice A — RM-04 | Selected adapter, bounded ingestion, ns representation, immutable query snapshots and concurrent recent-data SQL | RM-03 | Feed/schema, ownership, overload and query correctness evidence; qualified hot-path results |
| Slice B — RM-05 | Parquet commit/recovery, catalog handoff, unified hot/cold reads, late-event and correction plumbing | RM-04 | Crash/restart and tier-transition evidence; eligible events neither omitted nor double counted |
| Slice C — RM-06 | ASOF, TIME_BUCKET/OHLCV, correction/as-known history and Flight/Python workflows | RM-05 | Financial oracle agreement, knowledge-version cases and supported client round trips |
| Release readiness — RM-07 | Private/offline package, deployment/runbooks, access controls, capacity and recovery evidence | RM-04/05/06 | Required functional, reliability, security and workload gates passed; release review recorded |

This ordering is a proposed planning refinement. Financial semantics and client contracts are defined in RM-02 even though their complete delivery is in Slice C. Late-event and knowledge-version design precedes storage implementation; micro-sorting alone does not establish correction correctness.

## Workstreams and handoffs

| Workstream | Primary reference | Required handoff |
| --- | --- | --- |
| Product and domain | [Scope & MVP](../01-product/scope-and-mvp.md), [PRD](../01-product/product-requirements.md) | Exact accepted feed/capability matrix, semantics and scope dispositions |
| Architecture and systems | [Component contracts](../02-architecture/component-contracts.md), [engineering validation](../03-engineering/validation-and-traceability.md) | Safe lifetime/reuse model; accepted, visible and durable event states; bounded resource policy |
| Storage and recovery | [Data flow](../02-architecture/data-flow.md), [recovery](../07-operations/recovery.md) | Commit/catalog transition and declared failure model with recovery fixtures |
| Client engineering | [Flight and Python](../03-engineering/flight-and-python.md) | Pinned API, type/null/precision behavior, cancellation and access contract |
| Validation and operations | [Acceptance criteria](../06-benchmarking/acceptance-criteria.md), [operations readiness](../07-operations/readiness-and-traceability.md) | Retained evidence, deployment support matrix and release gate results |

Named owners and approvers are unassigned. Roles in the registry are proposed responsibilities, not staffing commitments. Fixture and packaging preparation may proceed alongside implementation after relevant contracts are fixed; milestone acceptance still follows the listed dependencies.

## Performance and planning constraints

Keep source profiles distinct: the eight-core MVP ingestion target of up to 1M ticks/s and hot aggregate P99 < 1 ms; Stage 2 correct hybrid aggregate < 50 ms; Stage 3 remote 50M-row Polars scenario < 1.5 s. The EPYC reference and burst profiles are separate. All are unverified targets requiring DEC-005 workload selection, timing endpoints, concurrency, loss accounting, resource measurements and durability mode. See the [performance ledger](../00-governance/performance-profiles.md).

Resolve publication/flush thresholds through DEC-004 rather than selecting between conflicting research settings implicitly. Resolve ordinary sockets versus bypass and actual hardware through DEC-011 when relevant to an adopted profile. Do not infer freshness from query cadence or hot query execution latency.

## Change and release control

Record scope changes through DEC-001/002, contract changes through the relevant decision record, and performance changes through DEC-005. Update affected requirements, fixtures and registry gates together. A Slice A demonstration does not establish complete MVP acceptance; omitted ASOF, bitemporality or client features require an explicit scope disposition.

Documentation completion, baseline approval and executable release acceptance are separate states. No roadmap gate is passed by this documentation update. Use the [decision register](../00-governance/decision-register.md), [MVP release checklist](mvp.md#release-checklist) and milestone evidence records to review progress.
