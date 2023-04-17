use crate::floodfill::get_travel_times;
use crate::read_files::{
    read_files_parallel_excluding_node_values,
};
use rayon::prelude::*;
use fs_err::File;
use std::io::BufWriter;
use std::sync::Arc;

use crate::shared::{Cost, NodeID};

pub fn make_and_serialise_nodes_within_120s(year: i32) {
    
    println!("Begun make_and_serialise_nodes_within_120s");
    // For ~10m walking nodes, takes ~90 mins to get all nearby nodes in 120s with 8 core machine; 128gb RAM was enough and 32gb wasnt
    
    let (graph_walk, graph_pt, _node_values_padding_row_count) =
        read_files_parallel_excluding_node_values(year);
    
    let arc_graph_walk = Arc::new(graph_walk);
    let arc_graph_pt = Arc::new(graph_pt);
    
    let indices = (0..arc_graph_walk.len()).collect::<Vec<_>>();
    println!("Number of iters to do: {}", arc_graph_walk.len());
    
    let results: Vec<(u32, Vec<u32>, Vec<u16>, Vec<Vec<u32>>)> = indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &arc_graph_walk,
                &arc_graph_pt,
                NodeID(*i as u32),
                28800,
                Cost(0 as u16),
                true,
                120,
            )
        })
        .collect();
    println!("Floodfill done for all nodes in graph_walk");
    
    // write the neighbouring nodes to a vector
    let mut nodes_to_neighbouring_nodes: Vec<Vec<u32>> = vec![vec![]; arc_graph_walk.len()];
    for res in results {
        let ix = res.0;
        nodes_to_neighbouring_nodes[ix as usize] = res.1;  // res.1 is Vec<u32> of all nodes reached
    }
    
    let file = BufWriter::new(File::create("serialised_data/nodes_to_neighbouring_nodes.bin").unwrap());
    bincode::serialize_into(file, &nodes_to_neighbouring_nodes).unwrap();
    println!("Serialised serialised_data/nodes_to_neighbouring_nodes.bin");
}

    

