# Assumptions & gaps

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

## Gaps

All items are **Open**. Closure is evidence-based; proposed ownership does not grant approval authority.

| ID | Priority / proposed owner | Missing contract or evidence | Source link | Closure criterion |
| --- | --- | --- | --- | --- |
| GAP-001 | Critical / Product | TSDB versus complete trading engine boundary | RES-009,021,024,055 | Approved scope with exclusions and capability map |
| GAP-002 | Critical / Systems | Enforced producer/consumer handles and slot lifecycle | RES-017,051,058 | Design and safety invariants reviewed; adversarial tests |
| GAP-003 | Critical / Query | Stable snapshots, retention and slow-reader policy | RES-019,054,061 | Repeatable reads across publication/reclamation |
| GAP-004 | Critical / Ingestion | Full queue/backpressure/drop/disconnect policy | RES-037,053 | Explicit policy with loss accounting and bounded memory tests |
| GAP-005 | Critical / Storage | Durability point, acknowledgment, WAL choice and crash recovery | RES-018,030 | Approved failure model and injected-failure recovery evidence |
| GAP-006 | Critical / Domain | Sequence gaps, duplicates, ordering and deterministic replay | RES-021,025,041 | Venue/partition rules and replay oracle |
| GAP-007 | Critical / Storage+Query | Atomic hot/cold visibility, no gaps or double counting | RES-019,047 | Snapshot/catalog handoff and concurrency tests |
| GAP-008 | High / Domain | Fixed-point/decimal vs float prices; sizes, instruments and IDs | RES-025,051 | Financial schema/range/rounding contract |
| GAP-009 | High / Domain | ASOF ties/tolerance/direction; buckets, calendars, OHLCV null/empty rules | RES-010–011,028 | Query semantics and expected-output cases |
| GAP-010 | High / Storage | Corrections, lateness limits and bitemporal version semantics | RES-012,030,046,049 | Immutable correction/version rules; late-event tests |
| GAP-011 | High / Engineering | Dependency pins, APIs and runnable complete samples | RES-051,054,060 | Cargo lockfile; buildable examples; compatibility record |
| GAP-012 | High / Performance | Allocation boundaries, pool budgets and retained-reader memory | RES-017,020,044,052 | Measured allocations/deallocations and exhaustion policy |
| GAP-013 | High / Performance | Workload definitions and benchmark methodology | RES-027,029,036–039,045,047–048 | Versioned generators, scripts, results and acceptance contracts |
| GAP-014 | High / Operations | Clock domains, synchronization, timezone, overflow and provenance | RES-040–042 | Timestamp/sequence schema and clock-quality reporting |
| GAP-015 | High / Operations | AuthN/AuthZ, encryption, secrets and feed entitlements | RES-015,025 | Threat model and scoped controls; offline packaging |
| GAP-016 | High / Operations | Observability, start/stop/drain, disk-full and process failure | RES-018,031,035 | Runbooks and injected-failure tests |
| GAP-017 | High / Storage | Partition/pruning/catalog, compaction, retention and disk sizing | RES-018–019,030,034 | Storage lifecycle and capacity model |
| GAP-018 | High / Networking | Adapter protocols, TLS, reconnect, framing and feed recovery | RES-025,035 | Adapter-specific contracts and conformance traces |
| GAP-019 | Medium / Product | Validated personas, customer value and commercial assumptions | RES-009,022–023,031–033 | Evidence register; reviewed positioning and license scope |
| GAP-020 | Medium / Documentation | Bibliography, source versions and baseline change process | RES-063 | Recoverable references or explicit unsupported markers |
| GAP-021 | High / Reliability | HA replication consistency and failover boundary | RES-031 | Separate future-scope decision; if adopted, replication/recovery contract |
| GAP-022 | High / Engineering | Test tiers for unsafe/concurrent, financial and end-to-end behavior | RES-044–048,051,058 | Requirement-linked verification plan; tools chosen by applicability |

## Conflicts and ambiguities

| ID | Priority | Preserved alternatives | Interpretation / resolution required |
| --- | --- | --- | --- |
| CON-001 | Critical | Edge/IoT pp. 3–5 versus financial TSDB pp. 6–17 | Earlier exploration is not silently promoted to MVP; DEC-001 |
| CON-002 | High | 8-core 1M pp. 17 vs 32-core 2.5M/8M pp. 21–23 | Distinct profiles, not necessarily contradiction; DEC-005 |
| CON-003 | High | 5 s flush p. 9 vs 10 s pp. 16/27; 500k ticks in both | Select configurable policy and distinguish persistence/publication; DEC-004 |
| CON-004 | Critical | Mutable Arrow views vs immutable finished arrays pp. 9/16/31/35/38 | Decide transfer/retain/recycle model; DEC-003 |
| CON-005 | Critical | Nonblocking writer guarantee vs finite ring and spin/drop/panic p. 31 | Cannot guarantee losslessness, bounded memory and unlimited producer progress simultaneously under stalled consumers; DEC-006 |
| CON-006 | High | Timestamp(ns) pp. 24–25 vs Int64 schema p. 29 | Semantic type mismatch; DEC-007 |
| CON-007 | High | 500 µs/1k rows p. 36 vs 4,096 rows p. 34 vs optional timer | Distinct examples, no canonical freshness contract; DEC-004 |
| CON-008 | High | 100 ms gate pp. 26–27 vs 10 ms query example p. 35 | Benchmark cadence versus sample behavior; DEC-005 |
| CON-009 | High | Near-term MVP financial features pp. 16–17 vs Stage 3 months 7–9 | Align MVP capability scope and roadmap; DEC-002 |
| CON-010 | High | Six DIMMs fully populated p. 21 vs twelve channels [EXT-03] | Correct proposed hardware population statement; DEC-011 |
| CON-011 | High | Arrow timezone None p. 24 vs UTC-adjusted Parquet p. 25 | Specify instant/local-time semantics and conversions; DEC-007 |
| CON-012 | Medium | Core open-source pp. 17–18 vs proprietary-layer language pp. 12–13 | Define license and component boundary; DEC-012 |

## Handling assumptions

An unspecified contract remains open rather than becoming an implicit requirement. Owners are proposed roles, all records have no committed due date, and closure requires the evidence listed above. Conflicting proposals remain visible until a reviewed decision resolves them.

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
