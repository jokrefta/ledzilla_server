use serde::{Deserialize, Serialize};

/// Not implemented yet
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct MotionConfig {
    pub direction_degrees: u16,
    pub distance_per_tick: u32,
    pub periodicity: u32,
}
