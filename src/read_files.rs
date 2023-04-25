use fs_err::File;
use serde::de::DeserializeOwned;
use smallvec::SmallVec;
use std::io::BufReader;
use std::time::Instant;

use crate::shared::{EdgePT, EdgeWalk};

pub fn read_sparse_node_values_2d_serial(year: i32) -> Vec<Vec<[i32; 2]>> {
    let now = Instant::now();
    let sparse_node_values_2d: Vec<Vec<[i32; 2]>> =
        deserialize_bincoded_file(&format!("sparse_node_values_6am_{year}_2d"));
    println!("Serial loading took {:?}", now.elapsed());
    return sparse_node_values_2d;
}

pub fn read_files_parallel_excluding_node_values(
    year: i32,
) -> (Vec<SmallVec<[EdgeWalk; 4]>>, Vec<SmallVec<[EdgePT; 4]>>) {
    let now = Instant::now();

    let (graph_walk, graph_pt) = rayon::join(
        || {
            deserialize_bincoded_file::<Vec<SmallVec<[EdgeWalk; 4]>>>(&format!(
                "p1_main_nodes_vector_6am_{year}"
            ))
        },
        || {
            deserialize_bincoded_file::<Vec<SmallVec<[EdgePT; 4]>>>(&format!(
                "p2_main_nodes_vector_6am_{year}"
            ))
        },
    );

    println!(
        "Parallel loading for files excluding travel time relationships took {:?}",
        now.elapsed()
    );
    (graph_walk, graph_pt)
}

pub fn read_small_files_serial() -> (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, [i8; 32]) {
    let now = Instant::now();

    let travel_time_relationships_7: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_7");
    let travel_time_relationships_10: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_10");
    let travel_time_relationships_16: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_16");
    let travel_time_relationships_19: Vec<i32> =
        deserialize_bincoded_file("travel_time_relationships_19");
    let subpurpose_purpose_lookup: [i8; 32] =
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
