use fs_err::File;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::Instant;

use crate::shared::{
    Cost, EdgePT, EdgeWalk, Multiplier, NodeID, NodePT, NodeWalk, Score, SecondsPastMidnight,
    SubpurposeScore,
};

pub fn serialise_sparse_node_values_2d(year: i32) {
    let inpath = format!("data/sparse_node_values_6am_{}_2d.json", year);
    let file = File::open(Path::new(&inpath)).unwrap();
    let reader = BufReader::new(file);
    let input: Vec<serde_json::Value> = serde_json::from_reader(reader).unwrap();

    let mut output: Vec<Vec<SubpurposeScore>> = Vec::new();
    for item in input.iter() {
        let sparse_subpurpose_scores_this_node: Vec<[usize; 2]> =
            serde_json::from_value(item.clone()).unwrap();
        let mut output_this_node: Vec<SubpurposeScore> = Vec::new();

        for val in sparse_subpurpose_scores_this_node.iter() {
            output_this_node.push(SubpurposeScore {
                subpurpose_ix: val[0] as usize,
                subpurpose_score: Score(val[1] as f64),
            });
        }
        output.push(output_this_node);
    }
    println!("Read and processed from from {}", inpath);

    let outpath = format!("serialised_data/sparse_node_values_6am_{}_2d.bin", year);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised sparse_node_values to {}", outpath);
}

pub fn serialise_rust_node_longlat_lookup(year: i32) {
    let inpath = format!("data/rust_nodes_long_lat_{}.json", year);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<[f64; 2]> = serde_json::from_str::<Vec<[f64; 2]>>(&contents)
        .unwrap()
        .into();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/rust_nodes_long_lat.bin");
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised rust_node_longlat_lookup to {}", outpath);
}

pub fn serialise_files(year: i32) {
    let now = Instant::now();

    let _len_graph_walk = serialise_graph_walk_vector(year);
    serialise_graph_pt_vector(year);
    serialise_node_values_padding_count(year);
    serialise_route_info(year);

    serialise_list_immutable_array_usize("subpurpose_purpose_lookup");
    serialise_list_multiplier("travel_time_relationships_7");
    serialise_list_multiplier("travel_time_relationships_10");
    serialise_list_multiplier("travel_time_relationships_16");
    serialise_list_multiplier("travel_time_relationships_19");
    println!("File serialisation year {}/tTook {:?}", year, now.elapsed());
}

fn serialise_node_values_padding_count(year: i32) {
    let contents_filename = format!("data/node_values_padding_row_count_6am_{}.json", year);
    let contents = fs_err::read_to_string(contents_filename).unwrap();
    let input_value: u32 = serde_json::from_str(&contents).unwrap();
    let filename = format!(
        "serialised_data/node_values_padding_row_count_6am_{}.bin",
        year
    );
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &input_value).unwrap();
}

fn serialise_graph_walk_vector(year: i32) -> usize {
    let contents_filename = format!("data/p1_main_nodes_updated_6am_{}.json", year);
    let file = File::open(Path::new(&contents_filename)).unwrap();
    let reader = BufReader::new(file);

    let input: Vec<serde_json::Value> = serde_json::from_reader(reader).unwrap();
    let mut graph_walk_vec: Vec<NodeWalk> = Vec::new();

    for item in input.iter() {
        // Converting 1 or 0 into boolean for has_pt
        let pt_status_integer = item["pt_status"].as_i64().unwrap();
        let pt_status_boolean = if pt_status_integer == 1 { true } else { false };

        let node_connections: Vec<[usize; 2]> =
            serde_json::from_value(item["node_connections"].clone()).unwrap();

        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in node_connections {
            edges.push(EdgeWalk {
                cost: Cost(array[0]),
                to: NodeID(array[1]),
            });
        }
        graph_walk_vec.push(NodeWalk {
            has_pt: pt_status_boolean,
            node_connections: edges,
        });
    }

    let filename = format!("serialised_data/p1_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk_vec).unwrap();
    return graph_walk_vec.len();
}

fn serialise_graph_pt_vector(year: i32) {
    //, len_graph_walk: usize) {
    let contents_filename = format!("data/p2_main_nodes_updated_6am_{}.json", year);
    let file = File::open(Path::new(&contents_filename)).unwrap();
    let reader = BufReader::new(file);

    let input: Vec<serde_json::Value> = serde_json::from_reader(reader).unwrap();
    let mut graph_pt_vec: Vec<NodePT> = Vec::new();

    for item in input.iter() {
        let next_stop_node: NodeID =
            serde_json::from_value(item["next_stop_node"].clone()).unwrap();

        let timetable: Vec<[usize; 2]> = serde_json::from_value(item["timetable"].clone()).unwrap();

        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in timetable {
            edges.push(EdgePT {
                leavetime: SecondsPastMidnight(array[0]),
                cost: Cost(array[1]),
            });
        }
        graph_pt_vec.push(NodePT {
            next_stop_node: next_stop_node,
            timetable: edges,
        });
    }

    // Add empty edges to ensure that each node has the same number of edges
    // DROPPED as believe this is unnecessary as all nodes with graph connections are at front of the graph_walk vec. Adam, 30th April 2023
    /*
    for _ in graph_pt_vec.len()..len_graph_walk {
        let edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        graph_walk_vec.push(NodePT {
            next_stop_node: NodeID(0),
            timetable: edges,
        });
    }
    assert!(graph_pt_vec.len() == len_graph_walk);
    */

    // Serialize the graph data into a binary file
    let filename = format!("serialised_data/p2_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_pt_vec).unwrap();
}

pub fn serialise_route_info(year: i32) {
    let contents_filename = format!("data/routes_info_{}.json", year);
    let file = File::open(Path::new(&contents_filename)).unwrap();
    let reader = BufReader::new(file);
    let input: Vec<serde_json::Value> = serde_json::from_reader(reader).unwrap();
    println!("Read routes_info from {}", contents_filename);

    // Convert route info dicts to strings
    let mut output: Vec<HashMap<String, String>> = Vec::new();
    for item in input.iter() {
        let next_val_map: HashMap<String, String> = serde_json::from_value(item.clone()).unwrap();
        // If storing as a string rather than hashmap
        // let next_val_str = serde_json::to_string(&next_val_map).unwrap();
        output.push(next_val_map);
    }

    let outpath = format!("serialised_data/route_info_{}.bin", year);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_list_multiplier(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<Multiplier> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_list_immutable_array_usize(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: [usize; 32] = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}
