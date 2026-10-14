# Flight and Python integration

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Source intent and engineering boundary

Research pp. 6, 17, 27–28 proposes Arrow Flight result delivery into Python/Polars/Pandas/Jupyter, including a remote 50M-row ASOF scenario. Product F11/CAP-07 and Architecture's client surface require a typed, observable contract (ENG-013). Existing VAL-004 qualifies absolute zero-copy wording: Flight uses transport/Arrow IPC processing, and remote processes do not share an address space.

## Proposed service/client contract

| Concern | Definition required before implementation acceptance |
| --- | --- |
| Query request | Supported RPC shape, SQL subset, parameters and request/session identity |
| Result schema | Arrow types/nullability/timezone/precision plus schema/version compatibility |
| Authentication/access | Approved client/query permissions, credentials and transport protection under N05 |
| Streaming | Batch boundaries, completion, partial-stream errors and retry behavior |
| Cancellation | Client disconnect/timeout propagates to execution and releases snapshot/result references |
| Resource controls | Concurrent-query, result-byte and slow-client limits; declared backpressure behavior |
| Python conversion | Supported Python/Arrow/Polars/Pandas versions and type-specific conversion behavior |

The PDF does not establish Flight SQL, subscriptions, REST endpoints or a concrete client package API. `pip install chronoshft` is source roadmap intent, not evidence that a package exists. Pin and document the chosen service/client API under DEC-013.

## Ownership and copy boundaries

Separate in-process immutable Arrow sharing, server result construction, IPC encoding, network transfer, client decoding and dataframe conversion. Report allocations/copies per boundary and supported type; eliminating JSON/CSV parsing does not eliminate all copying or serialization. Keep result/snapshot ownership until streaming finishes or is cancelled, and budget slow-client retention (ENG-005/006).

## Type and failure verification

Round-trip nanosecond timestamp semantics, nulls, chosen price/quantity precision, identifiers and supported nested/dictionary types if included. Exercise empty results, multiple batches, cancellation, disconnect, server execution failure and mid-stream error. Define whether a client returns incomplete data or raises an error; an incomplete stream must not appear successfully complete.

Check local and remote clients independently. Validate Parquet interoperability separately from dataframe conversion; raw Arrow/Parquet compatibility is not a guarantee that every client preserves every selected type without conversion.

## Performance evidence

The Stage 3 source target is 50M rows into remote Polars < 1.5 s with SQL containing ASOF (pp. 27–28; PERF-STAGE-3). Record input versus output row count, row width, join selectivity, encoded bytes, compression, network, server/client hardware, cache state and endpoint through usable dataframe completion. Keep the reference 4.2 GB/s egress scenario separate (PERF-REF-EGRESS). No target is measured or approved by this documentation.

Related: [query engine](query-engine.md), [schema](event-schema-and-sequencing.md), [validation](validation-and-traceability.md), [component contracts](../02-architecture/component-contracts.md). Governance: DEC-013; GAP-011/013/015; VAL-004/017/018.
