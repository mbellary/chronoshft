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
