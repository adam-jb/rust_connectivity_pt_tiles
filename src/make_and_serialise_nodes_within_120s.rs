use crate::floodfill::get_travel_times;
use crate::read_files::read_files_parallel_excluding_node_values;
use fs_err::File;
use rayon::prelude::*;
use std::io::BufWriter;
use typed_index_collections::TiVec;

use crate::shared::{NodeID, Cost, SecondsPastMidnight, FloodfillOutput};

pub fn make_and_serialise_nodes_within_120s(year: i32) {
    println!("Begun make_and_serialise_nodes_within_120s");
    // For ~10m walking nodes, takes ~90 mins to get all nearby nodes in 120s with 8 core machine; 128gb RAM was enough and 32gb wasnt

    let (graph_walk, graph_pt) = read_files_parallel_excluding_node_values(year);

    let indices = (0..graph_walk.len()).collect::<Vec<_>>();
    println!("Number of iters to do: {}", graph_walk.len());

    let results: Vec<FloodfillOutput> = indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &graph_walk,
                &graph_pt,
                NodeID(*i as usize),
                SecondsPastMidnight(28800),
                Cost(0),
                true,
                Cost(120),
            )
        })
        .collect();
    println!("Floodfill done for all nodes in graph_walk");

    // write the neighbouring nodes to a vector. CHECK .into() call is needed to convert initialised vec to TiVec
    let mut nodes_to_neighbouring_nodes: TiVec<NodeID, Vec<NodeID>> =
        vec![vec![]; graph_walk.len()].into();
    for res in results {
        nodes_to_neighbouring_nodes[res.start_node_id] = res.destinations_reached;
    }

    let file =
        BufWriter::new(File::create("serialised_data/nodes_to_neighbouring_nodes.bin").unwrap());
    bincode::serialize_into(file, &nodes_to_neighbouring_nodes).unwrap();
    println!("Serialised serialised_data/nodes_to_neighbouring_nodes.bin");
}
