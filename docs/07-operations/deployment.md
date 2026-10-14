# Deployment

!!! info "Research-derived operations draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages) and the Product, Architecture and Engineering contracts linked below. Source proposals and derived operational procedures remain distinct. No implemented control, approved SLA, recovery guarantee or measured performance is asserted.


## Operating boundary and source intent

The research proposes a customer-controlled single-node Rust binary with embedded DataFusion, local Parquet history and Flight clients (pp. 15–17, 26–28; RES-024/027). Private VPC, on-premises and co-located deployment without runtime Internet dependencies is source intent (p. 17; RES-015). Live external venue feeds still require their network connectivity and entitlements. Fleet management, active/passive HA and enterprise SLAs are later commercial proposals (pp. 17–18), not an available failover contract.

Use [Product scope](../01-product/scope-and-mvp.md) and DEC-001/002/010 to establish the release boundary. Do not infer execution, distributed storage or a complete FIX adapter from the research examples.

## Environment qualification

| Area | Record before deployment | Evidence boundary |
| --- | --- | --- |
| Software | Binary/build identity, compatible pinned Rust/Arrow/DataFusion/Parquet/Flight and client versions; package checksums and offline inventory | ENG-001/013/015; no published installation/API command is established |
| Hardware | CPU, RAM, NUMA topology, disks/filesystem, NIC/driver and adapter transport | MVP eight-core profile and EPYC reference are separate profiles |
| CPU placement | BIOS/SMT, power/frequency policy, affinity, IRQ routing and memory locality | Core isolation does not remove shared cache or memory-bandwidth contention |
| Storage | Data/log paths, permissions, capacity reserves, commit mode and retention | Striped NVMe has no redundancy guarantee |
| Network | Feed/client/admin endpoints, allowed peers and transport protection | Private placement does not supply authentication |
| Clock/schema | Clock source/quality, epoch/timezone, sequence/session and schema version | Nanosecond representation does not prove clock accuracy |

The [deployment topology](../02-architecture/deployment-topology.md) preserves the PDF's EPYC 9354, 384 GB RAM, separate OS disk, two striped cold disks and ConnectX-6 Dx proposal (pp. 21–23). Its six-DIMM “fully populated” claim is qualified by VAL-008. The proposed cores 0–1 for OS/flushing, 2–5 for ingress and 6–31 for query/Flight require locality/interference measurement (VAL-009; DEC-011). Select sockets or kernel bypass per adapter; WebSocket/TLS and ITCH paths cannot inherit one unverified DPDK design (VAL-025).

## Proposed configuration inventory

Document effective values and validation for adapter/venue/version, source sessions, schema, writer partitions, ring/batch capacities, publication row/timer thresholds, hot retention, persistence triggers, commit mode, reorder bounds, query/result/client limits and overload policy. Include storage paths, logging limits, access credentials and clock diagnostics. These are configuration concerns, not existing option names or defaults.

Resolve publication versus persistence independently: 500 µs/1,000-row and 4,096-row publication examples differ (CON-007); cold flush is 5 s or 500k ticks on p. 9 and 10 s or 500k ticks on pp. 16/27 (CON-003). Record selected semantics under DEC-004 rather than silently choosing one source value.

## Proposed installation and startup procedure

1. Verify the approved package inventory, checksums, version compatibility and configuration; ensure offline local fixtures can run.
2. Validate writable storage, capacity headroom, access boundaries, feed entitlement and clock/sequence assumptions.
3. Reconcile committed catalog/history and incomplete outputs according to the selected [recovery model](recovery.md) before accepting intake.
4. Initialize bounded writer/registry/query/persistence resources and check transport/adapter readiness.
5. Run a known fixture through admission, publication, SQL, declared persistence and the supported client; check row counts and type fidelity.
6. Enable live intake under the selected overload policy and confirm [observability](observability.md) reports the separate lifecycle states.

Process liveness alone is insufficient readiness. The readiness definition must state whether queries, intake and persistence are independently available and whether feed gaps or degraded history restrict use. No health endpoint is invented here.

## Shutdown, upgrade and rollback proposal

Stop new intake and queries as specified, resolve the writer-owned partial batch, drain bounded persistence/streams within a declared deadline and report accepted-but-uncommitted ranges. Keep retained arrays valid until readers release them (ENG-003–006/010/013). A forced stop must disclose the selected loss boundary.

Before upgrade, preserve configuration, catalog/schema versions and recovery evidence; verify new code can read the current data and metadata. Rollback requires backward compatibility with files/catalog changes made by the new version. Do not assume replacing the binary reverses a schema migration. Rehearse stop/restart and compatibility with fixtures before enabling production traffic.

## Dependencies and open gates

[Platform design](../02-architecture/platform-design.md), [component contracts](../02-architecture/component-contracts.md), [Engineering requirements](../03-engineering/engineering-requirements.md), [Flight integration](../03-engineering/flight-and-python.md) and [Product requirements](../01-product/product-requirements.md) supply the contracts. PRD-N04/N05; ENG-001/014/015; DEC-002/004/005/008/010/011/013 remain review inputs. See [operational readiness](readiness-and-traceability.md) for evidence gates.
