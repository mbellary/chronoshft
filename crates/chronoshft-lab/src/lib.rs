//! Rust migration foundations for the Technical Validation Lab.
//!
//! Stages 2–7 provide validation, documentation, capture/import, experiment,
//! reviewer workflows and the shared MkDocs gate. Legacy-tool retirement is Stage 8.
pub mod cli;
pub mod migration;

pub mod registry;

pub mod capture;
pub mod docs;
pub mod evidence;
pub mod experiment;
pub mod gate;
pub mod history;
pub mod process;
pub mod publication;
pub mod review;
pub mod runtime;
