use fs_err::File;
use serde::de::DeserializeOwned;
use std::io::BufReader;
use std::time::Instant;

use crate::shared::{Multiplier, NodePT, NodeWalk, SubpurposeScore};

pub fn read_sparse_node_values_2d_serial(year: i32) -> Vec<Vec<SubpurposeScore>> {
    let now = Instant::now();
    let sparse_node_values_2d: Vec<Vec<SubpurposeScore>> =
        deserialize_bincoded_file(&format!("sparse_node_values_6am_{year}_2d"));
    println!("Serial loading took {:?}", now.elapsed());
    sparse_node_values_2d
}

pub fn read_rust_node_longlat_lookup_serial() -> Vec<[f64; 2]> {
    let rust_node_longlat_lookup: Vec<[f64; 2]> =
        deserialize_bincoded_file(&format!("rust_nodes_long_lat"));
    rust_node_longlat_lookup
}

pub fn read_files_parallel_excluding_node_values(year: i32) -> (Vec<NodeWalk>, Vec<NodePT>) {
    let now = Instant::now();

    let (graph_walk, graph_pt) = rayon::join(
        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("p1_main_nodes_vector_6am_{year}")),
        || deserialize_bincoded_file::<Vec<NodePT>>(&format!("p2_main_nodes_vector_6am_{year}")),
    );

    println!(
        "Parallel loading for files excluding travel time relationships took {:?}",
        now.elapsed()
    );
    (graph_walk, graph_pt)
}

pub fn read_small_files_serial() -> (
    Vec<Multiplier>,
    Vec<Multiplier>,
    Vec<Multiplier>,
    Vec<Multiplier>,
    [usize; 32],
) {
    let now = Instant::now();

    let travel_time_relationships_7: Vec<Multiplier> =
        deserialize_bincoded_file("travel_time_relationships_7");
    let travel_time_relationships_10: Vec<Multiplier> =
        deserialize_bincoded_file("travel_time_relationships_10");
    let travel_time_relationships_16: Vec<Multiplier> =
        deserialize_bincoded_file("travel_time_relationships_16");
    let travel_time_relationships_19: Vec<Multiplier> =
        deserialize_bincoded_file("travel_time_relationships_19");
    let subpurpose_purpose_lookup: [usize; 32] =
        deserialize_bincoded_file("subpurpose_purpose_lookup");

    println!("Serial loading took {:?}", now.elapsed());
    (
        travel_time_relationships_7,
        travel_time_relationships_10,
        travel_time_relationships_16,
        travel_time_relationships_19,
        subpurpose_purpose_lookup,
    )
}

pub fn deserialize_bincoded_file<T: DeserializeOwned>(filename: &str) -> T {
    let path = format!("serialised_data/{}.bin", filename);
    let file = BufReader::new(File::open(path).unwrap());
    bincode::deserialize_from(file).unwrap()
}


















