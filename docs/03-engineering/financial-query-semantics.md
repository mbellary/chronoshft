# Financial query semantics

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Capability boundary

The PDF proposes ASOF matching, bucketing/OHLCV, tick windows, VWAP and bitemporal lookup (pp. 6–10, 17, 23, 28; RES-010–012/021/030/049). Product F05/F06/F09/F10 are proposed core capabilities; F13 book reconstruction is a candidate extension. This Engineering section defines reviewable operator contracts for ENG-009/012, complementing [Architecture query/storage](../02-architecture/query-and-storage.md). Algorithms and supported SQL syntax remain version-dependent implementation choices.

## Proposed ASOF contract to ratify

The product fixture suggests preceding quote matching: for a trade at 10:00:00.102, a quote at 10:00:00.098 is eligible (p. 6). Define match keys (instrument/venue as required), direction, inclusive/exclusive boundary, tolerance, tie-break among equal-time quotes, unmatched-row/null behavior and required ordering. Prevent cross-instrument matches and unintended look-ahead. Other directions are optional until explicitly specified.

Oracle fixtures must include no quote, multiple equal-time quotes, tolerance boundary, different instruments, unsorted inputs, null times and a quote correction arriving after the trade. A current versus as-known view may deliberately select different versions; both must follow the declared contract.

## Bucketing, OHLCV and VWAP

| Operator concern | Proposed semantic decision | Oracle case |
| --- | --- | --- |
| TIME_BUCKET | Duration, origin, timezone/calendar and boundary inclusion | Event exactly on adjacent bucket boundary |
| Open/close | First/last eligible event using declared total order | Equal timestamps with distinct venue sequence |
| High/low | Numeric precision and invalid/null handling | Negative/invalid/missing prices under policy |
| Volume | Quantity unit, trade eligibility and correction/cancel handling | Amended/cancelled trade and zero quantity |
| Empty bars | Omit, emit nulls or fill under declared policy | Gap spanning multiple buckets |
| Tick windows | Count definition, partition/reset and version handling | Duplicate/replay and late event at boundary |
| VWAP | Eligible price × quantity arithmetic and zero-volume result | Zero denominator and precision/overflow limits |

OHLCV and TIME_BUCKET are source capability intent, not a completed function specification. Tick-window/VWAP mention does not automatically add release scope beyond Product. Choose functions/planner rules/custom execution operators only after semantics and supported DataFusion version are fixed (VAL-026).

## Current and as-known history

Event time describes market occurrence; knowledge time describes when a version became known. Define version identity, correction/cancellation relation, knowledge cutoff inclusion, clock source and how multiple versions resolve. Preserve historical knowledge states through flush and compaction (DEC-009; GAP-010).

A proposed fixture contains an original trade, a late arrival, an amendment and a cancellation. Before/after knowledge cutoffs must yield independently expected results over hot-only, cold-only and hybrid sources. A micro-sort window cannot substitute for this version model; timestamp precision alone cannot establish it.

## Optional order-book reconstruction

A table named `live_order_book` in the source aggregate demo (p. 34) contains timestamp/price/size and does not reconstruct L1/L2/L3. Adoption needs venue add/modify/delete/trade semantics, order IDs/priority where relevant, snapshot/incremental synchronization, gap recovery and deterministic reference traces (VAL-027; DEC-014). Retain the candidate boundary until scope is ratified.

## Acceptance trace

Independent domain oracles should precede throughput claims. Trace fixtures to PRD-F05/F06/F09/F10 and CAP-03/06; optional books to F13/CAP-10. Related: [schema and sequencing](event-schema-and-sequencing.md), [query engine](query-engine.md), [validation](validation-and-traceability.md), [Financial domain overview](../05-financial-domain/financial-query-semantics.md).
