#[allow(dead_code)]
// Simple sway script used for testing LSP capabilites
pub(crate) const TEST_SWAY_PROGRAM: &str = r#"script;

use std::*;

/// A simple Particle struct
struct Particle {
    position: [u64; 3],
    velocity: [u64; 3],
    acceleration: [u64; 3],
    mass: u64,
}

impl Particle {
    /// Creates a new Particle with the given position, velocity, acceleration, and mass
    fn new(position: [u64; 3], velocity: [u64; 3], acceleration: [u64; 3], mass: u64) -> Particle {
        Particle {
            position: position,
            velocity: velocity,
            acceleration: acceleration,
            mass: mass,
        }
    }
}

fn main() {
    let position = [0, 0, 0];
    let velocity = [0, 1, 0];
    let acceleration = [1, 1, 0];
    let mass = 10;
    let p = ~Particle::new(position, velocity, acceleration, mass);
}
"#;

#[allow(dead_code)]
// Simple manifest file for testing LSP capabilites
pub(crate) const TEST_MANIFEST: &str = r#"[project]
name = "lsp_test_project"
authors = ["Fuel Labs"]
entry = "main.sw"
license = "Apache-2.0"

[dependencies]
"#;

#[allow(dead_code)]
// Simple manifest lock file for testing LSP capabilites
pub(crate) const TEST_MANIFEST_LOCK: &str = r#"[[package]]
name = 'core'
source = 'path+from-root-1B78E18C184A86E5'
dependencies = []

[[package]]
name = 'lsp_test_project'
source = 'root'
dependencies = ['std']

[[package]]
name = 'std'
source = 'git+https://github.com/fuellabs/sway?tag=v0.15.1#a34b4b99fcdd065d559f6cbb9dec0697c3f5edd1'
dependencies = ['core']
"#;
