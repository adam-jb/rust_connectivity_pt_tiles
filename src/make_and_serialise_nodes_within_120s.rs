use fs_err::File;
use rayon::prelude::*;
use std::io::BufWriter;
use typed_index_collections::TiVec;


use crate::shared::{NodeID, Cost, SecondsPastMidnight, NodeWalk, NodePT, FloodfillOutput};
use crate::floodfill::get_travel_times;


// For ~10m walking nodes, takes ~90 mins to get all nearby nodes in 120s with 8 core machine; 128gb RAM was enough and 32gb wasnt
pub fn make_and_serialise_nodes_within_120s(graph_walk: Vec<NodeWalk>, graph_pt: Vec<NodePT>) {
    println!("Begun make_and_serialise_nodes_within_120s");

    // Convert graphs to TiVec to be indexed by NodeID values
    let graph_walk: TiVec<NodeID, NodeWalk> = TiVec::from(graph_walk);
    let graph_pt: TiVec<NodeID, NodePT> = TiVec::from(graph_pt);
    
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

    // write the neighbouring nodes to a vector
    let mut nodes_to_neighbouring_nodes: Vec<Vec<NodeID>> =
        vec![vec![]; graph_walk.len()];
    for res in results {
        let mut nodes_reached = Vec::new();
        for destination in res.destinations_reached {
            nodes_reached.push(destination.node);
        }
        nodes_to_neighbouring_nodes[res.start_node_id.0] = nodes_reached;
    }

    let file =
        BufWriter::new(File::create("serialised_data/nodes_to_neighbouring_nodes.bin").unwrap());
    bincode::serialize_into(file, &nodes_to_neighbouring_nodes).unwrap();
    println!("Serialised serialised_data/nodes_to_neighbouring_nodes.bin");
}
