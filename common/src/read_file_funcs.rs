use fs_err::File;
use serde::de::DeserializeOwned;
use std::time::Instant;
use std::io::BufReader;

use crate::structs::{Multiplier, NodeRoute, NodeWalk, SubpurposeScore, NodeWalkCyclingCar, SUBPURPOSES_COUNT};

pub fn read_files_serial_walk_cycling_car(mode: &String, time_of_day: usize) -> (Vec<Multiplier>, Vec<Vec<SubpurposeScore>>, Vec<NodeWalkCyclingCar>) {

    let mut travel_time_relationships: Vec<Multiplier> = Vec::new();
    let mut graph: Vec<NodeWalkCyclingCar> = Vec::new();
    let mut sparse_node_values: Vec<Vec<SubpurposeScore>> = Vec::new();
    
    if mode == "car" {
        graph = deserialize_bincoded_file(&format!("graph_{}_{}", &mode, time_of_day));
        sparse_node_values = deserialize_bincoded_file(&format!("sparse_node_values_{}_{}", &mode, time_of_day));    
    }
    
    else {
        graph = deserialize_bincoded_file(&format!("graph_{}", &mode));
        sparse_node_values = deserialize_bincoded_file(&format!("sparse_node_values_{}", &mode));    
    }
    
    travel_time_relationships = deserialize_bincoded_file(&format!("{}_travel_time_relationships_{}", mode, time_of_day));
    
    (
        travel_time_relationships,
        sparse_node_values,
        graph,
    )
}

// read stop_rail_statuses_2022 as binary: standard deserialisation may be fine
pub fn read_stop_rail_statuses(year: i32) -> Vec<bool> {
    let stop_rail_statuses: Vec<bool> =
        deserialize_bincoded_file(&format!("stop_rail_statuses_{year}"));
    stop_rail_statuses
}

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

pub fn read_files_parallel_inc_node_values(year: i32) -> (Vec<Vec<SubpurposeScore>>, Vec<NodeWalk>, Vec<NodeRoute>) {
    let now = Instant::now();
    
    let (node_values_2d, (graph_walk, graph_routes)) = rayon::join(
        || deserialize_bincoded_file::<Vec<Vec<SubpurposeScore>>>(&format!("sparse_node_values_6am_{year}_2d")),
        || {
            rayon::join(
                || {
                    deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_6am_{year}"))
                },
                || {
                    deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_6am_{year}"))
                },
            )
        },
    );
    
    println!(
        "Parallel loading for files took {:?}",
        now.elapsed()
    );
    (node_values_2d, graph_walk, graph_routes)
}

// TO TRY: possible speed improvement: do appending with rayon too, so both graphs are appended to in parallel
pub fn read_files_extra_parallel_inc_node_values(year: i32) -> (Vec<Vec<SubpurposeScore>>, Vec<NodeWalk>, Vec<NodeRoute>) {
    let now = Instant::now();
    
    // if editing: make sure you get the types being deserialised into right: the compiler may panic without telling you why if you do
    let (((mut graph_walk1, mut graph_walk2), mut graph_walk3), ((mut graph_routes1, mut graph_routes2), (mut graph_routes3, node_values_2d))) =
    rayon::join(
        || {
            rayon::join(
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_1")),
                        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_2")),
                    )
                },
                || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_chunk_3")),
            )
        },
        || {
            rayon::join(
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_1")),
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_2")),
                    )
                },
                || {
                    rayon::join(
                        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_chunk_3")),
                        || deserialize_bincoded_file::<Vec<Vec<SubpurposeScore>>>(&format!("sparse_node_values_6am_{year}_2d")),
                    )
                },
            )
        }
    );
    
    println!(
        "Parallel loading for files without extend took {:?}",
        now.elapsed()
    );
    
    graph_walk1.reserve(graph_walk1.len() * 3 + 10);  // add 10 to reserve to ensure definitely space
    graph_walk1.append(&mut graph_walk2);
    graph_walk1.append(&mut graph_walk3);
    
    graph_routes1.reserve(graph_routes1.len() * 3 + 10);
    graph_routes1.append(&mut graph_routes2);
    graph_routes1.append(&mut graph_routes3);
    
    println!(
        "Parallel loading for files with {} chunks and extend took {:?}",
        3, 
        now.elapsed()
    );

    (node_values_2d, graph_walk1, graph_routes1)
}

pub fn read_files_parallel_excluding_node_values(year: i32) -> (Vec<NodeWalk>, Vec<NodeRoute>) {
    let now = Instant::now();

    let (graph_walk, graph_routes) = rayon::join(
        || deserialize_bincoded_file::<Vec<NodeWalk>>(&format!("graph_pt_walk_6am_{year}")),
        || deserialize_bincoded_file::<Vec<NodeRoute>>(&format!("graph_pt_routes_6am_{year}")),
    );

    println!(
        "Parallel loading for files excluding travel time relationships took {:?}",
        now.elapsed()
    );
    (graph_walk, graph_routes)
}

pub fn read_small_files_serial() -> (
    Vec<Multiplier>,
    Vec<Multiplier>,
    Vec<Multiplier>,
    Vec<Multiplier>,
    [usize; SUBPURPOSES_COUNT],
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
    
    let subpurpose_purpose_lookup: [usize; SUBPURPOSES_COUNT] = 
        read_vec_as_array_usize("subpurpose_to_purpose_integer");

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


pub fn read_vec_as_array_usize(filename: &str) -> [usize; SUBPURPOSES_COUNT] {
    let inpath = format!("serialised_data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output_vector: Vec<usize> = serde_json::from_str(&contents).unwrap();
    
    let mut output: [usize; SUBPURPOSES_COUNT] = [0; 33];
    for (index, value) in output_vector.iter().enumerate() {
        output[index] = *value;
    }
    return output
}

pub fn read_vec_as_array_multiplier(filename: &str) -> [Multiplier; SUBPURPOSES_COUNT] {
    let inpath = format!("serialised_data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output_vector: Vec<Multiplier> = serde_json::from_str(&contents).unwrap();
    
    let mut output: [Multiplier; SUBPURPOSES_COUNT] = [Multiplier(0.0); 33];
    for (index, value) in output_vector.iter().enumerate() {
        output[index] = *value;
    }
    return output
}








