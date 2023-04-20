use serde::{Deserialize, Serialize};
use std::cmp::{Ord, PartialEq, PartialOrd};
use std::collections::HashMap;
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

pub struct FloodfillOutput {
    pub start_node_id: u32,
    pub destination_ids: Vec<u32>,
    pub destination_travel_times: Vec<u16>,
    pub nodes_visited_sequences: Vec<Vec<u32>>,
    pub init_travel_time: u16,
}

#[derive(Serialize)]
pub struct LinkCoords {
    pub start_node_longlat: [f64; 3],
    pub end_node_longlat: [f64; 3],
}

#[derive(Serialize)]
pub struct FinalOutput {
    pub num_iterations: i32,
    pub start_node: u32,
    pub score_per_purpose: [f64; 5],
    pub per_link_score_per_purpose: HashMap<u32, [f64; 5]>,
    pub link_coordinates: HashMap<u32, LinkCoords>,
    pub key_destinations_per_purpose: [[[f64; 2]; 3]; 5],
    pub init_travel_time: u16,
}
