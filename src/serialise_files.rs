use fs_err::File;
use smallvec::SmallVec;
use std::io::{BufReader, BufWriter};
use std::time::Instant;
use typed_index_collections::TiVec;

use crate::shared::{
    Cost, EdgePT, EdgeWalk, GraphPT, GraphWalk, HasPt, NodeID, Score, SubpurposeScore,
};

pub fn serialise_sparse_node_values_2d(year: i32) {
    let inpath = format!("data/sparse_node_values_6am_{}_2d.json", year);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: TiVec<NodeID, Vec<SubpurposeScore>> =
        serde_json::from_str(&contents).unwrap().into();

    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/sparse_node_values_6am_{}_2d.bin", year);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

pub fn serialise_rust_node_longlat_lookup() {
    let inpath = format!("data/rust_nodes_long_lat.json");
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: TiVec<NodeID, [f64; 2]> = serde_json::from_str(&contents).unwrap().into();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/rust_nodes_long_lat.bin");
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

pub fn serialise_files(year: i32) {
    let now = Instant::now();

    let len_graph_walk = serialise_graph_walk_vector(year);
    serialise_graph_pt_vector(year, len_graph_walk);
    serialise_node_values_padding_count(year);
    serialise_route_info(year);

    serialise_list_immutable_array_i8("subpurpose_purpose_lookup");
    serialise_list_Score("travel_time_relationships_7");
    serialise_list_Score("travel_time_relationships_10");
    serialise_list_Score("travel_time_relationships_16");
    serialise_list_Score("travel_time_relationships_19");
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

    let mut graph_walk_vec: TiVec<NodeID, GraphWalk> = Vec::new().into();
    //let mut graph_walk_vec = Vec::new();
    for item in input.iter() {
        let pt_status = item["pt_status"].as_bool().unwrap();
        let node_connections: Vec<[usize; 2]> =
            serde_json::from_value(item["node_connections"].clone()).unwrap();
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in node_connections {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u32),
            });
        }
        graph_walk_vec.push(GraphWalk {
            pt_status: HasPt(pt_status),
            node_connections: edges,
        });
    }

    let filename = format!("serialised_data/p1_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk_vec).unwrap();
    return graph_walk_vec.len();
}

fn serialise_graph_pt_vector(year: i32, len_graph_walk: usize) {
    let contents_filename = format!("data/p2_main_nodes_updated_6am_{}.json", year);
    let file = File::open(Path::new(&contents_filename)).unwrap();
    let reader = BufReader::new(file);

    let input: Vec<serde_json::Value> = serde_json::from_reader(reader).unwrap();

    let mut graph_pt_vec: TiVec<NodeID, GraphPT> = Vec::new().into();
    //let mut graph_pt_vec = Vec::new();
    for item in input.iter() {
        let next_stop_node = item["pt_status"].parse::<i32>().unwrap();
        let timetable: Vec<[usize; 2]> =
            serde_json::from_value(item["timetables"].clone()).unwrap();
        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in timetable {
            edges.push(EdgePT {
                leavetime: Cost(array[0] as u32),
                cost: Cost(array[0] as u32),
            });
        }
        graph_walk_vec.push(GraphPT {
            next_stop_node: NodeID(next_stop_node),
            timetable: edges,
        });
    }

    // Add empty edges to ensure that each node has the same number of edges
    for _ in graph_pt_vec.len()..len_graph_walk {
        let edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        graph_walk_vec.push(GraphPT {
            next_stop_node: NodeID(next_stop_node),
            timetable: edges,
        });
    }
    assert!(graph_pt_vec.len() == len_graph_walk);

    // Serialize the graph data into a binary file
    let filename = format!("serialised_data/p2_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_pt_vec).unwrap();
}

fn serialise_route_info(year: i32) {
    let inpath = format!("data/route_info_{}.json", year);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: TiVec<NodeID, String> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/route_info_{}.bin", year);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_list_Score(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = fs_err::read_to_string(&inpath).unwrap();
    let output: Vec<Score> = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

fn serialise_list_immutable_array_i8(filename: &str) {
    let inpath = format!("data/{}.json", filename);
    let contents = std::fs::read_to_string(&inpath).unwrap();
    let output: [i8; 32] = serde_json::from_str(&contents).unwrap();
    println!("Read from {}", inpath);

    let outpath = format!("serialised_data/{}.bin", filename);
    let file = BufWriter::new(File::create(&outpath).unwrap());
    bincode::serialize_into(file, &output).unwrap();
    println!("Serialised to {}", outpath);
}

// This is the previous one
/*
fn serialise_graph_walk_vector(year: i32) -> usize {
    let contents_filename = format!("data/p1_main_nodes_updated_6am_{}.json", year);
    let contents = fs_err::read_to_string(contents_filename).unwrap();

    //!!!
    let input: Vec<Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    //!!
    let mut graph_walk_vec = Vec::new();
    for input_edges in input.iter() {
        let mut edges: SmallVec<[EdgeWalk; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgeWalk {
                to: NodeID(array[1] as u32),
                cost: Cost(array[0] as u16),
            });
        }
        graph_walk_vec.push(edges);
    }

    let filename = format!("serialised_data/p1_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_walk_vec).unwrap();
    return graph_walk_vec.len();
}
*/

/*
fn serialise_graph_pt_vector(year: i32, len_graph_walk: usize) {
    let contents_filename = format!("data/p2_main_nodes_updated_6am_{}.json", year);
    let contents = fs_err::read_to_string(contents_filename).unwrap();

    let input: Vec<Vec<[usize; 2]>> = serde_json::from_str(&contents).unwrap();

    let mut graph_pt_vec = Vec::new();
    for input_edges in input.iter() {
        let mut edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        for array in input_edges {
            edges.push(EdgePT {
                leavetime: LeavingTime(array[0] as u32),
                cost: Cost(array[1] as u16),
            });
        }
        graph_pt_vec.push(edges);
    }

    for _ in graph_pt_vec.len()..len_graph_walk {
        let edges: SmallVec<[EdgePT; 4]> = SmallVec::new();
        graph_pt_vec.push(edges);
    }
    assert!(graph_pt_vec.len() == len_graph_walk);

    let filename = format!("serialised_data/p2_main_nodes_vector_6am_{}.bin", year);
    let file = BufWriter::new(File::create(filename).unwrap());
    bincode::serialize_into(file, &graph_pt_vec).unwrap();
}
*/
