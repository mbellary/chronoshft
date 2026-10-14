# Operational readiness and traceability

!!! info "Research-derived operations draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages) and the Product, Architecture and Engineering contracts linked below. Source proposals and derived operational procedures remain distinct. No implemented control, approved SLA, recovery guarantee or measured performance is asserted.


## Authority and status

The Operations tab translates research intent and existing proposed contracts into reviewable operating procedures. All four original placeholders are populated; capacity/overload and this evidence handoff are added. Documentation completion does not establish operational or release readiness. No DEC/GAP/VAL record is closed, and named owners, approvers, review dates and numerical service objectives remain pending.

## Source-to-operations map

| Physical PDF pages / source records | Operations section | Product / Architecture / Engineering trace |
| --- | --- | --- |
| 15–18, 26–28; RES-015/024/031 | Deployment boundary, offline packaging, commercial HA distinction | PRD-F01/N04/N05; deployment topology/platform design; ENG-001/015; DEC-002/010/012 |
| 21–23; RES-034–039 | Hardware qualification, capacity and performance monitoring | PRD-N01–N03; deployment topology; ENG-006/014; DEC-005/011; VAL-008/009/025 |
| 9, 16, 27; RES-018/019 | Persistence visibility, restart and disk failure | PRD-F07/F08/N07/N08; query/storage; ENG-010/011; DEC-008; VAL-023/024; CON-003 |
| 17, 27–28; RES-030/046/049 | Late/corrected history, compaction and restore checks | PRD-F09/F10; catalog/version contracts; ENG-012; DEC-009 |
| 29–42; RES-051–062 | Bounded intake, publication, reader retention and shutdown | PRD-F03/N03/N06/N08; active memory/component contracts; ENG-003–006; DEC-003/004/006; CON-007 |
| 6, 17, 27–28; RES-013/048 | Client access, slow streams and cancellation | PRD-F11/N05; client boundary; ENG-013/015; DEC-013; VAL-004 |
| 24–25; RES-040–042 | Clock/type/sequence diagnostics and recovery checks | PRD-F12; financial model; ENG-007; DEC-007/014; VAL-010/011/018 |

Security mechanisms, telemetry backends, alerts, upgrade/rollback and backup procedures are derived operational proposals where the PDF does not supply an implementation. Existing [Product requirements](../01-product/product-requirements.md), [Architecture contracts](../02-architecture/component-contracts.md) and [Engineering requirements](../03-engineering/engineering-requirements.md) provide the traceable acceptance intent.

## Proposed evidence gates

| Gate | Concrete evidence required | Proposed reviewing role / open dependency |
| --- | --- | --- |
| Deployable/offline package | Versioned inventory, validated config, supported fixture startup/SQL/client workflow without external services | Engineering + Operations; ENG-001/015; DEC-002/010 |
| Bounded resources | Full/wrapped queue, stalled reader/flusher/client, burst/drain and memory accounting | Systems + Operations; ENG-003–006; DEC-003/006 |
| Visibility and telemetry | Separate admission/publication/commit states, gap/loss reconciliation, low-volume timers and instrumentation overhead | Engineering + Operations; ENG-002/004/014; DEC-004 |
| Recovery/restore | Selected acknowledgment/failure model, crash/disk/compaction tests and verified restoration coverage | Storage + Operations; ENG-010–012; DEC-008/009 |
| Access protection | Threat boundary, concrete permissions/transport/secrets, unauthorized and exhaustion tests | Security + Client Engineering; ENG-013/015; GAP-015/018 |
| Performance qualification | Separate reproducible MVP/reference/burst/hybrid/Flight runs under simultaneous load | Performance + Operations; ENG-014; DEC-005/011 |
| Upgrade/shutdown | Partial-batch/drain/loss report, data/catalog compatibility and rehearsed rollback | Engineering + Operations; DEC-003/008/013 |

These are proposed release review inputs, not adopted new requirement IDs. Set service objectives and accountable owners through recorded review; retain test artifacts and approvals in the existing [decision register](../00-governance/decision-register.md) and [Engineering validation](../03-engineering/validation-and-traceability.md).

## Documentation integration

Validate navigation, relative links and rendering using the repository's strict build:

```bash
uv run --locked --extra docs mkdocs build --strict
```

A successful documentation build verifies integration only. Runtime monitoring, security, recovery, implementation performance and publication of the site require their own evidence.
