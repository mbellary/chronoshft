# Ingestion engine

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Source intent and adapter scope

Research pp. 8–9, 16, 26–27 propose direct feed decoding into preallocated Arrow columns. WebSockets (Binance/Coinbase), ITCH structures and generic FIX/ITCH/WebSocket context are named; the exact venue/version and live versus mock coverage remain DEC-002. No complete FIX adapter or feed entitlement contract is supplied (RES-025; VAL-025). Implements proposed ENG-002–004/006/007.

## Proposed decode and admission pipeline

| Step | Behavior | Failure contract to resolve |
| --- | --- | --- |
| Receive/frame | Decode declared transport framing, session and packet boundaries | Disconnect, truncated frame, oversized message, TLS/reconnect |
| Validate | Check event type, instrument, numeric/time ranges and schema | Malformed/unsupported inputs produce explicit counted outcomes |
| Sequence | Preserve venue partition/session and sequence; apply declared gap/duplicate policy | Decide replay/resync/quarantine or explicitly degraded visibility |
| Reorder if selected | Maintain a bounded micro-sort window (Stage 2, p. 27) | Declare row/byte/time limit, tie-break and beyond-window route |
| Admit | Check queue and append capacity before mutating columns | Apply DEC-006 backpressure/rejection/drop/disconnect policy |
| Append | One writer owns all mutable columns; append one complete validated row | No partial-row publication; handle construction/append errors |
| Freeze/publish | Freeze a partial/full initialized batch; notify retained registry | If coordinator is full, retain ownership and apply bounded policy |

Admission and malformed-input policies are proposed refinements. The source ring checks fullness after append and may keep appending after failed rotation; reverse that ordering and prove bounds (VAL-012). A successful API call must identify what was accepted and what remains uncommitted.

## Publication and timer ownership

The example uses 4,096-row batches; double buffering proposes 500 µs or 1,000 rows (pp. 34–39). Define configurable thresholds and precedence under DEC-004. Send timer requests to the exclusive writer; a timer must not mutate builders concurrently with append. Empty timers produce no duplicate batch. Partial batches need correct lengths, sequence coverage and identity; missed timer intervals need an explicit freshness policy.

Published means eligible for a new query snapshot. The low-volume timer, 100 ms source query cadence and 5/10 s cold-flush examples do not jointly define a freshness SLA.

## Allocation and overload accounting

Preallocate declared decoder/writer capacities and measure append, rotation, notification and reclamation separately. JSON decoding, network buffers, builder turnover, reference wrappers, logging and destruction may allocate. Existing VAL-014 records that builder `finish()` transfers values rather than retaining capacity in that builder; no zero-allocation result is claimed.

Track received, malformed, duplicate, gap-affected, admitted, rejected/dropped, published and committed outcomes with mutually defined categories. Source replay availability and retry identity determine whether retries can be safe. A bounded spin is not a complete saturation policy, and a live feed may not tolerate indefinite upstream blocking (DEC-006).

## Verification and shutdown

Exercise full/wrapped queues, burst input, zero/partial batches, low-volume timers, malformed payloads, equal-time events, duplicate/reconnect/gap recovery and delayed coordinator/persistence. A shutdown proposal stops admission, resolves the active partial batch, drains selected work and reports accepted-but-uncommitted events under DEC-008. Use [schema contracts](event-schema-and-sequencing.md), [memory lifecycle](active-memory-lifecycle.md) and [validation](validation-and-traceability.md).

Architecture trace: [component contracts](../02-architecture/component-contracts.md), [data flow](../02-architecture/data-flow.md). Product trace: F02/F03/F12, N03/N06; CAP-01/02/08.
