use crate::floodfill::get_travel_times;
use crate::read_files::{
    read_files_parallel_excluding_node_values,
};
use rayon::prelude::*;
use fs_err::File;
use std::io::BufWriter;
use std::time::Instant;
use std::sync::Mutex;

use crate::shared::{Cost, NodeID};

pub fn make_and_serialise_nodes_within_120s(year: i32) {
    
    // read in graph_walk and graph_pt
    println!("Begun make_and_serialise_nodes_within_120s");
    
    let (graph_walk, graph_pt, node_values_padding_row_count) =
        read_files_parallel_excluding_node_values(year);
    
    let start_time = Instant::now();
    let iter_count = Mutex::new(0);
    let indices = (0..graph_walk.len()).collect::<Vec<_>>();
    
    let results: Vec<(u32, Vec<u32>, Vec<u16>, Vec<Vec<u32>>)> = indices
        .par_iter()
        .enumerate()
        .map(|(count, i)| {
            let mut iter_count = iter_count.lock().unwrap();
            *iter_count += 1;
            if *iter_count % 1000 == 0 {
                let elapsed = start_time.elapsed();
                println!("Iteration: {}, Time elapsed: {:?}", iter_count, elapsed);
            }
            get_travel_times(
                &graph_walk,
                &graph_pt,
                NodeID(*i as u32),
                28800,
                Cost(0 as u16),
                true,
                120,
            )
        })
        .collect();
    
    // write the neighbouring nodes to a vector
    let mut nodes_to_neighbouring_nodes: Vec<Vec<u32>> = vec![vec![]; graph_walk.len()];
    for res in results {
        let ix = res.0;
        nodes_to_neighbouring_nodes[ix as usize] = res.1;  // res.1 is Vec<u32> of all nodes reached
    }
    
    let file = BufWriter::new(File::create("serialised_data/nodes_to_neighbouring_nodes.bin").unwrap());
    bincode::serialize_into(file, &nodes_to_neighbouring_nodes).unwrap();
    println!("Serialised serialised_data/nodes_to_neighbouring_nodes.bin");
}

    

