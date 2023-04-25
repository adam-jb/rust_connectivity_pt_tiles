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

#[derive(Serialize)]
pub struct FloodfillOutput {
    pub start_node_id: NodeID,
    pub init_travel_time: u16,
    pub scaled_purpose_scores: [f64; 5],
}
