# ChronosHFT — Market Landscape, Challenges and the FDAP Opportunity

| Control | Value |
| --- | --- |
| Document ID | CHRONOS-P2-MARKET-001 |
| Version / date | 1.0 / 2026-10-13 |
| Status | Baseline-derived product context; market and competitive claims unverified |
| Source | BASE-001: HFT - Research Notes.pdf, 42 physical PDF pages |
| Source SHA-256 | `18b5ed26b6f4dfd8a83c452abf7cb81946f338078e66c85c71f48343631f37a2` |
| Relationship | Supporting Phase 2 document; does not replace DOC-02 or DOC-03 |
| Companion documents | [PRD](product-requirements.md), [Product Architecture](product-architecture.md), [Scope and MVP](scope-and-mvp.md), [Phase 2 handoff](phase-2-definition.md) |

## 1. Purpose and evidence boundary

This document explains the market landscape described in the October 2026 research baseline, the problems it identifies, and how Flight, DataFusion, Arrow and Parquet (FDAP) support ChronosHFT's proposed response. It connects commercial motivation to product capabilities and identifies the engineering work that the foundation does not supply.

“Current market” here means the landscape recorded by the baseline, not a separately refreshed market study. No market size, growth rate, market share, customer interviews or independently verified competitor benchmarks are supplied by the PDF. Its numbered references do not resolve to a complete bibliography (Phase 0 RES-063). Market assertions remain research hypotheses until supported by external evidence. Physical PDF pages are used throughout.

The original PDF remains unchanged. Source statements are paraphrased; qualifications and proposed validation actions are separate assessment findings. No new approved product scope, technology, license or performance promise is introduced.

## 2. Market landscape described by the research

The baseline describes a crowded analytical/time-series database market in which reusable open-source internals reduce the cost of developing a database foundation. It argues that differentiation increasingly depends on domain-specific behavior and developer experience rather than access to a generic columnar engine (pp. 1–3; RES-001/002/005).

| Research trend | Implication for a new product | Qualification |
| --- | --- | --- |
| Reusable memory, execution, storage and transport primitives | A team can focus effort on integration and domain capabilities | Reduced foundational effort does not prove production readiness or a short delivery schedule |
| Compute/storage separation and open-format cloud storage | Customers can retain data in formats independent of one query engine | This general market context does not mandate cloud storage or distribution in the single-node MVP |
| Interoperability with analytics/data-science tools | Established formats can reduce connector and conversion work | Compatibility depends on types, versions, APIs and conversion behavior; not universal zero-cost access |
| Demand across finance, IoT and observability | Several markets share high-volume time-series needs | Demand and growth are asserted, not quantified by an evidence-backed market study |
| Accessible foundational technology | The generic foundation alone provides weak differentiation | A durable advantage must be demonstrated through complete workflows and reliable domain behavior |

Source: BASE-001 pp. 1–3; RES-001–005. The claim that a production-grade database can be built over a weekend is retained as an unsupported source assertion, not adopted as a planning estimate.

## 3. From broad time-series opportunity to the financial focus

The research initially explores edge/IoT storage, constrained hardware, intermittent connectivity, compaction and fleet management (pp. 3–5; RES-006–008). It then develops a financial TSDB proposal centered on quantitative developer experience, real-time capture and concurrent analysis (pp. 6–17; RES-009/023).

These are different product hypotheses. The edge exploration explains the search for specialization; it does not establish a financial MVP requirement for Gorilla/Chimp/Sprintz compression, edge rollups or a cloud fleet control plane. Their proposed scope disposition remains in the Scope and MVP document and DEC-001.

The financial proposal targets mid-market hedge funds, prop trading desks, digital-asset market makers and boutique asset managers. The research argues that these teams need capable infrastructure without recreating all database internals or maintaining extensive custom data plumbing (pp. 6–8, 14–18). Customer demand, purchasing behavior and willingness to pay remain unvalidated hypotheses (GAP-019).

| User group | Problem described by the baseline | Proposed benefit |
| --- | --- | --- |
| Quantitative researchers | Data preparation and movement interrupt SQL/Python research | Faster access to typed market data and financial query primitives |
| Trading-data engineers | Bespoke tick capture and live/historical integration require substantial engineering | Reusable capture, storage and unified-query foundation |
| Prop desks and mid-market funds | Specialized incumbent tooling or custom systems create adoption and maintenance burdens | Familiar SQL/Python workflows and customer-controlled deployment |
| Risk analysts and monitoring users | Recent market data must be queryable during capture | Concurrent recent-data analysis with a qualified latency contract |

