use crate::floodfill::get_travel_times;
use crate::read_files::{
    read_files_parallel_excluding_node_values,
};
use rayon::prelude::*;
use fs_err::File;
use std::io::BufWriter;
use crate::shared::{Cost, NodeID};
use crate::get_time_of_day_index::get_time_of_day_index;



pub fn make_and_serialise_nodes_within_120s(year: i32) {
    
    // read in graph_walk and graph_pt
    println!("Begun make_and_serialise_nodes_within_120s");
    let time_of_day_ix = get_time_of_day_index(28800);
    
    let (graph_walk, graph_pt, node_values_padding_row_count) =
        read_files_parallel_excluding_node_values(year);
    
    let indices = (0..graph_walk.len()).collect::<Vec<_>>();
    
    let results: Vec<(u32, Vec<u32>, Vec<u16>, Vec<Vec<u32>>)> = indices
        .par_iter()
        .map(|i| {
            get_travel_times(
                &graph_walk,
                &graph_pt,
                NodeID(*i as u32),
                28800,
                Cost(0 as u16),
                true,
                200,
            )
        })
        .collect();
    
    // write the neighbouring nodes to a vector
    let mut nodes_to_neighbouring_nodes: Vec<Vec<u32>> = vec![vec![]; graph_walk.len()];
    for res in results {
        let ix = res.0;
        nodes_to_neighbouring_nodes[ix as usize] = res.1;
    }
    
    // look at all nodes: the old pre-parallelisation code
    /*
    for iter in 0..graph_walk.len() {
                  
        // start floodfill for 200s, ignoring PT routes, starting bang on the node (so no initial walking time Cost)
        start_node, destination_ids, destination_travel_times, nodes_visited_sequences = get_travel_times(
            &graph_walk,
            &graph_pt,
            NodeID(iter as u32),
            28800,
            Cost(0 as u16),
            true,
            200,
        )
        
        nodes_to_neighbouring_nodes.push(destination_ids);
        
        if (iter % 1000) == 0 {
            println!("make_and_serialise_nodes_within_120s() done iter {}", iter};
        }
    }
    */
    
    
    let file = BufWriter::new(File::create("serialised_data/nodes_to_neighbouring_nodes.bin").unwrap());
    bincode::serialize_into(file, &nodes_to_neighbouring_nodes).unwrap();
    println!("Serialised serialised_data/nodes_to_neighbouring_nodes.bin");
}
    
    

