# Validation and traceability

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Evidence status

Engineering documentation is populated; engineering approval and executable validation remain pending. This page provides a proposed evidence plan for DOC-06/07 using the existing [technical validation register](../00-governance/technical-validation.md), [decision register](../00-governance/decision-register.md) and [Product readiness](../01-product/product-readiness.md). No new VAL, GAP or DEC record is marked closed.

## Research-to-engineering map

| Physical PDF pages / source IDs | Engineering destination | Product / architecture dependency |
| --- | --- | --- |
| 2, 11–17, 26; RES-002/024 | ENG-001; engineering design | F01/CAP-01/03; system architecture, platform design |
| 8–9, 16, 26–27; RES-025/044/046 | Ingestion; ENG-002–004/006 | F02/F03/N03/N06; adapter/writer contracts |
| 29–32; RES-051–053 | Memory lifecycle and bounded admission | F03/N03/N08; active memory; VAL-012–014 |
| 33–36; RES-054–057 | Non-destructive query provider and plan correctness | F03/F04; query/storage; VAL-015–018 |
| 36–39; RES-058/059 | Writer-owned timer/double-buffer lifetime | F03; data flow; VAL-019/020 |
| 39–42; RES-060–062 | Batch coordinator and reference retirement | F03/N08; active memory; VAL-021/022 |
| 23–26; RES-040–042 | Schema and sequencing; ENG-007 | F12/CAP-08; financial data model; VAL-010/011/018 |
| 9, 16, 27–28; RES-018/019/030/046/049 | Persistence, handoff and correction versions | F07–F10/CAP-04–06; data flow; VAL-023/024 |
| 6–10, 17, 23, 28; RES-010–013/021 | Financial semantics and Flight/Python | F05/F06/F11; optional F13; VAL-004/026/027 |
| 17, 21–23, 26–28, 42; RES-027/029/034–039/045/047/048/062 | Workload-qualified performance evidence | N01/N02; deployment topology; DEC-005/011 |

Page references were extracted directly from the supplied PDF; prototype critique follows the dated Phase 0 assessment, not a new upstream API audit. Architecture registry/catalog/lifetime refinements remain proposals. The [source PDF](../assets/research/HFT-Research-Notes.pdf) and [extraction matrix](../00-governance/research-extraction.md) retain the original evidence boundary.

## Proposed verification tiers

| Tier | Evidence required | Requirement / finding coverage |
| --- | --- | --- |
| Dependency/build | Pinned Rust/FDAP/ring/client set, complete compiling examples and offline artifact inventory | ENG-001/013/015; VAL-017; GAP-011 |
| State and memory safety | Restricted handles, documented unsafe invariants, applicable model/unsafe-memory tooling, wrap/full/partial/slow-reader schedules | ENG-003–006; VAL-012–014/019–022 |
| Adapter/schema | Venue fixtures, gaps/reconnect/replay, numeric/time/null/precision round trips | ENG-002/007; VAL-010/011/018/025 |
| Query/domain | Independent oracles for projection/filter/limit, ASOF, bars, correction knowledge states | ENG-005/008/009/012; VAL-015/016/023/026/027 |
| Persistence/recovery | Crash-point, disk-full, retry, catalog handoff, compaction and restart fixtures under chosen failure model | ENG-010–012; VAL-023/024 |
| Client/access | Supported dataframe matrix, cancellation/partial failures, permission controls and offline local operation | ENG-013/015; VAL-004; GAP-015 |
| Performance | Correctness under load, distributions and loss/resource/freshness/durability records | ENG-006/014; VAL-005/006/009/022 |

Choose tools by applicability once implementation exists; this table is a validation design, not a report of executed tests. A crash-free stress run alone cannot prove ownership safety or financial correctness.

## Benchmark contract checklist

Each adopted profile needs an exact query, adapter/schema, row/byte volume, duration, warm/cold cache state, concurrent clients, machine/NUMA/affinity, dependency versions, publication policy and durability mode. Define timing endpoints and sample count; report P50/P95/P99 where relevant, errors/losses, memory/backlog, allocations/deallocations and page faults. Archive harness, inputs and results so claims are reproducible.

Use separate MVP, EPYC reference, burst, hybrid and Flight workloads. Query every 100 ms is scheduling cadence; P99 < 1 ms is execution latency; neither specifies receipt-to-visible freshness. Reconcile 5/10 s flush (CON-003) and 500 µs/1k versus 4,096 rows publication (CON-007) through DEC-004. See [performance profiles](../00-governance/performance-profiles.md) and [benchmark targets](../01-product/benchmark-targets.md).

