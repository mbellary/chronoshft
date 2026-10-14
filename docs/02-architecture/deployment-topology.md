# Deployment topology

!!! info "Source reference profile · 2026-10-14"
    Extracted from [unified HFT research notes](../assets/research/HFT-Research-Notes.pdf), pp. 17 and 21–23. Hardware and performance values are source proposals, not verified procurement requirements or measured results.

## Deployment model

The baseline is a customer-controlled single-node Rust binary on private/VPC or co-located bare-metal infrastructure, without runtime Internet dependencies. The MVP performance description uses an 8-core node; the later EPYC reference is a distinct profile. Enterprise fleet management and active/passive HA are market extensions, not a defined distributed runtime (pp. 17–18; DEC-001/002/010).

## Reference hardware

| Component | PDF proposal (p. 21) | Qualification from existing assessment |
| --- | --- | --- |
| CPU | AMD EPYC 9354; 32 cores / 64 threads; 3.25 GHz base / 3.8 GHz boost; 256 MB L3 | Single socket alone does not prove uniform locality or bounded jitter |
| Memory | 384 GB DDR5-4800, 6 × 64 GB | Source calls this fully populated; assessment records twelve memory channels, so six DIMMs are not full channel population (VAL-008) |
| OS/log storage | 960 GB enterprise PCIe Gen4 NVMe | Logging/system activity still needs an interference budget |
| Cold storage | 2 × 3.84 TB enterprise NVMe, software striped | Striping gives no redundancy or durable-commit guarantee |
| Network | ConnectX-6 Dx dual-port 25/100GbE with DPDK/kernel bypass | Adapter-specific transport, framing/TLS and allocation design is unresolved (VAL-025) |

These are extracted specifications, not a current product recommendation. Existing dated hardware evidence is linked in the [research evidence register](../00-governance/research-baseline.md#evidence-register-and-maintenance).

## Proposed core allocation in the source

| Physical cores | Source responsibility (pp. 21–22) | Review obligation |
| --- | --- | --- |
| 0–1 | Linux, logs and background micro-Parquet flushing | Measure competition between OS work and persistence; reserve adequate capacity |
| 2–5 | NIC ingress, decoding and ring writes | Specify writer partitions, IRQ routing and exclusive ownership |
| 6–31 | DataFusion workers and Arrow Flight | Bound query/result resource use and measure shared cache/memory contention |

The PDF proposes boot-level isolation and affinity. Record BIOS/NUMA policy, SMT, CPU affinity, IRQ placement, power/frequency settings and memory allocation locality before claiming isolation. Dedicated core groups still share memory bandwidth and other resources (VAL-009; DEC-011).

## Workload profiles

| Profile | Source target | Boundary |
| --- | --- | --- |
| MVP (p. 17) | 1M ticks/s; hot query P99 < 1 ms on 8 cores | Separate baseline from EPYC reference |
| EPYC sustained (pp. 22–23) | 2.5M ticks/s at average 48-byte tick payload | Specify resident/wire bytes, duration, losses and persistence |
| EPYC burst (p. 23) | 8M ticks/s for up to 500 ms | Size excess backlog against service rate and headroom |
| EPYC hot queries (p. 23) | P95 < 180 µs; P99 < 850 µs for time-bucket/VWAP under load | Requires exact queries, windows, concurrency and tail measurement |
| EPYC egress (p. 23) | About 4.2 GB/s over loopback/100GbE wording | Separate local and remote tests and client conversion cost |

No entry is an observed benchmark. Stage 2 hybrid < 50 ms and Stage 3 50M remote rows < 1.5 seconds are separate roadmap gates, not additional guarantees of this hardware (pp. 27–28; DEC-005).

## Capacity implications

The [existing feasibility assessment](../00-governance/performance-profiles.md#quantitative-feasibility-checks-reviewer-derived) derives 120 MB/s and 10.368 TB/day of raw payload at 2.5M × 48 bytes. With service held at 2.5M/s, an 8M/s half-second burst adds about 132 MB of excess payload backlog; actual resident memory requires metadata, schema and outstanding-reader headroom. Five minutes of raw retained payload at that sustained rate is 36 GB, distinct from the ingestion queue.

The two cold drives provide 7.68 TB nominal striped capacity, about 17.8 hours of that raw payload before compression, reserves and filesystem overhead. Compression ratio and sustainable writing remain unverified. Select retention, compaction and recovery together; cold-path slowdown must connect to the full-ring policy rather than silently exhausting memory.

## Deployment review

Close DEC-011 with an adapter-specific sockets/bypass comparison and recorded locality/interference measurements. Close DEC-005 with reproducible ingestion, query, persistence and egress workloads under simultaneous load. Link durability and restart evidence to DEC-008. Related: [platform design](platform-design.md), [active memory](active-memory.md), [performance profiles](../00-governance/performance-profiles.md), [hardware setup](../06-benchmarking/hardware-setup.md).
