# Hardware setup

!!! info "Extracted reference and reproducibility plan · 2026-10-14"
    The [HFT research notes](../assets/research/HFT-Research-Notes.pdf), physical pp. 17 and 21–23, describe separate 8-core MVP and EPYC reference profiles. Specifications below are source proposals, not procurement recommendations or measured deployment evidence (DEC-011).

## Source reference environment

| Component | PDF reference | Validation qualification |
| --- | --- | --- |
| CPU | EPYC 9354, 32 cores/64 threads, 3.25 GHz base/3.8 GHz boost, 256 MB L3 | Record actual topology and frequency behavior; a single socket does not prove uniform locality |
| RAM | 384 GB DDR5-4800, 6 × 64 GB | Existing assessment notes twelve memory channels; the source's full-population description is unsupported (VAL-008) |
| OS/log disk | 960 GB enterprise PCIe Gen4 NVMe | Record log/system interference |
| Cold storage | 2 × 3.84 TB enterprise NVMe, software striped | No redundancy or durable-commit guarantee follows from striping |
| NIC | ConnectX-6 Dx dual-port 25/100GbE, DPDK/kernel bypass | Pin driver/firmware and adapter-specific transport; bypass is not a mandatory MVP contract |

Source core groups (pp. 21–22) are 0–1 for OS/logging/flushing, 2–5 for ingress/decode/writes, and 6–31 for DataFusion/Flight. Treat these as physical-core proposals; map actual logical CPUs and SMT siblings before setting affinity. Shared memory/cache and OS/flusher contention still require measurement (VAL-009/025).

## Required run manifest

| Area | Record for every reproducible run |
| --- | --- |
| Identity/build | Run ID/date, repository commit/dirty state, Rust/compiler/build flags, dependency lockfile, DataFusion/Arrow/Parquet/Flight/ring and Python client versions |
| CPU/locality | Model, sockets, cores/SMT, NUMA nodes, BIOS/microcode, affinity for writer/query/flusher/client, allocation locality, IRQ routing |
| Power/OS | OS/kernel, governor/turbo/C-state policy, isolation settings, scheduler, background services and temperature/throttling observations |
| Memory | DIMM/channel population, capacity/speed, huge-page/locking/pre-fault policy if used, swap and peak RSS/page faults |
| Storage | Model/firmware, filesystem/mounts, striping configuration, free space, codec/row-group settings, sync/commit mode and cache state |
| Network/client | NIC/driver/firmware, negotiated link, MTU, sockets/bypass mode, protocol/TLS/framing, generator and client placement, remote topology |
| Engine | Writer partitions, worker counts, ring/batch sizes, publish/flush/retention policy, overload and durability mode |
| Measurement | Timer/clock calibration, histogram resolution, profiler versions/options and instrumentation overhead |

Retain machine-readable settings and relevant topology/configuration output with the [evidence bundle](evidence-and-traceability.md). Do not infer that source affinity numbers match the test host. Any tuned run needs a comparable baseline with changes disclosed.

## Isolation and profiling experiments

Compare colocated and separated ingestion/query/persistence work, local and remote NUMA allocation, sockets and approved bypass adapters, and busy-spin versus other applicable wait strategies. Measure CPU consumption as well as latency. These are proposed experiments; no kernel or BIOS changes are performed by this documentation update.

Use allocator counters for the complete append-to-reclaim lifecycle, CPU/cache profiling such as applicable `perf` counters, and I/O/network telemetry. Report instrumented safety/resource runs separately from minimally instrumented latency runs when overhead is material. Never equate lack of tracing GC with zero allocator or paging latency (VAL-005/006).

## Capacity planning

At 2.5M × 48 bytes, logical payload is 120 MB/s or 10.368 TB/day; five-minute retention is 36 GB before overhead. The nominal 7.68 TB striped disks hold about 17.8 hours of that raw payload before reserves, filesystem overhead or compression. Include query-retained batches, double buffers, catalog, persistence backlog and client output in memory sizing. Compression and sustainable disk throughput remain unverified.

Sources: [Deployment topology](../02-architecture/deployment-topology.md), [performance feasibility](../00-governance/performance-profiles.md), [active memory](../02-architecture/active-memory.md) and [product architecture](../01-product/product-architecture.md).

## Physical reference profile and publication gate

Reference profile ID **HW-EPYC-9354-CX6DX-01** denotes the proposed single-node bare-metal environment: one AMD EPYC 9354 (32 physical cores/64 threads; source-listed 3.25 GHz base, 3.8 GHz boost, 256 MB L3), 384 GB DDR5-4800 in six 64 GB DIMMs, one 960 GB enterprise Gen4 NVMe system/log device, two 3.84 TB enterprise NVMe cold-storage devices in a software stripe, and a Mellanox/NVIDIA ConnectX-6 Dx dual-port NIC in DPDK kernel-bypass mode. Pin the actual active port/link speed for each run; the source's 25/100GbE description does not establish an exact board SKU. Use source core groups 0–1 / 2–5 / 6–31 only after mapping physical cores to actual logical CPU IDs.

This is the exact declared reference configuration available from the research, **not a captured physical machine manifest**. Motherboard, DIMM/device SKUs, firmware, kernel, DPDK/PMD versions, active link speed, BIOS and actual affinity remain unrecorded. A socket-based run is a separate profile and cannot be labeled kernel bypass. DEC-011 remains open until these values and the observed topology are captured.

Every future performance graph/table must link a run ID and immutable `hardware-manifest.json` alongside raw results. The manifest must contain the profile ID plus every field in [Required run manifest](#required-run-manifest), actual device identities, physical/logical CPU mapping and transport configuration. Hash the manifest and store that hash in `summary.json`; the caption links both. Missing fields, placeholders or absent manifests block publication as measured evidence. Runs on other hardware receive a distinct profile ID with documented differences; they cannot inherit the EPYC profile label. Historical research targets remain labeled source proposals and are exempt from being represented as new measurements.
