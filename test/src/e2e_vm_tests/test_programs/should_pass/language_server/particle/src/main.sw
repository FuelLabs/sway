script;

use std::*;

/// A simple Particle struct 
pub struct Particle {
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
    let acceleration = [1, 1, position[1]];
    let mass = 10;
    let p = ~Particle::new(position, velocity, acceleration, mass);
}