## Open-decision handoff

| Decision / gap | Concrete review artifact | Proposed reviewing role |
| --- | --- | --- |
| DEC-001/002; GAP-001/018 | Approved financial boundary, full MVP/slices and venue/version matrix | Product + Domain |
| DEC-003; GAP-002/003/012 | Ownership alternative, restricted API, reuse proof and complete memory budget | Engineering + Architecture |
| DEC-004/006; GAP-004 | Separate freshness/flush policies, saturation and loss-accounting contract | Product + Engineering |
| DEC-005/011; GAP-013 | Workload-specific performance acceptance and actual hardware plan | Validation + Performance |
| DEC-007/014; GAP-006/008/009/014 | Schema, sequence, clocks and financial operator oracle definitions | Domain + Engineering |
| DEC-008/009; GAP-005/007/010 | Failure/acknowledgment/commit model and versioned catalog/compaction protocol | Architecture + Engineering |
| DEC-013; GAP-011/015 | Pinned Flight/Python API/types plus access/cancellation/resource contract | Client Engineering + Security |
| GAP-022 | Requirement-linked evidence suite and applicable concurrency/unsafe tools | Engineering + Validation |

Roles are proposed; named owners, approval dates and evidence remain unassigned. This handoff does not create approval records.

## Delivery and exit assessment

Follow source slices: recent-data capture/SQL → historical continuity → Flight/financial workflow (pp. 26–28). The source month 1–9 estimates are not commitments, and a hot-path demonstration alone is not the complete proposed MVP. Keep bitemporality/ASOF disposition explicit when staging delivery ([Scope and MVP](../01-product/scope-and-mvp.md)).

Documentation completion means substantive pages, source/requirement links and a navigable strict build. Engineering baseline approval additionally needs ratified contracts/decisions; release acceptance needs compiling implementation and retained safety, correctness, recovery and qualified benchmark evidence. Documentation integration passes do not close those gates.

## Requirement-to-executable evidence contract

The [Product claim map](../01-product/product-traceability.md#product-to-engineering-evidence-map) links directly to this canonical engineering map. Planned file paths below are destinations, not existing executable tests; there is no Rust crate or retained benchmark output in this repository. Do not render nonexistent files as successful evidence links.

| Product requirement / claim | Engineering gate and planned Rust integration file | Required benchmark/evidence output |
| --- | --- | --- |
| F03 / N08: stable active-memory query snapshot | ENG-003/005; `tests/active_memory_isolation.rs`; delayed-reader, cancellation, wraparound and reclamation assertions | `active-memory-isolation/test-results.json`, lifetime/model logs |
| F08: instrument/time pruning with correct results | ENG-008/010/012; `tests/parquet_pruning.rs`; unpruned oracle, unrelated-symbol exclusion, boundary/late-data assertions | `parquet-pruning/summary.json`, scan counters and fixture hashes |
| N03: scoped zero-allocation ingestion | ENG-003/004/014; `tests/ingestion_zero_alloc.rs`; allocator counters and negative control | `ingestion-zero-alloc/allocator.json`, CI test log |
| N01/N06: 0% drops at 5,000,000 ticks/sec | ENG-006/014; `tests/ingestion_loss_5m.rs`; unique identity/count reconciliation under specified duration and concurrent query/flush load | `ingestion-loss-5m/summary.json`, raw counts/rate samples and queue high-water/drain records |

The 5M/s case is a requested additional verification profile, not a measured result or replacement for the existing 1–2M sustained, 2.5M reference and 8M burst profiles. Duration, fixture, repetition count, admission and durability boundaries must be fixed before accepting it. Verify the generator actually offered 5M eligible events/sec; do not substitute achieved throughput after silently rejecting events. Reconcile offered eligible unique identities with accepted and visible identities after drain, and durable identities if durability is within scope. Require zero rejected/dropped eligible events, no unexplained sequence gaps, duplicates or missing IDs, plus bounded backlog. A rounded displayed percentage cannot pass a nonzero loss count.

Each output lives under `evidence/<run-id>/`, accompanied by `hardware-manifest.json`, `inputs.json`, raw data and test logs. `summary.json` records product/ENG/VAL IDs, workload profile, test file and test name, source commit, harness command, manifest/input hashes, timing/count boundaries and pass/fail. The evidence index must expose immutable repository URLs pinned to the tested commit for Rust files and retained artifact URLs for outputs, with reciprocal requirement IDs. CI checks link existence, commit/hash agreement and the gate result before any claim is marked verified. Until artifacts exist, status stays **planned / unverified**; no VAL or DEC closes automatically.
