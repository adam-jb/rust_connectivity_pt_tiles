use derive_more::{From, Into};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;
use std::cmp::{Ord, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Sub};

pub const TOP_CLUSTERS_COUNT: usize = 10;

// Serializes a `usize` as a `u32` to save space. Useful when you need `usize` for indexing, and
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
pub fn serialize_usize_as_u16<S: Serializer>(x: &usize, s: S) -> Result<S::Ok, S::Error> {
    if let Ok(x) = u16::try_from(*x) {
        x.serialize(s)
    } else {
        Err(serde::ser::Error::custom(format!("{} can't fit in u16", x)))
    }
}

pub fn deserialize_usize_as_u16<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    let x = <u16>::deserialize(d)?;
    Ok(x as usize)
}

// NodeID is a usize, which is saved as u32 to save space
#[derive(
    Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug, From, Into,
)]
pub struct NodeID(
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub usize,
);

#[derive(
    Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, From, Into,
)]
pub struct SecondsPastMidnight(
    #[serde(
        serialize_with = "serialize_usize",
        deserialize_with = "deserialize_usize"
    )]
    pub usize,
);

// Allow instances of SecondsPastMidnight type to do minus '-' operation with other instances of this type
impl Sub for SecondsPastMidnight {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        SecondsPastMidnight(self.0 - other.0)
    }
}

#[derive(
    Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, From, Into,
)]
pub struct Cost(
    #[serde(
        serialize_with = "serialize_usize_as_u16",
        deserialize_with = "deserialize_usize_as_u16"
    )]
    pub usize,
);

// Allow instances of Cost to be summed
impl Add for Cost {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Cost(self.0 + other.0)
    }
}

// Allow Cost to be multiplied by SecondsPastMidnight, or compared against, or added
impl SecondsPastMidnight {
    pub fn add(&self, other: &Cost) -> SecondsPastMidnight {
        SecondsPastMidnight(self.0 + other.0)
    }
}

// to allow a SecondsPastMidnight instance to cast into Cost with val.into()
impl From<SecondsPastMidnight> for Cost {
    fn from(seconds_past_midnight: SecondsPastMidnight) -> Cost {
        Cost(seconds_past_midnight.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Multiplier(pub f64);

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Debug, From, Into)]
pub struct Score(pub f64);

// Allow Score to be multiplied by Multiplier, and to get the natural log of itself
impl Score {
    pub fn multiply(&self, multiplier: Multiplier) -> Score {
        Score(self.0 * multiplier.0)
    }

    pub fn ln(self) -> Self {
        Score(self.0.ln())
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

// To allow creation of empty vec of scores
impl Default for Score {
    fn default() -> Self {
        Score(0.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct EdgeWalk {
    pub to: NodeID,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeWalk {
    pub has_pt: bool,
    pub node_connections: SmallVec<[EdgeWalk; 4]>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct EdgePT {
    pub leavetime: SecondsPastMidnight,
    pub cost: Cost,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodePT {
    pub next_stop_node: NodeID,
    pub timetable: SmallVec<[EdgePT; 4]>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DestinationReached {
    pub node: NodeID,
    pub cost: Cost,
    pub previous_node: NodeID,
    pub previous_node_iters_taken: usize,
    pub arrived_at_node_by_pt: u8, // 0 for walk; 1 for PT
}

pub struct FloodfillOutput {
    pub start_node_id: NodeID,
    pub seconds_walk_to_start_node: Cost,
    pub destinations_reached: Vec<DestinationReached>,
}

#[derive(Serialize)]
pub struct FinalOutput {
    pub start_node: NodeID,
    pub init_travel_time: Cost,
    pub num_iterations: u32,
    pub score_per_purpose: [Score; 5],
    pub key_destinations_per_purpose: [[[f64; 2]; TOP_CLUSTERS_COUNT]; 5],
    pub per_link_score_per_purpose: Vec<[Score; 5]>,
    pub link_coordinates: Vec<Vec<String>>,
    pub link_is_pt: Vec<u8>,
    pub link_route_details: Vec<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct UserInputJSON {
    pub start_nodes_user_input: Vec<NodeID>,
    pub init_travel_times_user_input: Vec<Cost>,
    pub trip_start_seconds: SecondsPastMidnight,
}