Source: pp. 6–10, 14–18; RES-009/022/023. These describe a financial data platform; they do not establish a complete trading-execution system or a profitability guarantee.

## 4. The FDAP foundation

| Component | Role described in the baseline | Challenge it helps address | Additional ChronosHFT responsibility |
| --- | --- | --- | --- |
| Flight — Apache Arrow Flight | RPC transport for Arrow-oriented data transfer | Custom wire-protocol/client work and text-oriented result movement | Define query/client API, access controls, supported types and measured transfer costs |
| DataFusion — Apache DataFusion | SQL parser, planner and execution foundation | Rebuilding a complete query engine and repeating financial analysis logic in clients | Define financial semantics, extensions, live-data access and hot/cold query behavior |
| Arrow — Apache Arrow | Standard columnar in-memory representation | Custom layouts and repeated representation changes between execution and research tools | Define event schemas, ownership, publication, retention and memory bounds |
| Parquet — Apache Parquet | Compressed columnar historical format | Proprietary file formats, historical storage and data lock-in | Define commit/catalog behavior, recovery, corrections, compaction and retention |

Source: pp. 2, 7, 11–13; RES-002/013/018/019. The four components provide reusable primitives. Their composition into a correct, low-latency financial TSDB is the proposed ChronosHFT engineering responsibility.

## 5. Challenge-to-solution matrix

The “FDAP contribution” column identifies foundation support; it does not claim that selecting the stack automatically solves the entire challenge.

| ID / challenge | Customer impact | FDAP contribution | ChronosHFT work and limit | Source / Phase 2 mapping |
| --- | --- | --- | --- | --- |
| MKT-01 Foundational engineering cost | Team effort goes into formats, query machinery and networking before useful domain features | Reuse Arrow, DataFusion, Parquet and Flight | Integration, lifecycle, security, recovery and testing remain necessary; no validated development-time saving | pp. 1–2, 11–13; RES-001/002; F01 |
| MKT-02 Python data-movement friction | Large results incur conversion and loading work before research begins | Arrow-compatible representations and Flight transfer reduce reliance on text/row conversion | Type compatibility, transport encoding and dataframe materialization must be measured; absolute network zero-copy is not established | pp. 6–7, 10, 20; RES-013/014; F11, CAP-07 |
| MKT-03 Asynchronous trades and quotes | Researchers repeatedly implement temporal alignment | DataFusion supplies an extensible query foundation | ASOF direction, instrument keys, ties, tolerance and unmatched cases require defined semantics and tests | pp. 6, 16, 28; RES-010; F05, CAP-03 |
| MKT-04 Repeated financial aggregation logic | OHLCV and time/tick-window code varies across clients | Columnar execution and query extensions support shared primitives | Bucket alignment, trading calendars, empty bars and OHLCV rules remain domain work; TICK_WINDOW scope pending | pp. 6, 17, 28; RES-011; F06 |
| MKT-05 Late data and changing historical knowledge | Research may use corrected information unavailable at the original decision time | Arrow/Parquet can carry event and version metadata; DataFusion can execute selected views | Bitemporal identity, knowledge boundaries, correction rules and historical visibility must be built; Parquet alone is not an audit system | pp. 6–7, 17, 28; RES-012/030/049; F09/F10, CAP-06 |
| MKT-06 Recent/historical fragmentation | Users query separate stores or combine results manually | DataFusion can execute over Arrow and Parquet representations | Hybrid catalog/snapshot rules must exclude gaps and double counting during persistence; columnar compatibility does not eliminate Parquet decoding | pp. 9, 12, 16, 27; RES-019/047; F08, CAP-05 |
| MKT-07 Finite memory and persistent history | Continuous capture exceeds RAM; frequent disk work can interfere with live workloads | Arrow recent data and background Parquet persistence support tier separation | Commit, eviction, disk-full, retention and recovery policies required; separate threads still share hardware resources | pp. 8–9, 16, 27; RES-016/018/026; F07, CAP-04 |
| MKT-08 Microburst capture and concurrent readers | Burst traffic and slow consumers can cause delay, overload or loss | Arrow provides batch representation; DataFusion consumes readable batches | Feed decoding, bounded queues, safe ownership/reclamation and overload accounting require custom integration; Disruptor is supplementary, not FDAP | pp. 8, 11–13, 26–42; RES-017/025/053; F02/F03, N06 |
| MKT-09 Latency jitter | Tail latency can interrupt timely analysis | Rust-based integration and controlled memory behavior support the proposed approach | Rust/no GC does not guarantee deterministic latency; allocation, OS scheduling, I/O, caches and query contention require evidence | pp. 10, 17, 21–23, 26; RES-020/029/038/044; N02/N03 |
| MKT-10 Proprietary data and operational control | Customers need control of deployment and stored assets | Open data formats and embeddable execution support self-hosted designs | Offline packaging, authentication, encryption and feed entitlements remain separate requirements; formats do not provide security by themselves | pp. 7, 15, 17; RES-015/023; N04/N05/N09 |
| MKT-11 Order-book reconstruction | Raw add/modify/delete streams require stateful interpretation | DataFusion extension points support exposing domain-derived views | Venue state machine, sequence recovery and book consistency must be designed; candidate extension, not established core MVP | p. 10; RES-021; F13, CAP-10 |

