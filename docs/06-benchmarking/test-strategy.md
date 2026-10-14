# Test strategy

!!! info "Proposed correctness and safety validation · 2026-10-14"
    Derived from the [HFT research notes](../assets/research/HFT-Research-Notes.pdf), physical pp. 6–10, 16–17, 23–28 and 29–42, the [Architecture contracts](../02-architecture/component-contracts.md) and existing [technical validation findings](../00-governance/technical-validation.md). This is a test plan, not executed-test evidence or a new upstream API audit.

## Verification sequence

Pin dependencies and compile complete examples first. Establish schema/domain oracles and memory ownership invariants before performance tuning. Then verify integration, recovery and clients, and finally run the [load profiles](workload-definitions.md) while checking correctness. A crash-free stress run or a strict documentation build cannot prove unsafe-code soundness or financial correctness.

## Requirement-linked test matrix

| Area | Proposed cases and oracle | Traceability |
| --- | --- | --- |
| Admission and sequencing | Malformed framing, decode failures, sequence gaps/duplicates, reconnect, full-ring policy and within/beyond-window lateness; reconcile identities and outcomes | PRD-F02/F12, N06; VAL-010/011/025; DEC-002/006/007/014 |
| Writer/publication | Invalid zero/non-power-of-two ring sizes; append at capacity, partial/empty publication, repeated wraparound and timer/append schedules; readers see only initialized consistent columns | PRD-F03, N03/N08; VAL-012/013/019 |
| Lifetime/reclamation | Delay query/persistence owners across many rotations; cancel/drop in varying order; reuse only after all owners and retention permit it | PRD-F03, N08; VAL-014/020/021; DEC-003 |
| Provider planning | Repeated/concurrent non-destructive scans, reordered/sparse projections, schema equality, filter/limit semantics and pushdown declarations against a simple reference dataset | PRD-F03/F04; VAL-015/016/017 |
| Time/numeric fidelity | Nanosecond precision, equal timestamps with sequence, timezone/null/overflow boundaries and exact prices/sizes through Arrow → Parquet → Flight → supported dataframe | PRD-F11/F12, N09; VAL-010/011/018 |
| Financial operators | ASOF keys/direction/ties/tolerance/nulls; bucket origin/boundaries; OHLCV open/close ordering; VWAP precision/zero volume against independent oracles | PRD-F05/F06; VAL-026/027; DEC-014 |
| Hybrid/corrections | Query during flush, catalog publication, eviction and compaction; exactly-once logical coverage; current/as-known correction versions against expected identities | PRD-F07–F10, N08; VAL-023; DEC-009 |
| Persistence/restart | Partial write, failed sync/commit, disk full, retry and crash before/after catalog publication; restart discovers only eligible committed versions | PRD-F07, N07; VAL-024; DEC-008 |
| Flight/client/access | Row/type checks, result size, loopback/remote clients, cancellation, disconnect and partial-stream error; selected authentication/access/offline behavior | PRD-F11, N04/N05/N09; VAL-004; DEC-013 |
| Resource and overload | Slow disk/readers, stalled coordinator, bounded queue/RSS, shutdown with partial batches and repeated bursts; no silent overwrite | PRD-N01/N03/N06/N08; VAL-005/006/009/022 |

Requirement IDs refer to [Product requirements](../01-product/product-requirements.md); full engineering coverage is in [Engineering validation](../03-engineering/validation-and-traceability.md). Expected results depend on unresolved contracts; a test must record the selected semantics rather than assuming source prose resolves them.

## Concurrency and unsafe-code review

Document exclusive mutation, initialized-before-publication ordering, bounded capacity, reader leases/generations and reuse eligibility for every unsafe boundary. Restrict producer/consumer handles instead of relying on caller discipline through public `&self` methods. Exercise minimal modeled schedules and adversarial stress runs.

Use applicable tools such as Loom for a model using its instrumented synchronization and Miri for supported unsafe-memory tests after an implementation/toolchain is selected. Record model limits, excluded paths and failures; neither tool substitutes for the ownership argument. These tools are proposed additions, not requirements extracted from the PDF or tests run here.

Specifically test capacity before append, writer-owned timer requests, retained registry snapshots and single projection application. Disruptor callback completion is not downstream query completion: event-slot sequencing alone cannot authorize Arrow allocation reuse. Measure `finish()`/`reserve()` allocations and destruction rather than assuming a pool-return protocol exists (pp. 29–42).

## Recovery and semantic oracles

Choose the acknowledgment/commit/replay model before defining restart success. A volatile-only mode may lose accepted but uncommitted data; tests must quantify that window. Background Parquet writing and striping do not establish zero-loss recovery. Inject failure at each commit transition and compare restored event/version identities with the selected model.

Keep query expectations independent of the optimized implementation: small hand-checkable datasets and a straightforward reference calculation for sums, counts, VWAP, joins, buckets and correction knowledge states. Count equality alone cannot detect duplicate/gap compensation. Venue-specific order-book reconstruction requires its own approved scope and fixtures.

## Evidence and exit

Retain fixtures/seeds, expected results, build/tool versions, safety arguments, test logs and failing schedules in the [evidence package](evidence-and-traceability.md). Record passed, failed, blocked and not-applicable cases with justification. Close findings only through their decision/review process; all proposed suites currently remain unexecuted.
