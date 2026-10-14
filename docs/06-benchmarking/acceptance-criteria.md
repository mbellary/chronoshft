# Acceptance criteria

!!! info "Proposed gates; approval and execution pending · 2026-10-14"
    The [HFT research notes](../assets/research/HFT-Research-Notes.pdf), physical pp. 17, 21–23 and 26–28, provide source targets. [Product requirements](../01-product/product-requirements.md), [Scope & MVP](../01-product/scope-and-mvp.md) and [Architecture contracts](../02-architecture/component-contracts.md) define review dependencies. This page does not approve requirements, close findings or report benchmark passes.

## Gate order and evidence

| Gate | Proposed acceptance condition | Evidence / dependency | Current status |
| --- | --- | --- | --- |
| Contract readiness | Scope, schema, timing endpoints, overload, ownership, durability and financial semantics declared | DEC-001/002/003/004/006/007/008/009/013/014 dispositions; requirement-linked contract | Pending |
| Dependency/build | Complete implementation compiles with pinned compatible libraries | Lockfile, build logs and version manifest; VAL-017 | Untested |
| Safety | Exclusive mutation/publication and delayed-reader-safe reuse established; bounded capacity enforced | Ownership review plus applicable model/unsafe/stress results; VAL-012–014/019–022 | Untested |
| Correctness | Independent query/domain oracles and type/sequence round trips pass | [Test matrix](test-strategy.md); VAL-010/011/015/016/018/023/026/027 | Untested |
| Recovery | Crash/restart and retry match the selected acknowledgment/failure model | Commit-point fault injection and identity reconciliation; DEC-008/009, VAL-024 | Untested |
| Performance | Selected profile meets every adopted metric while correctness holds under declared concurrent load | Raw timing/loss/resource evidence, reproducible manifest; DEC-005/011, GAP-013 | Untested |
| Release disposition | Required scope and evidence reviewed, exceptions explicit and authorized | Named owners, review/date and approved decisions; not implied by site build | Pending |

A blocked or failed required gate prevents claiming overall acceptance. A not-applicable gate needs a scope rationale; it cannot hide an unimplemented MVP capability.

## Performance gate selection

Use the complete [workload ledger](workload-definitions.md) and [measurement contract](benchmark-methodology.md). Select source MVP, EPYC reference, stage or product-added profiles explicitly; never combine their most favorable numbers into one promise.

| Candidate gate | Numerical target retained | Acceptance qualification |
| --- | --- | --- |
| Source recent-data baseline | PERF-MVP-ING: up to 1M ticks/s; PERF-MVP-Q: P99 < 1 ms | 8-core profile; approved duration/schema/window; concurrent load and loss accounting |
| Source reference | 2.5M ticks/s; 8M/s ≤ 500 ms; hot P95 < 180 µs / P99 < 850 µs | EPYC profile, average 48-byte payload; recovery and timing scope declared |
| Stage 1 | 1M ticks/s and query every 100 ms | AVG/COUNT correctness and safety; cadence alone does not establish latency/freshness |
| Stage 2 | Correct cross-tier aggregate < 50 ms | Declared hot/cold data and cache state; no gaps/duplicates during handoff |
| Stage 3 | 50M remote rows → Polars < 1.5 s with ASOF | Complete dataframe endpoint, join oracle, bytes/network/client versions |
| Product ingestion | 1–2M ticks/s; 8M/s ≤ 1,000 ms; zero eligible-event drops | 48–64-byte payload; repeated bursts and bounded drain, not universal loss-free capture |
| Product query tails | P50 < 50 µs / P99 < 150 µs / P99.9 < 850 µs | Bounded lookup/VWAP under declared ingest/maintenance; sufficient samples and errors reported |
| Product local dataframe | 50M rows over loopback < 1.5 s | Separate from remote Stage 3; include materialization/type fidelity |
| Product footprint | 1B 64-byte ticks → < 8.5 GB | Complete storage boundary and lossless equality; 8×–10× remains a separate aspiration |

All rows are candidate gates and remain untested. Approximately 4.2 GB/s reference egress, 20× DX and competitor comparisons require their own matched experiments before publication. Numerical acceptance cannot override a correctness failure.

## Freshness, allocations and durability

Approve receipt-to-visible freshness separately from query latency (DEC-004). Resolve conflicting publication/flush examples rather than selecting one silently. Measure allocations through decode, append, rotation, registry and reclamation; zero steady-state allocation applies only to a declared path/interval with instrumentation evidence (PRD-N03). Fast average performance and lack of GC do not establish deterministic tails.

Define eligible-event loss using explicit rejection/duplicate/gap accounting (PRD-N06). Define durable acknowledgment and recovery independently of acceptance/publication (PRD-N07). No zero-loss-on-crash claim is permitted by the source examples alone.

## Delivery slices and approval

Recent-data Slice A, historical continuity Slice B and Flight/financial workflow Slice C organize evidence collection; they do not reduce the complete proposed MVP without a DEC-002 disposition. ASOF/bitemporality scope must stay explicit. The source months 1–9 are estimates, not acceptance deadlines.

For every gate retain requirement/profile IDs, revision/run ID, result, evidence links, reviewer/owner and disposition. Review ownership is proposed as Product/Domain for scope and semantics, Architecture/Engineering for safety and recovery, and Validation/Performance for measurements. Named approvers remain unassigned.

Related: [Product readiness](../01-product/product-readiness.md), [decision register](../00-governance/decision-register.md), [evidence and traceability](evidence-and-traceability.md). Documentation completion requires populated pages and a strict build; release readiness requires the executable evidence above.
