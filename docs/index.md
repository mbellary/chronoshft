---
hide:
  - toc
---

# ChronosHFT

<div class="chronos-hero" markdown>

<span class="chronos-eyebrow">RESEARCH · ARCHITECTURE · ENGINEERING</span>

## Financial data. Explicit contracts. Measured performance.

The documentation workspace for a proposed Rust financial time-series database and analytics engine, built around Apache Arrow Flight, DataFusion, Arrow and Parquet.

[Start with the research](00-governance/research-baseline.md){ .md-button .md-button--primary }
[Explore the architecture](02-architecture/system-architecture.md){ .md-button }

</div>

!!! info "Documentation foundation"
    Research & governance and Product contain substantive drafts. Other topic pages remain outlines; architectural proposals and performance targets are not approved specifications or verified results. The original research PDF still needs to replace its empty placeholder.

## Explore the documentation

<div class="grid cards" markdown>

-   **A · Research & governance**

    ---

    Explore the completed Phase 0 extraction, validation, open gaps, pending decisions and readiness gates.

    [Research baseline](00-governance/research-baseline.md) · [Assessment](00-governance/research-assessment.md) · [Traceability](00-governance/traceability.md)

-   **B · Product definition**

    ---

    Explore the market context, users, capabilities, proposed MVP and Phase 2 handoff.

    [Phase 2 overview](01-product/phase-2-definition.md) · [Market & FDAP](01-product/market-landscape-and-fdap.md) · [Requirements](01-product/product-requirements.md) · [Product architecture](01-product/product-architecture.md) · [Scope & MVP](01-product/scope-and-mvp.md)

-   **C · Architecture & engineering**

    ---

    Follow the runtime topology, component interfaces and subsystem contracts.

    [System architecture](02-architecture/system-architecture.md) · [Engineering design](03-engineering/engineering-design.md) · [Memory lifecycle](03-engineering/active-memory-lifecycle.md)

-   **D · Rust engineering knowledge**

    ---

    Connect ownership, concurrency and memory management to implementation problems.

    [Fundamentals](04-rust/fundamentals.md) · [Concurrency](04-rust/concurrency-patterns.md) · [Exercises](04-rust/reference-exercises.md)

-   **E · Validation, operations & delivery**

    ---

    Establish reproducible evidence, operational contracts and readiness gates.

    [Benchmarks](06-benchmarking/benchmark-methodology.md) · [Operations](07-operations/deployment.md) · [Roadmap](09-roadmap/implementation-roadmap.md)

</div>

## Documentation map

| Section | Responsibility | Entry point |
| --- | --- | --- |
| 00 · Governance | Sources, assessment, decisions and traceability | [Research baseline](00-governance/research-baseline.md) |
| 01 · Product | Requirements, capabilities and MVP boundary | [Product requirements](01-product/product-requirements.md) |
| 02 · Architecture | Runtime topology, interfaces and data flow | [System architecture](02-architecture/system-architecture.md) |
| 03 · Engineering | Algorithms, invariants and subsystem design | [Engineering requirements](03-engineering/engineering-requirements.md) |
| 04 · Rust | Language concepts and implementation patterns | [Rust fundamentals](04-rust/fundamentals.md) |
| 05 · Financial domain | Protocols, sequencing, order books and query semantics | [Market data protocols](05-financial-domain/market-data-protocols.md) |
| 06 · Benchmarking | Workloads, tests, hardware and acceptance gates | [Benchmark methodology](06-benchmarking/benchmark-methodology.md) |
| 07 · Operations | Deployment, observability, security and recovery | [Deployment](07-operations/deployment.md) |
| 08 · Decisions | Architecture proposals and approval records | [ADR-0001: Product boundary](08-decisions/ADR-0001-product-boundary.md) |
| 09 · Roadmap | Dependencies, milestones and implementation readiness | [Implementation roadmap](09-roadmap/implementation-roadmap.md) |

## From research to evidence

```mermaid
flowchart LR
    R[Original research] --> A[Assessment and decisions]
    A --> P[Approved requirements]
    P --> D[Approved designs]
    D --> I[Rust implementation]
    I --> V[Tests and benchmarks]
```

This is the proposed documentation authority hierarchy. Each stage needs explicit evidence and review before it authorizes the next.

## Read and contribute

Begin with the [research assessment](00-governance/research-assessment.md), then follow product definition, architecture, engineering design and validation. Keep source claims, proposed improvements and approved decisions distinct. See the [decision register](00-governance/decision-register.md) and [traceability](00-governance/traceability.md) for the populated Phase 0 records.

The source plan is preserved in the [research directory](https://github.com/mbellary/chronoshft/tree/main/research). The [repository README](https://github.com/mbellary/chronoshft#readme) explains local preview and publication.
