use serde::{Deserialize, Serialize};
use std::cmp::{Ord, PartialEq, PartialOrd};
use std::hash::Hash;

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct NodeID(pub u32);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Cost(pub u16);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct LeavingTime(pub u32);

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgeWalk {
    pub to: NodeID,
    pub cost: Cost,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgePT {
    pub leavetime: LeavingTime,
    pub cost: Cost,
}

#[derive(Deserialize)]
pub struct UserInputJSON {
    pub start_nodes_user_input: Vec<i32>,
    pub init_travel_times_user_input: Vec<i32>,
    pub trip_start_seconds: i32,
}

// FloatBinHeap is to allow f64 in a binary heap
// No longer used
/*
#[derive(Debug, PartialEq, PartialOrd, Serialize)]
pub struct FloatBinHeap(pub f64);

impl Eq for FloatBinHeap {}

impl Ord for FloatBinHeap {
    fn cmp(&self, other: &Self) -> Ordering {
        //self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal) // for reverse ordering
    }
}

impl Hash for FloatBinHeap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let float_bits = self.0.to_bits();
        float_bits.hash(state);
    }
}
*/