PRD shorthand F01 means PRD-F01; N02 means PRD-N02. Capability IDs refer to the Product Architecture document. Acceptance of these mappings does not close associated Phase 0 gaps.

## 6. How the proposed solution works in a user workflow

A researcher wants to align trades with the latest preceding quote, produce bars and compare current history with what was known before a correction.

1. **Capture:** a supported feed adapter decodes and maps events into a declared schema. Arrow supplies representation; adapter correctness and ordering are ChronosHFT responsibilities.
2. **Recent-data publication:** completed readable data becomes available to query execution. The memory/reader protocol must preserve consistency while capture continues.
3. **Persistence:** background processing writes historical Parquet data and publishes the approved catalog transition. Visibility and durability are separate states.
4. **Query:** DataFusion executes the approved financial semantics across the eligible recent and historical versions. It must not double count data present in both tiers.
5. **Research access:** Flight delivers typed query results to the supported Python client; dataframe conversion and end-to-end latency are measured.
6. **Correction-aware analysis:** explicit event/knowledge metadata selects the requested historical view; neither Arrow nor Parquet supplies this policy automatically.

Source direction: pp. 6–13, 15–17, 27–28. This is a conceptual product workflow, not a claim that an implementation exists. It makes the benefit concrete: fewer separate data representations and less repeated client-side domain logic, provided the integration is correct.

## 7. Build versus reuse

The baseline explicitly separates reusable primitives from the missing active-memory, hybrid-query and ingestion integration (pp. 11–13).

| Reuse as foundation | Design/integrate for ChronosHFT | Validate before product promises |
| --- | --- | --- |
| Arrow layouts and arrays | Financial schemas and safe retained-reader lifecycle | Precision, bounded memory, ownership and concurrent correctness |
| DataFusion query machinery | Financial extensions and unified catalog access | Semantic oracle agreement and qualified tail latency |
| Parquet readers/writers | Persistence/catalog transitions and correction history | Failure recovery, tier continuity and retention behavior |
| Flight transport | SQL/Python client workflow and compatible types | End-to-end throughput, copies/conversions and access controls |
| Optional queue libraries | Feed scheduling and batch coordination | Saturation, loss accounting and lifecycle correctness |

“Must build” here means take responsibility for the missing product contract and integration. It does not require rewriting every mechanism if a suitable reusable implementation is later validated. The PDF's assertion that no ready-made OSS product supplies the complete architecture is not treated as an exhaustive current-market survey. Proprietary-layer wording also does not resolve the open-core license boundary (DEC-012).

## 8. Competitive context and fair comparison

The PDF discusses kdb+, custom in-house systems, ClickHouse, DuckDB and PostgreSQL/CSV research workflows. The following records the questions that motivate comparison without endorsing categorical criticisms.

| Alternative discussed | Baseline concern | Relevant comparison to perform |
| --- | --- | --- |
| kdb+ and specialized incumbent infrastructure | Entrenched code and specialized skills can increase switching cost | Migration effort, compatible analytics, operator burden and total workflow value |
| Bespoke tick-capture systems | Teams repeatedly maintain custom capture/storage plumbing | Recovery correctness, adapter coverage, maintenance effort and deployment cost |
| ClickHouse | Background maintenance and result transfer may affect the proposed latency/workflow target | Matched concurrent ingestion/query workload, maintenance state, resource limits and actual client transfer path |
| DuckDB | Embedded/local research model may differ from shared continuous-capture needs | Explicit process/deployment architecture and concurrent workload, including supported feature scope |
| PostgreSQL/CSV workflows | Representation conversion and loading may slow notebook workflows | Identical data, result types and query semantics; measure total query-to-dataframe time |

Source: pp. 6–10, 14, 17–20; RES-022/032/033. The claims that ClickHouse necessarily produces severe merge jitter, DuckDB necessarily blocks the described ingestion scenario, or alternative clients necessarily require slow text translation remain unverified. They cannot establish general product inferiority. Existing products' current capabilities would need an independently sourced refresh before publishing a competitive feature matrix.

