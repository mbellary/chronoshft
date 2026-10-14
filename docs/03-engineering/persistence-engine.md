# Persistence engine

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Source baseline and open choices

The research proposes sequential background micro-Parquet writes, unified hot/cold routing and bounded micro-sorting (pp. 9, 16, 27–28; RES-018/019/046/049). Flush settings conflict: 5 s or 500k ticks on p. 9 versus 10 s or 500k ticks on pp. 16/27 (CON-003). Publication is a separate clock. Parquet/background writes alone do not define crash-consistent durability (VAL-024). Implements proposed ENG-010–012 and [Architecture query/storage](../02-architecture/query-and-storage.md).

## Proposed commit lifecycle

| State | Required contract |
| --- | --- |
| Queued | Hold immutable batch ownership; apply bounded backlog/admission policy |
| Writing | Write an output excluded from query-visible catalog membership |
| Validated | Complete file metadata/schema/coverage checks and chosen durability steps |
| Committed | Publish file identity and coverage/version metadata under the selected atomic visibility/recovery protocol |
| Hot retirement eligible | Future snapshots select each event/version once; existing reader references stay valid |
| Reclaimed | All owners release storage and selected memory reuse protocol permits it |

Temporary-output naming, synchronization, atomic metadata updates, checksums and restart reconciliation are candidate mechanisms to select under DEC-008, not source-defined implementations. Specify failure behavior at every state and preserve batch ownership through retry or explicit failure disposition.

## Acknowledgment and recovery alternatives

| Candidate mode | Required disclosure / evidence |
| --- | --- |
| Volatile admission with asynchronous flush | Accepted-but-uncommitted data can be lost; define query visibility after restart |
| WAL-backed admission | Define log commit boundary, ordering, replay identity, truncation and duplicate prevention |
| Source-replay recovery | Verify venue/session retention and replay availability; reconcile acknowledged/query-visible ranges |

No mode is selected here. Process crash, power loss, file corruption and disk loss are different failure models; a promise covering one does not cover all. NVMe striping improves neither redundancy nor the catalog recovery contract automatically.

## Storage layout and retention decisions

Select instrument/venue/time partitioning, file naming, row-group sizing, encoding/compression, time/sequence statistics, file-count budgets and schema evolution. The PDF asks for naming/pruning details but does not supply the algorithm. Local NVMe is the working reference; object storage remains optional and requires its own commit/consistency design (DEC-002).

At the reference 2.5M ticks/s × 48 bytes, raw payload is 120 MB/s or 10.368 TB/day. Two striped 3.84 TB drives nominally hold about 17.8 hours at that rate before compression/overhead; these are feasibility calculations, not measured capacity guarantees. Define retention and headroom from the actual resident/file schema and measured compression ([performance profiles](../00-governance/performance-profiles.md)).

## Corrections and compaction

Bounded micro-sort can handle a configured delay window but cannot represent arbitrary retroactive corrections. Proposed design preserves stable event identity and knowledge/version metadata; late versions must remain query-eligible even if they arrive outside ordinary event-time partitions. Choose delta/version files or another approved mechanism under DEC-009; no algorithm is assumed.

Compaction must preserve declared current/as-known results, publish replacement coverage atomically, and defer old-file deletion until active snapshots finish. Deduplication cannot erase legitimate equal-time events. Hot/cold handoff must avoid both gaps and duplicate aggregates (ENG-011/012).

## Failure and compatibility evidence

Inject failure before file completion, before/after catalog publication, during retries, hot eviction and compaction. Restart must reconcile partial/orphan outputs and committed metadata according to the chosen protocol. Verify disk-full/permission/corruption responses, slow-writer pressure, correction visibility and nanosecond/type round trips. External readers must interpret the documented Parquet schema; raw-file access alone does not implement the engine's correction-aware view.

Product trace: F07–F10/F12, N07–N09; CAP-04/05/06/08. Decisions: DEC-004/008/009; gaps GAP-005/007/010. Related: [query engine](query-engine.md), [schema](event-schema-and-sequencing.md), [validation](validation-and-traceability.md).
