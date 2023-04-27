use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::{Ord, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::hash::Hash;

// Serializes a `usize` as a `u32` to save space. Useful when you need `usize` for indexing, but
// the values don't exceed 2^32.
pub fn serialize_usize<S: Serializer>(x: &usize, s: S) -> Result<S::Ok, S::Error> {
    if let Ok(x) = u32::try_from(*x) {
        x.serialize(s)
    } else {
        Err(serde::ser::Error::custom(format!("{} can't fit in u32", x)))
    }
}

// Deserializes a `usize` from a `u32`.
pub fn deserialize_usize<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    let x = <u32>::deserialize(d)?;
    Ok(x as usize)
}

// Same as above but for serialising u32 as u16
pub fn serialize_u32_as_u16<S: Serializer>(x: &u32, s: S) -> Result<S::Ok, S::Error> {
    if let Ok(x) = u16::try_from(*x) {
        x.serialize(s)
    } else {
        Err(serde::ser::Error::custom(format!("{} can't fit in u16", x)))
    }
}
pub fn deserialize_u32_as_u16<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    let x = <u16>::deserialize(d)?;
    Ok(x as usize)
}


// NodeID is a usize, which is saved as u32 to save space
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct NodeID(
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub usize,
);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct LinkID(pub u32);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Cost(
    #[serde(
        serialize_with = "serialize_u32_as_u16",
        deserialize_with = "deserialize_u32_as_u16"
    )]
    pub u32,
);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct HasPt(pub bool);

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Score(pub f64);

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgeWalk {
    pub to: NodeID,
    pub cost: Cost,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgePT {
    pub leavetime: Cost,
    pub cost: Cost,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SubpurposeScore {
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub subpurpose_ix: usize,
    pub subpurpose_score: Score,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GraphWalk {
    pub pt_status: HasPt,
    pub node_connections: SmallVec<[EdgeWalk; 4]>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct GraphPT {
    pub next_stop_node: NodeID,
    pub timetable: SmallVec<[EdgePT; 4]>,
}

#[derive(Deserialize)]
pub struct UserInputJSON {
    pub start_nodes_user_input: Vec<NodeID>,
    pub init_travel_times_user_input: Vec<Cost>,
    pub trip_start_seconds: Cost,
}

pub struct FloodfillOutput {
    pub start_node_id: NodeID,
    // TODO destinations_reached: Vec<(NodeID, Cost, Path)>
    pub destination_ids: Vec<NodeID>,
    pub destination_travel_times: Vec<Cost>,
    pub nodes_visited_sequences: Vec<Vec<NodeID>>,
    pub init_travel_time: Cost,
}

#[derive(Serialize)]
pub struct LinkCoords {
    pub start_node_longlat: [f64; 3],
    pub end_node_longlat: [f64; 3],
}

#[derive(Serialize)]
pub struct PurposeScores {
    pub Business: f64,
    pub Education: f64,
    pub Entertainment: f64,
    pub Shopping: f64,
    pub VisitFriends: f64,
}

#[derive(Serialize)]
pub struct FinalOutput {
    pub num_iterations: i32,
    pub start_node: NodeID,
    pub score_per_purpose: [f64; 5],
    pub per_link_score_per_purpose: HashMap<LinkID, [f64; 5]>,
    pub link_coordinates: HashMap<LinkID, Vec<String>>,
    pub key_destinations_per_purpose: [[[f64; 2]; 3]; 5],
    pub init_travel_time: u16,
}

#[derive(Serialize, Debug)]
pub struct LinkCoordsString {
    pub start_node_longlat: String,
    pub end_node_longlat: String,
}

impl LinkCoords {
    pub fn to_string_with_6dp(&self) -> LinkCoordsString {
        let start_node_longlat = self
            .start_node_longlat
            .iter()
            .map(|n| format!("{:.6}", n))
            .collect::<Vec<String>>()
            .join(",");
        let end_node_longlat = self
            .end_node_longlat
            .iter()
            .map(|n| format!("{:.6}", n))
            .collect::<Vec<String>>()
            .join(",");
        LinkCoordsString {
            start_node_longlat,
            end_node_longlat,
        }
    }
}
