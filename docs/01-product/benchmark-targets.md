# Product benchmark targets

| Control | Value |
| --- | --- |
| Document | CHRONOS-P2-BENCH-001 |
| Version / date | 1.0 / 2026-10-14 |
| Status | User-proposed engineering targets; validation and product approval pending |
| Deployment profile | Single-node, bare-metal financial TSDB |
| Companions | [Product requirements](product-requirements.md), [Scope & MVP](scope-and-mvp.md), [Performance profiles](../00-governance/performance-profiles.md), [Benchmark methodology](../06-benchmarking/benchmark-methodology.md) |

ChronosHFT needs explicit, measurable performance objectives rather than a vague promise of being “fast.” The following profile captures the supplied targets for institutional quantitative capture and research workflows.

!!! info "Targets and evidence"
    These numbers are proposed acceptance targets, not measured ChronosHFT results or verified industry standards. They supplement the source-derived PRD profiles without silently replacing them. Hardware, workload, measurement boundaries and approval must be recorded before adopting a release gate. No audited database benchmark establishing this complete profile was supplied.

## 1. Ingestion throughput: the microburst benchmark

Market feeds can spike when matching engines resume or macroeconomic announcements arrive. Test sustained capture separately from short bursts, including concurrent queries and background persistence.

| Metric | Proposed profile |
| --- | --- |
| Sustained ingestion | 1,000,000–2,000,000 ticks/sec per node |
| Microburst capacity | 8,000,000 ticks/sec for windows up to 1,000 ms |
| Peak data loss | Zero dropped eligible events during the defined burst test |
| Logical payload | 48–64 bytes per tick |
| Illustrative fields | `Timestamp_ns`, `Symbol_ID`, `Bid_Price`, `Ask_Price`, `Bid_Size`, `Ask_Size` |
| Peak logical payload rate | 384–512 MB/sec, using decimal MB |

The supplied design proposes a preallocated, lock-free Arrow ring buffer. This is an engineering candidate, not evidence of loss-free capture or safe reader ownership. Measure feed sequence gaps, decoder errors, queue saturation, rejected events and accepted/visible/durable counts separately. Packet loss and event loss require distinct accounting.

At 64 bytes per tick, an eight-million-tick burst represents 512 MB of logical payload in one second. Actual memory and network traffic may be higher because of metadata, framing, alignment, validity buffers and copies. Buffer capacity depends on the backlog after downstream processing, and must include recovery between repeated bursts.

Related requirements: PRD-F02/F03, PRD-N01/N06.

## 2. Tail latency: the jitter metrics

Measure simple lookups and single-instrument VWAP slices under continuous ingestion. A low average cannot replace tail measurements.

| Percentile | Proposed gate | Load and scope |
| --- | --- | --- |
| P50 | < 50 microseconds | Simple lookup or bounded single-instrument VWAP slice |
| P99 | < 150 microseconds | Same workload under continuous 1M ticks/sec ingestion |
| P99.9 | < 850 microseconds | Same workload; include background persistence and maintenance |

Pin query cardinality, time-window size, concurrency, data residency and cache state. Report engine execution time and client-observed latency separately; declare which boundary each gate uses. Use sufficient samples for P99.9 and report maxima, timeouts and errors as well as percentiles.

The supplied comparison alleges 50–500 ms tail spikes in ClickHouse and general TSDBs. That range is an **unverified comparison claim**, not an industry baseline. Compare pinned versions on identical hardware, data, queries and maintenance conditions before publishing competitive conclusions. Sub-millisecond P99.9 does not by itself establish deterministic latency or a zero-allocation architecture; allocation behavior needs separate instrumentation.

Related requirements: PRD-F03/F04, PRD-N02/N03.

## 3. Quantitative research egress: the dataframe metric

Measure the complete SQL-to-dataframe workflow, including result transfer and materialization into the supported Python client.

| Metric | Proposed target or comparison status |
| --- | --- |
| Arrow Flight dataframe load | < 1.5 seconds for 50,000,000 rows |
| Deployment boundary | Local loopback; remote-network results reported separately |
| Supplied delivery-rate aspiration | Approximately 3.5–4.2 GB/sec, conditional on output width and actual bytes |
| Supplied legacy ODBC/JDBC comparison | 35–90 seconds, with 45 seconds in the supplied summary; unverified and workload dependent |

At 48–64 logical bytes per row, 50M rows contain 2.4–3.2 GB; completion in 1.5 seconds implies more than 1.6–2.13 GB/sec of logical payload delivery. The separate 3.5–4.2 GB/sec aspiration cannot be inferred from row count alone. Record schema, result byte size, compression, query time, transfer time, dataframe materialization time, memory use and client versions.

