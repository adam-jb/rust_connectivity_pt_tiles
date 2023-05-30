use fs_err::File;
use rayon::prelude::*;
use std::io::BufWriter;
use typed_index_collections::TiVec;

use common::floodfill_public_transport_no_scores::floodfill_public_transport_no_scores;
use common::structs::{Cost, NodeID, NodeRoute, NodeWalk, SecondsPastMidnight, FloodfillOutput};
use common::read_file_funcs::read_files_parallel_excluding_node_values;

fn main() {
    
    let year = 2022;
    let (graph_walk, graph_routes) = read_files_parallel_excluding_node_values(year);
    
    let graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let graph_routes: TiVec<NodeID, NodeRoute> = TiVec::from(graph_routes);
    
    for seconds_travel_time in [10] { //vec![120, 180, 240, 300] {
        make_and_serialise_nodes_within_n_seconds(Cost(seconds_travel_time), &graph_walk, &graph_routes);
        println!("Found nearby nodes within {} seconds walk", seconds_travel_time);
    }
}

// For ~10m walking nodes, takes ~90 mins to get all nearby nodes in 120s with 8 core machine; 128gb RAM was enough and 32gb wasnt
pub fn make_and_serialise_nodes_within_n_seconds(
    seconds_travel_time: Cost,
    graph_walk: &TiVec<NodeID, NodeWalk>,
    graph_routes: &TiVec<NodeID, NodeRoute>,
) {
    println!("Begun make_and_serialise_nodes");

    let indices = (0..graph_walk.len()).collect::<Vec<_>>();
    println!("Number of iters to do: {}", graph_walk.len());

    let results: Vec<FloodfillOutput> = indices
        .par_iter()
        .map(|i| {
            floodfill_public_transport_no_scores(
                &graph_walk,
                &graph_routes,
                NodeID(*i as usize),
                SecondsPastMidnight(28800),
                Cost(0),
                true,
                seconds_travel_time,
            )
        })
        .collect();
    println!("Floodfill done for all nodes in graph_walk");
    
    // write the neighbouring nodes to a vector
    let mut nodes_to_neighbouring_nodes: Vec<Vec<NodeID>> = vec![vec![]; graph_walk.len()];
    for res in results {
        let mut nodes_reached = Vec::new();
        for destination in res.destinations_reached {
            nodes_reached.push(destination.node);
        }
        nodes_to_neighbouring_nodes[res.start_node_id.0] = nodes_reached;
    }

    let outpath = format!(
        "serialised_data/nodes_to_neighbouring_nodes_{}.bin",
        seconds_travel_time.0
    );
    let file = BufWriter::new(File::create(outpath.clone()).unwrap());
    bincode::serialize_into(file, &nodes_to_neighbouring_nodes).unwrap();
    println!("Serialised {}", outpath);
}


