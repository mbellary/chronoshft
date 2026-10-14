# Benchmark methodology

!!! info "Research-derived validation plan · 2026-10-14"
    The [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical pp. 17, 21–23, 26–28 and 29–42) supply targets and prototypes. [Architecture](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md) supply the boundaries. Procedures below are proposed validation refinements; no benchmark has been executed or acceptance gate approved.

## Measurement contract

Before a run, select one [workload profile](workload-definitions.md), its requirement IDs and the exact implementation revision. Record schema, adapter, event/byte counts, query text, window and cardinality, concurrency, cache state, duration, warmup, repetitions, publication/flush thresholds, retention, overload policy and durability mode. Attach the [hardware manifest](hardware-setup.md). Unresolved settings must be explicit rather than filled from unrelated source examples (DEC-004/005/006/008/011).

| Metric | Start → end | Required accounting |
| --- | --- | --- |
| Feed capture | Generator send → adapter receipt | Packets, messages, sequence gaps and decoder errors; generator bottlenecks |
| Admission throughput | Declared adapter/writer entry → accepted/rejected outcome | Offered, decoded, eligible, accepted and rejected events; per-thread versus per-node scope |
| Query freshness | Local receipt/admission → eligibility in a new snapshot | Publication delay, timer misses and partial batches; separate from query runtime |
| Engine query latency | Declared engine request entry → complete result execution | Include or separately disclose planning, queueing and execution; first batch alone is not completion |
| Client query latency | Client request → complete result consumed | Transport and client conversion included; report first-result latency separately |
| Durable lag | Accepted event → selected durable commit | Acknowledgment/failure model, pending rows and catalog visibility |
| Flight dataframe load | Client SQL submission → verified dataframe materialization | Input/output rows, actual bytes, query/transfer/conversion timings and peak memory |
| Storage footprint | Complete reference dataset → all files within declared boundary | Data, indexes/catalog and metadata; lossless round trip and compaction state |

Use monotonic elapsed timing. Document clock resolution, timestamp overhead and any TSC calibration; cross-host receipt-to-delivery timing requires declared clock synchronization and uncertainty. Venue event time is not a local processing timer.

## Run procedure

1. Validate the dataset and query oracle before timing. Pin all dependency/build versions and archive the manifest.
2. Preallocate and warm the selected paths for a declared interval. Record startup separately; do not exclude steady-state rotations, reclamation or recurring maintenance.
3. Schedule offered traffic independently of completion so saturation cannot silently reduce the offered load. Record planned arrival, actual send and completion times; disclose generator lag and avoid coordinated omission.
4. Run ingestion alone, then with queries, then with persistence/maintenance and clients. Use identical inputs and disclosed configuration changes to expose interference.
5. Repeat measured trials with fixed seeds and declared durations. Report per-trial distributions and variability; do not select only the best run.
6. Reconcile events and oracle results, measure backlog recovery, and archive raw data before assessing any target.

Duration, repetition count and sample sufficiency are contract fields pending DEC-005, not source-approved constants. P99.9 needs enough observations to characterize the tail and trial variability; disclose sample counts and uncertainty.

## Latency, loss and resource reporting

Report P50/P95/P99/P99.9 where applicable, maxima, sample counts, errors, timeouts and incomplete requests. Retain raw samples or mergeable histograms and report their resolution. Do not discard failed requests from the headline without reporting their rate and timing treatment. Query every 100 ms (Stage 1) is scheduling cadence; it establishes neither P99 latency nor receipt-to-visible freshness.

At consistent cutoffs reconcile eligible input with acceptance/rejection, duplicates, published snapshots and committed data. A row count alone cannot reveal compensating gaps and duplicates; retain event identities/sequences or checksums. Report packet and event loss separately. Zero loss applies only to the declared tested envelope, not unbounded overload.

Instrument allocations/deallocations, resident and peak memory, page faults, CPU/cache behavior, storage throughput and queue occupancy. Attribute allocations to decode, append, freeze/rotation, registry insertion, query, persistence and reclamation. The PDF's builder `finish()`/`reserve()` and `Arc` examples do not prove allocation-free turnover (VAL-014/020). Run profiling separately where its overhead changes timing and disclose instrumentation differences. The source suggests allocator auditing such as jemalloc (p. 26); tool choice remains implementation-dependent.

## Comparisons and claim limits

Match datasets, semantics, output widths, hardware, cache/durability settings and concurrent maintenance when comparing databases. Pin competitor versions and preserve scripts. PERF-DX's 20× loading claim, < 100 ns signaling and supplied competitor tail/ODBC figures remain hypotheses, not verified baselines. Flight transport does not guarantee zero encoding or conversion (VAL-004/022).

## Sources and evidence

- [Performance profile ledger](../00-governance/performance-profiles.md) and [Product benchmark targets](../01-product/benchmark-targets.md): retain distinct source and user-proposed profiles.
- [Component contracts](../02-architecture/component-contracts.md) and [data flow](../02-architecture/data-flow.md): accepted, published, durable and recyclable are separate states.
- [Test strategy](test-strategy.md), [acceptance criteria](acceptance-criteria.md) and [evidence and traceability](evidence-and-traceability.md): correctness precedes performance acceptance.