Arrow Flight supports Arrow-oriented transport. It does not guarantee zero serialization, zero copying or direct placement into arbitrary Polars/Pandas memory layouts. Measure actual encoding and conversion costs, and verify row counts and type fidelity.

Related requirement: PRD-F11; retain the PRD's remote 50M-row scenario as a separate profile.

## 4. Storage and space efficiency: the cost matrix

| Metric | Proposed profile |
| --- | --- |
| Reference dataset | 1 billion ticks at 64 logical bytes each |
| Raw logical size | 64 GB, using decimal GB |
| Parquet footprint target | < 8.5 GB per billion ticks |
| Compression aspiration | Approximately 8×–10× relative to raw logical size |

The < 8.5 GB gate implies a ratio greater than approximately 7.53×. An 8×–10× ratio corresponds to 8–6.4 GB, so report the file-size gate and compression aspiration separately.

Evaluate dictionary encoding, bit packing and compatible Parquet codecs against representative financial data. The supplied Gorilla/Chimp proposal requires a separate format/interoperability decision; it is not an established standard Parquet encoding requirement. Preserve price precision, timestamps and correction metadata, and verify round-trip equality. Count all files and metadata within the declared storage boundary, and report dataset distribution, row-group sizes, codec, write throughput and historical scan performance.

Related requirements: PRD-F07/F08/F09/F10/F12.

## 5. MVP benchmark checklist

These are proposed benchmark gates. Egress and storage depend on capabilities beyond the recent-data Slice A; this checklist does not redefine Slice A as the complete MVP.

| Metric category | ChronosHFT target | Evidence required |
| --- | --- | --- |
| Sustained ingestion | 1–2M ticks/sec per node | Defined duration, feed/schema and concurrent workload |
| Maximum ingest burst | 8M ticks/sec for up to 1,000 ms | Repeated bursts, bounded backlog and recovery evidence |
| P50 / P99 / P99.9 query latency | < 50 / < 150 / < 850 microseconds | Declared query scope, timing boundary and percentile samples |
| 50M-row dataframe load | < 1.5 seconds over local loopback | Actual output bytes, end-to-end timing and type fidelity |
| Peak event drop rate | 0% within the tested profile | Sequence/count reconciliation and overload accounting |
| Billion-tick Parquet footprint | < 8.5 GB; 8×–10× aspiration | Representative dataset, complete size accounting and lossless round trip |

The supplied 1.5M ticks/sec “maximum per thread,” 50–500 ms competitor latency and 45-second dataframe load figures are comparison hypotheses. Per-thread and per-node throughput are different measurement scopes; no universal competitor threshold is adopted here.

Before release acceptance, record the CPU, RAM, NUMA placement, NVMe, NIC, OS, versions, power settings, durability mode, dataset, warmup, trial count, concurrency and background tasks. Store reproducible scripts and raw results alongside [workload definitions](../06-benchmarking/workload-definitions.md) and [acceptance criteria](../06-benchmarking/acceptance-criteria.md). All gates remain untested.

## 6. Supplied references and their limits

The supplied sources are retained for investigation. Citation inclusion does not verify every adjacent numerical claim.

- [NVIDIA: single-digit microsecond capital-markets inference](https://developer.nvidia.com/blog/achieving-single-digit-microsecond-latency-inference-for-capital-markets/) reports STAC-ML LSTM inference measurements. These are not TSDB ingestion, SQL or dataframe benchmarks.
- [KX: benchmarking specialized databases for high-frequency data](https://kx.com/blog/benchmarking-specialized-databases-for-high-frequency-data/) provides specialized-database benchmark context; it does not establish this entire ChronosHFT profile.
- [AWS: optimizing tick-to-trade latency, part 2](https://aws.amazon.com/blogs/web3/optimize-tick-to-trade-latency-for-digital-assets-exchanges-and-trading-platforms-on-aws-part-2/) concerns trading infrastructure latency; applicability requires matched measurement boundaries.
- [Databricks community: database discussion](https://answers.databricks.com/best-database-high-frequency-trading-real-time-bidding)
- [HFTPerformance framework article](https://medium.com/@gwrx2005/hftperformance-an-open-source-framework-for-high-frequency-trading-system-benchmarking-and-803031fe7157)
- [Quant trading system architecture](https://mbrenndoerfer.com/writing/quant-trading-system-architecture-infrastructure)
- [ClickHouse issue #88753](https://github.com/ClickHouse/ClickHouse/issues/88753)
- [EPAM: quant trading](https://solutionshub.epam.com/blog/post/quant-trading)

Community discussions, issue reports and secondary articles require corroboration before use as audited performance evidence. The supplied closing question about an L2 column layout is outside this benchmark page; the illustrative payload above does not establish a complete L2 event schema.