ChronosHFT's proposed differentiation is the combined financial workflow: concurrent capture and recent-data analysis, unified history, declared financial semantics, correction-aware research and SQL/Python access. Each part needs evidence; FDAP selection alone does not establish superiority.

## 9. Adoption and commercial challenges

| Challenge | Baseline proposal | Evidence still needed |
| --- | --- | --- |
| Crowded database market | Specialize in financial developer experience | Representative user interviews and successful workflows |
| Incumbent inertia | Offer familiar SQL/Python entry points | Migration path, compatibility limits and switching-cost study |
| Private/proprietary data | Self-hosted/private VPC/co-location deployment | Offline installation, access-control and operational evidence |
| Build-versus-buy decision | Save infrastructure engineering effort | Measured maintenance/setup burden and pilot value; no automatic 12-month saving |
| Community adoption | Open-source single-node core and educational notebook examples | License decision, reproducible examples and user feedback |
| Sustainable commercial offering | Fleet management, active-passive HA and support; resource/throughput pricing | Separate feature contracts, reliability evidence and validated pricing assumptions |

Source: pp. 7–10, 17–18; RES-022/023/031. Paid features are future proposals, not core MVP requirements. Unlimited developer seats and resource-band licensing are hypotheses, not finalized terms. The “20× faster notebook loading” example is an unverified marketing target and must not be published as a measured result.

## 10. Claims that require explicit qualification

| Source claim | Safe treatment in future documentation |
| --- | --- |
| Absolute zero-copy / no network serialization | Describe Arrow-oriented interoperability; specify and measure actual encoding, transfer and conversion costs |
| Rust ownership ensures fully deterministic latency | State no tracing-GC objective; verify bounded memory and latency under the approved workload |
| Background isolation means no contention | Identify shared CPU/memory/network/disk resources and measure interference |
| Parquet metadata provides compliance/audit storage | Define integrity, versioning, access, retention and audit requirements separately |
| Nanoseconds ensure correct event sequence | Preserve precision; separately define sequence, clock and tie-breaking contracts |
| Ring buffers or Disruptor guarantee reader safety | Require explicit ownership, retention and query-completion/reclamation protocol |
| Off-the-shelf foundation implies production readiness | Retain security, operations, recovery, correctness and validation gates |
| Performance and market superiority are established | Treat figures and comparisons as hypotheses until workload-matched evidence exists |

These qualifications follow the Phase 0 assessment; they are not changes to original research history. Source prototypes and targets remain traceable.

## 11. Validation agenda and documentation placement

Before using this document as externally verified market collateral, gather:

- Customer evidence for capture/research friction and deployment constraints.
- A dated, primary-source competitor capability review with pinned versions/configurations.
- Reproducible capture, concurrent-query, cross-tier and query-to-dataframe benchmarks.
- Financial and correction-aware correctness fixtures, plus failure/recovery evidence.
- Reviewed licensing, commercial boundaries and adoption/migration assumptions.

These are proposed validation actions, not invented requirements or completed research. Phase 0 GAP-013/019/020 and DEC-001/002/005/012/013 remain relevant.

This page appears before the PRD in the Product navigation as **Market landscape & FDAP**. The reasoning sequence is market context → challenges → FDAP contribution → ChronosHFT responsibilities → product requirements. Preserve the existing 16-document register; this supporting document uses CHRONOS-P2-MARKET-001 rather than renumbering approved identifiers. Repository integration preserves the existing MkDocs navigation and document identifiers.

## 12. Source coverage and change record

| Source pages | Coverage in this document | Phase 0 links |
| --- | --- | --- |
| 1–3 | Market shifts, reusable primitives and differentiation | RES-001–005 |
| 3–5 | Earlier edge-market exploration and its scope separation | RES-006–008 |
| 6–7 | Financial workflow challenges and FDAP mapping | RES-009–015 |
| 8–10 | Hot/cold needs, jitter, microbursts and deployment | RES-016–023 |
| 11–13 | Reuse/build boundaries and single-node proposal | RES-002/017/019/024/025 |
| 14–18 | Product vision, personas and commercial hypotheses | RES-023–032 |
| 18–20 | Competitive claims retained as unverified | RES-033 |
| 21–28 | Performance/domain qualifications and staged workflow | RES-034–049 |
| 29–42 | Lifecycle/prototype limits relevant to claimed benefits | RES-050–062 |

| Version | Date | Change |
| --- | --- | --- |
| 1.0 | 2026-10-13 | Added research-grounded market context, 11 challenge mappings, FDAP responsibilities, competitive qualifications and validation agenda |
