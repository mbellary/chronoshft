# ChronosHFT — Scope and MVP Definition

| Control | Value |
| --- | --- |
| Document | CHRONOS-P2-SCOPE-001 |
| Version / date | 1.0 / 2026-10-13 |
| Status | Proposed scope; DEC-001/002 unresolved |
| Inputs | BASE-001 physical pp. 6–18, 23–28; Phase 0 assessment |
| Companions | [PRD](product-requirements.md); [Product Architecture](product-architecture.md); [Phase 2 handoff](phase-2-definition.md) |

## 1. Scope rules

The user-authorized task is product-definition documentation. It does not itself approve a product boundary, delivery schedule or research technique. Source-stated MVP features are retained even when the source places their delivery in later stages. Proposed dispositions below resolve neither DEC-001 nor DEC-002; an approval record must do that. No implementation or benchmark is included.

## 2. Proposed complete MVP

The research p. 16–17 MVP and functional requirements collectively support a single-node Rust financial TSDB with supported feeds, concurrent recent-data SQL, Parquet history, unified queries, ASOF, OHLCV/time bucketing, correction/as-known behavior, ns representation, and Flight/Python access. This definition includes source FR-3 bitemporality rather than silently omitting it because it appears in Stage 3.

| Scope item | Proposed disposition | PRD / source | Decision boundary |
| --- | --- | --- | --- |
| Single-node Rust + FDAP | Core MVP | F01; RES-002/024 | Version/platform support pending |
| Binance/Coinbase WebSockets, ITCH structures | Core adapter family; exact release coverage pending | F02; RES-025 | First venue, both crypto adapters, live/mock ITCH and version coverage require DEC-002 |
| Concurrent live SQL | Core MVP | F03/F04; RES-017/028 | Visibility/isolation pending |
| ASOF, TIME_BUCKET, OHLCV | Core MVP | F05/F06; RES-010/011 | Financial semantics pending |
| Background Parquet + unified history | Core MVP | F07/F08; RES-018/019 | Flush/commit/recovery pending |
| Late events/corrections + as-known history | Core MVP | F09/F10; RES-012/030/049 | Knowledge/version semantics pending |
| Flight/Python/Polars/Pandas path | Core MVP | F11; RES-013 | Supported client matrix pending |
| Nanosecond representation | Core MVP | F12; RES-040 | Clock/order/overflow rules pending |
| Private/offline runtime | Core MVP | N04; RES-015 | Packaging, feed connectivity and security pending |
| Full L1/L2/L3 reconstruction | Candidate extension | F13; RES-021 | Separate domain and scope decision |
| Object-storage cold path | Optional source proposal | RES-003/018; p. 16 | Local/NVMe first recommended; not approved exclusion |
| Fleet UI, active-passive HA, support SLAs | Future enterprise proposal | RES-031; pp. 17–18 | License and reliability contracts pending |
| Edge/IoT compression and fleet product | Historical alternative, proposed outside financial MVP | RES-006–008; pp. 3–5 | DEC-001 |
| Strategy execution, OMS, pre-trade risk and order gateways | Proposed external boundary | GAP-001; DEC-001 | Research does not define complete execution requirements |
| Sharding/distributed query/consensus | Not established MVP requirement | RES-024/031 | No implicit expansion from fleet/HA wording |

## 3. Delivery slices and gates

These follow the source three-stage dependency sequence (pp. 26–28; RES-043), without adopting months 1–9 as a committed schedule. All gates are proposed, and target figures remain unverified. A stage demonstration is not complete MVP release acceptance.

| Slice | Product increment | Source gate retained | Proposed prerequisite / acceptance refinement |
| --- | --- | --- | --- |
| A — Recent-data capture and SQL | Single node, selected feed/fixtures, ns recent data, concurrent aggregates | 1M ticks/s; query every 100 ms; sub-ms hot P99 | Approved event/schema/overload/visibility contracts; correctness and ownership validation; qualified workload |
| B — Historical continuity | Parquet persistence, unified query, late-data handling | Correct cross-tier aggregate < 50 ms | Approved commit/recovery and catalog handoff; injected-failure and tier-transition fixtures |
| C — Financial research workflow | Flight/Python, ASOF, bucketing/OHLCV, as-known history | Remote 50M-row Polars scenario < 1.5 s | Financial oracle fixtures, supported client versions, qualified output size/network; correction-version evidence |

If the team wants Slice A alone to be the launch MVP, record a deliberate scope change and disposition of F05–F11; do not label it the complete source MVP automatically.

## 4. Explicit noncommitments and unresolved choices

No guaranteed ingestion rate, query freshness, data-loss limit, retention duration, HA RPO/RTO, customer SLA, pricing tier or license is approved here. Their absence is a review item, not a suggestion that production may ignore them.

Ring size 1,024 × 4,096 rows, double buffering, busy spinning, 500 µs publication, 5/10 s flushing and a five-minute hot window are research settings/examples. They are not fixed product limits. Copy-free in-process sharing does not establish copy-free network transfer. Nanosecond resolution does not establish exchange-event order. A sample table name does not establish book reconstruction.

## 5. MVP exit criteria

1. Record approved product boundary and exact feed/capability matrix (DEC-001/002).
2. Ratify source-derived requirements and explicitly accept/reject derived candidates.
3. Approve time/financial/correction semantics and demonstrate oracle agreement.
4. Define accepted/visible/durable states, overload policy, recovery behavior and retention limits.
5. Demonstrate concurrent ingestion/query and correct hot/cold transitions with no silently missing/double-counted eligible events.
6. Supply workload-qualified evidence for every adopted performance target; report losses, memory, durability mode and errors.
7. Demonstrate supported Python workflows, open-format access and private/offline runtime packaging.
8. Complete required security, operational and reliability checks for the approved deployment scope.

These criteria are proposed release conditions derived from the source and Phase 0 gaps. No criterion is marked passed. Named approvers, dates and evidence must be recorded in the project governance system before release.
