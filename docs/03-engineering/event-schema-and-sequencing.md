# Event schema and sequencing

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Source and schema mismatch

Research pp. 23–26 proposes signed 64-bit epoch nanoseconds, Arrow `Timestamp(Nanosecond, ...)` and Parquet `TIMESTAMP(isAdjustedToUTC=true, unit=NANOS)` (RES-040–042). The ring sample instead declares non-null `Int64` timestamp, `Float64` price and `Int32` size (pp. 29–30). Treat it as a demonstration, not a ratified financial schema (VAL-018). This section refines [Architecture platform design](../02-architecture/platform-design.md), ENG-007 and PRD-F12/CAP-08.

## Proposed cross-layer schema contract

| Concern | Definition to ratify | Verification |
| --- | --- | --- |
| Event time | Epoch, unit, timezone/UTC semantics, supported range and source resolution | Arrow → Parquet → query → Flight → dataframe round trip without truncation |
| Receive / knowledge time | Capture point, clock source and version visibility meaning | Distinguish event time from when the engine learned a version |
| Instrument and venue | Stable mapping, identifier namespace and session/partition provenance | Same timestamp across instruments never causes accidental matching/deduplication |
| Source sequence | Sequence scope, reconnect reset and gap/wrap rules | Deterministic replay with equal timestamps and out-of-order arrivals |
| Event identity/version | Stable identity and correction/amend/cancel relation | Retries are accounted for without dropping distinct events |
| Price and quantity | Decimal/fixed-point or floating-point, scale, units and overflow | Exact expected precision, negative/invalid values and aggregation limits |
| Nullability | Meaning of missing fields versus invalid events | Nullable/non-null constraints and client conversions stay consistent |
| Schema version | Compatibility and migration across active/history/results | Mixed-schema scans produce defined results or explicit errors |

These are derived design concerns; field names and physical layouts require DEC-007/009/014. The eight-byte nanosecond representation has a finite signed range (about ±292 years around the epoch); reject/handle overflow rather than wrapping silently (VAL-010).

## Ordering and late events

Nanosecond resolution is neither clock accuracy nor a total event order (VAL-011). Propose a deterministic venue/partition sequence tie-break where valid; cross-partition ordering needs an explicit rule. Preserve source precision even if an adapter only provides millisecond/microsecond timestamps; representation does not create finer source accuracy.

Define gap and duplicate policies per adapter, bounded reorder-window limits and treatment of events beyond that window. A repeated timestamp is not necessarily a duplicate. Keep ordering metadata through persistence and financial execution so open/close and ASOF ties are reproducible.

## Acceptance and review

Test equal-time events, partition/session resets, duplicate replay, unknown instruments, timestamp extremes, precision loss, nullable fields, numeric overflow and schema evolution. Include current/as-known correction round trips. Coordinate Domain/Engineering review with GAP-006/008/010/014; proposed roles are not assigned owners.

Source trace: RES-040–042/051; VAL-010/011/018/023. Related: [ingestion](ingestion-engine.md), [financial semantics](financial-query-semantics.md), [Flight/Python](flight-and-python.md).
