use fs_err::File;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::Instant;

use common::structs::{
    Cost, EdgeRoute, EdgeWalk, Multiplier, NodeID, LinkID, Angle, NodeRoute, NodeWalk,NodeWalkCyclingCar, Score, SecondsPastMidnight,
    SubpurposeScore, EdgeWalkCyclingCar,
};
use common::read_file_funcs::deserialize_bincoded_file;


// All serialisation you want to do should go here
fn main() {
    serialise_files(2022);
}

pub fn serialise_files(year: i32) {
    let now = Instant::now();

    serialise_graph_walk_and_len(year);
    serialise_graph_routes(year);
    serialise_node_values_padding_count(year);
    serialise_route_info(year);

    serialise_list_immutable_array_usize("subpurpose_purpose_lookup");
    serialise_list_multiplier("travel_time_relationships_7");
    serialise_list_multiplier("travel_time_relationships_10");
    serialise_list_multiplier("travel_time_relationships_16");
    serialise_list_multiplier("travel_time_relationships_19");

    serialise_sparse_node_values_2d(&*format!("sparse_node_values_6am_{year}_2d")); // &* converts String to &str
    serialise_rust_node_longlat_lookup(year);
    
    serialise_graph_walk_cycling_car_vector("walk");
    serialise_graph_walk_cycling_car_vector("cycling");
    
    serialise_sparse_node_values_2d("sparse_node_values_walk");
    serialise_sparse_node_values_2d("sparse_node_values_cycling");
    
    serialise_list_multiplier("walk_travel_time_relationships_7");
    serialise_list_multiplier("cycling_travel_time_relationships_7");
    
    println!("File serialisation year {}/tTook {:?}", year, now.elapsed());
}

fn serialise_graph_walk_cycling_car_vector(mode: &str) {
    let contents_filename = format!("data/graph_{}.json", mode);
    let contents = fs_err::read_to_string(contents_filename).unwrap();

    let input: Vec<Vec<[usize; 5]>> = serde_json::from_str(&contents).unwrap();

    let mut graph_walk = Vec::new();
    for input_edges in input.iter() {
        let mut edges: SmallVec<[EdgeWalkCyclingCar; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalkCyclingCar {
                cost: Cost(array[0] as usize),
                to: NodeID(array[1] as usize),
                angle_leaving_node_from: Angle(array[2] as u16),
                angle_arrived_from: Angle(array[3] as u16),
                link_arrived_from: LinkID(array[4] as u32),
            });
        }
        graph_walk.push(NodeWalkCyclingCar {
            edges
        });
    }

    let filename = format!("serialised_data/graph_{}.bin", mode);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk).unwrap();
}


fn serialise_graph_routes(year: i32) {
    let contents_filename = format!("data/graph_pt_routes_6am_{}.json", year);
    let file = File::open(Path::new(&contents_filename)).unwrap();
    let reader = BufReader::new(file);

    let routes: Vec<serde_json::Value> = serde_json::from_reader(reader).unwrap();
    
    let mut graph_routes: Vec<NodeRoute> = Vec::new();

    for item in routes.iter() {
        let next_stop_node: NodeID =
            serde_json::from_value(item["next_stop_node"].clone()).unwrap();

        let timetable: Vec<[usize; 2]> = serde_json::from_value(item["timetable"].clone()).unwrap();

        let mut edges: SmallVec<[EdgeRoute; 4]> = SmallVec::new();
        for array in timetable {
            edges.push(EdgeRoute {
                leavetime: SecondsPastMidnight(array[0]),
                cost: Cost(array[1]),
            });
        }
        
        graph_routes.push(NodeRoute {
            next_stop_node: next_stop_node,
            timetable: edges,
        });
    }
    
    // Pad with empty values so length matches that of graph_walk
    let graph_walk: Vec<NodeWalk> = deserialize_bincoded_file(&format!("graph_pt_walk_6am_{year}"));
    for _i in routes.len()..graph_walk.len() {
        graph_routes.push(NodeRoute::make_empty_instance());
    }
    assert!(graph_walk.len() == graph_routes.len());

    // Serialize the graph data into a binary file
    let filename = format!("serialised_data/graph_pt_routes_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_routes).unwrap();
}

pub fn serialise_sparse_node_values_2d(input_str: &str) {
    let inpath = format!("data/{}.json", input_str);
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

    let outpath = format!("serialised_data/{}.bin", input_str);
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

fn serialise_graph_walk_and_len(year: i32) {
    let contents_filename = format!("data/graph_pt_walk_6am_{}.json", year);
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
            edges: edges,
        });
    }

    let filename = format!("serialised_data/graph_pt_walk_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk_vec).unwrap();
    
    let filename = format!("serialised_data/graph_pt_walk_len_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk_vec.len()).unwrap();
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
