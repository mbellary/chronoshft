//! Explicit migration status; these stages are distinct from experiment stages.
pub struct Stage {
    pub number: u8,
    pub title: &'static str,
    pub state: &'static str,
}

pub const STAGES: &[Stage] = &[
    Stage {
        number: 1,
        title: "Compatibility baseline and Rust foundations",
        state: "implemented",
    },
    Stage {
        number: 2,
        title: "Registry validation",
        state: "implemented",
    },
    Stage {
        number: 3,
        title: "History and evidence verification",
        state: "implemented",
    },
    Stage {
        number: 4,
        title: "Documentation generation and link checks",
        state: "implemented",
    },
    Stage {
        number: 5,
        title: "Execution capture and experiment helpers",
        state: "implemented",
    },
    Stage {
        number: 6,
        title: "Reviewer workflow",
        state: "implemented",
    },
    Stage {
        number: 7,
        title: "Regression tests, CI and instructions",
        state: "implemented",
    },
    Stage {
        number: 8,
        title: "Cutover and Python tooling retirement",
        state: "pending",
    },
];

pub const BASELINE: &str = "tests/compatibility/python-v1/manifest.json";
