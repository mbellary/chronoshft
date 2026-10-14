# Rust foundations for the HFT engine

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Start with the engine you are building

ChronosHFT's working direction is a single-node financial time-series database and analytics engine: ingest market events, publish Arrow batches, query with DataFusion, persist Parquet history and stream Flight results. Strategies, order routing and pre-trade risk need separate scope decisions. Rust learning here follows this data path (research pp. 15–17, 26–28; PRD-F01–F12; DEC-001/002).

A **tick** is a market event. A **batch** groups events for columnar processing. A **snapshot** fixes the data a query can see. The **hot path** is the latency-sensitive ingest work; **cold storage** holds historical files. FDAP means Flight, DataFusion, Arrow and Parquet.

## Learning order

| Read next | Rust concept | Engine outcome |
| --- | --- | --- |
| This page | Syntax, types, control flow, modules and Cargo | Decode and validate a tick |
| [Ownership and lifetimes](ownership-and-lifetimes.md) | Moves, borrowing, slices and shared ownership | Keep batches alive through readers |
| [Idioms and errors](idioms-and-errors.md) | Newtypes, enums, Result and iteration | Express admission and failure explicitly |
| [Engine design patterns](engine-design-patterns.md) | Traits, state transitions and snapshots | Define subsystem boundaries |
| [Concurrency](concurrency-patterns.md) | Threads, channels, async and cancellation | Give each mutable builder one owner |
| [Memory management](memory-management.md) | Allocation, layout and retirement | Bound retained memory |
| [Atomics](atomics-and-memory-ordering.md) | Publication and memory ordering | Explain visibility before custom synchronization |
| [Lock-free patterns](lock-free-patterns.md) | SPSC protocols and unsafe review | Evaluate source prototypes safely |
| [FDAP integration](fdap-integration.md) | Cross-library types and ownership | Connect Arrow, SQL, files and clients |
| [Exercises](reference-exercises.md) | Runnable practice and evidence | Verify correctness before optimization |

## Toolchain and project basics

`rustc` compiles Rust; `cargo` builds projects and manages dependencies. A **crate** is a compilation unit, and a **module** organizes names within it. Start locally:

```sh
cargo new chronos-learning
cd chronos-learning
cargo run
cargo check
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --release
```

Put a complete example containing `fn main()` in `src/main.rs`. `cargo check` checks compilation without producing the final executable. Debug builds help development; release builds are appropriate for measured performance. Keep the toolchain version and Cargo.lock with reproducible applications. No Arrow/DataFusion version set is selected by this reference (ENG-001; DEC-010).

Use `src/lib.rs` for reusable decoder, schema and lifecycle logic; keep process startup in `src/main.rs`. Suggested modules are `feed`, `event`, `ingest`, `active`, `query`, `persist` and `service`. These are teaching names, not a mandated repository layout. `pub` exposes an item; private fields let constructors enforce invariants.

## Read Rust syntax through a market event

```rust
#[derive(Debug, Clone, Copy)]
struct Tick {
    event_ns: i64,
    instrument: u32,
    price_units: i64,
    quantity: u64,
}

fn notional(tick: &Tick) -> Option<i128> {
    i128::from(tick.price_units).checked_mul(i128::from(tick.quantity))
}

fn main() {
    let tick = Tick {
        event_ns: 1_700_000_000_000_000_000,
        instrument: 7,
        price_units: 15_025,
        quantity: 100,
    };
    let mut accepted = 0_u64;
    if tick.quantity > 0 {
        accepted += 1;
    }
    match notional(&tick) {
        Some(value) => println!("instrument={} ts={} notional={value}",
                                tick.instrument, tick.event_ns),
        None => println!("numeric overflow"),
    }
    assert_eq!(accepted, 1);
}
```

`let` binds a value; `mut` permits reassignment. `struct` groups named fields. `&Tick` borrows without transferring ownership. `->` names the return type. The last expression without a semicolon becomes the returned value. `Some` and `None` are the alternatives of `Option`; `match` handles them. `::` selects an associated function, while `.` calls a method. `#[derive(...)]` asks the compiler to generate selected trait implementations.

The example's price scale and quantity unit are illustrative. Its output is in price-units × quantity, not automatically a monetary amount. Production must ratify price/quantity units, timestamp semantics, nullability and overflow policy (ENG-007; DEC-007/014). A signed `i64` timestamp has finite range; nanosecond representation does not establish accurate clocks or a total event order.

## Types and collections you will use

| Type | Meaning | HFT use and caution |
| --- | --- | --- |
| `i64`, `u64`, `u32` | Fixed-width signed/unsigned integers | Time, quantity and identifiers; use checked conversions |
| `usize` | Platform-sized index | Buffer lengths; do not assume a portable wire width |
| `bool`, `enum` | Flag or named alternatives | Connection state and admission outcomes |
| `[T; N]`, `Vec<T>` | Fixed array or growable owned collection | Fixed tables and batched staging; bound growth |
| `&[T]` | Borrowed slice | Inspect a packet or batch without owning it |
| `String`, `&str` | Owned text or borrowed text | Config and diagnostics; avoid repeated formatting on append |
| `Option<T>`, `Result<T, E>` | Optional value or success/error | Empty publication, malformed input and disk failure |

A `for` loop can inspect a slice without indexing. `loop` runs until an explicit exit; every service loop needs shutdown behavior. `if` and `match` can produce values. Avoid assuming casts with `as` validate ranges: use `TryFrom` for incoming numeric values.

## Traits, generics and closures

A **trait** describes behavior shared by types; `impl` supplies that behavior. A generic function works with types satisfying a trait bound. `fn decode<D: Decoder>(decoder: &mut D, bytes: &[u8])` can specialize code for a concrete decoder. `Box<dyn Decoder>` stores a selected implementation behind an interface and uses dynamic dispatch. Choose based on extensibility and measurements, not a blanket prohibition of either approach.

A closure such as `|tick| tick.quantity > 0` is a small function value. `move` on a closure transfers captured ownership, commonly when spawning a worker. The [design patterns page](engine-design-patterns.md) shows where these fit the engine.

## Check your understanding

Explain why receive time differs from event time, why `usize` is a buffer index rather than a timestamp, and which failure `Option` represents in `notional`. Then work through [exercise 1](reference-exercises.md#practice-sequence).

Language companion: [The Rust Book](https://doc.rust-lang.org/book/). Engine contracts: [schema and sequencing](../03-engineering/event-schema-and-sequencing.md), ENG-001/002/007 and VAL-010/011/018.
