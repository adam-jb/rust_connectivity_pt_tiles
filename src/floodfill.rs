use std::collections::{HashMap, BinaryHeap};
use crate::priority_queue::PriorityQueueItem;
use crate::shared::{Cost, EdgePT, EdgeWalk, NodeID, FloatBinHeap};
use smallvec::SmallVec;
use rand::Rng;
use std::sync::Arc;

// returns unique i32 based on sequence of two integers
fn cantor_pairing(x: u32, y: u32) -> u32 {
    ((x + y) * (x + y + 1)) / 2 + y
}

pub fn get_travel_times(
    graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    start: NodeID,
    trip_start_seconds: i32,
    init_travel_time: Cost,
    walk_only: bool,
    max_travel_time: u16,
) -> (u32, Vec<u32>, Vec<u16>, Vec<Vec<u32>>) {
        
    let time_limit: Cost = Cost(max_travel_time);
    
    let start_nodes_taken_sequence: Vec<u32> = vec![start.0];

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, Vec<u32>>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: init_travel_time,
        value: start,
        nodes_taken: start_nodes_taken_sequence.to_vec(),
    });
    
    let mut nodes_visited = vec![false; graph_walk.len()];
    //let mut nodes_visited = HashSet::new();
    let mut destination_ids: Vec<u32> = vec![];
    let mut destination_travel_times: Vec<u16> = vec![];
    let mut nodes_visited_sequences: Vec<Vec<u32>> = vec![];


    // catch where start node is over an hour from centroid
    if init_travel_time >= Cost(3600) {
        return (
            start.0,
            destination_ids,
            destination_travel_times,
            nodes_visited_sequences,
        );
    }

    while let Some(mut current) = queue.pop() {
        
        if nodes_visited[current.value.0 as usize] {
            continue;
        }
        nodes_visited[current.value.0 as usize] = true;

        destination_ids.push(current.value.0);
        destination_travel_times.push(current.cost.0);
        nodes_visited_sequences.push(current.nodes_taken.to_vec());
        
        current.nodes_taken.push(current.value.0);
        
        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        for edge in &graph_walk[(current.value.0 as usize)][1..] {
            let new_cost = Cost(current.cost.0 + edge.cost.0);
            if new_cost < time_limit {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    value: edge.to,
                    nodes_taken: current.nodes_taken.to_vec(),
                });
            }
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        if !walk_only {
            if graph_walk[(current.value.0 as usize)][0].cost == Cost(1) {
                get_pt_connections(
                    &graph_pt,
                    current.cost.0,
                    &mut queue,
                    time_limit,
                    trip_start_seconds,
                    &current.value,
                    &current.nodes_taken,
                );
            }
        }

    }
        
    return (
        start.0,
        destination_ids,
        destination_travel_times,
        nodes_visited_sequences,
    );
}


fn get_pt_connections(
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    time_so_far: u16,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID, Vec<u32>>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: &NodeID,
    current_nodes_taken: &Vec<u32>,
) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u16 = 0;
    let mut next_leaving_time = 0;

    for edge in &graph_pt[(current_node.0 as usize)][1..] {
        if time_of_arrival_current_node <= edge.leavetime.0 as u32 {
            next_leaving_time = edge.leavetime.0;
            journey_time = edge.cost.0;
            found_next_service = 1;
            break;
        }
    }

    // add to queue
    if found_next_service == 1 {
        let wait_time_this_stop = next_leaving_time as u32 - time_of_arrival_current_node;
        let arrival_time_next_stop =
            time_so_far as u32 + wait_time_this_stop as u32 + journey_time as u32;

        if arrival_time_next_stop < time_limit.0 as u32 {
            // Notice this uses 'leavingTime' from first 'edge' for the ID
            // of next node: this is legacy from our matrix-based approach in python
            let destination_node = &graph_pt[(current_node.0 as usize)][0].leavetime.0;

            queue.push(PriorityQueueItem {
                cost: Cost(arrival_time_next_stop as u16),
                value: NodeID(*destination_node as u32),
                nodes_taken: current_nodes_taken.to_vec(),
            });
        };
    }
}



pub fn get_all_scores_links_and_key_destinations(
    travel_times: &(u32, Vec<u32>, Vec<u16>, Vec<Vec<u32>>), // nodeID, destination node IDs, travel times to destinations, sequence of nodes taken to each node reached
    node_values_2d: &Vec<Vec<[i32; 2]>>, //&Vec<i32>,
    travel_time_relationships: &[i32], //&Vec<i32>,
    subpurpose_purpose_lookup: &[i8; 32],
    count_original_nodes: u32,
    node_values_padding_row_count: u32,
    nodes_to_neighbouring_nodes: &Arc<Vec<Vec<u32>>>,
) -> (i32, u32, [f64; 5], HashMap<u32, [f64; 5]>, HashMap<u32, [u32; 2]>, [HashMap<FloatBinHeap, u32>; 5]) {
    
    // Got this from 'subpurpose_purpose_lookup_integer_list.json' in connectivity-processing-files
    let subpurpose_purpose_lookup_integer: [u8; 32] = [2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 1, 2, 2, 1, 2, 4, 3, 3, 1, 3, 2, 3, 1, 2, 3, 3, 3, 1, 2, 1];
    
    // Get this from score_multipler_by_subpurpose_id_{mode_simpler}.json in connectivity-processing-files
    let score_multipler: [f64; 32] = [0.00831415115437604, 0.009586382150013575, 0.00902817799219063, 0.008461272650878338, 0.008889733875203568, 0.008921736222033676, 0.022264233988222335, 0.008314147237807904, 0.010321099162180719, 0.00850878998927169, 0.008314150893271383, 0.009256043337142108, 0.008338366940103991, 0.009181584368558857, 0.008455731022360958, 0.009124946989519319, 0.008332774189837317, 0.046128804773287346, 0.009503140563990153, 0.01198700845708387, 0.009781270599036206, 0.00832427047935188, 0.008843645925786448, 0.008531419360132648, 0.009034318952510731, 0.008829954505680167, 0.011168757794031156, 0.017255946829128663, 0.008374145360142223, 0.008578983146921768, 0.008467735739894604, 0.012110456385386992];

    // based on subpurpose_integers_to_ignore.json; they include ['Residential', 'Motor sports', 'Allotment']
    let subpurposes_to_ignore: [i8; 3] = [0, 10, 14];
    
    let subpurposes_count: usize = 32;
    let count_nodes_no_value = node_values_padding_row_count / 32;
    
    
    let mut subpurpose_scores: [f64; 32] = [0.0; 32];
    
    let start = travel_times.0;
    let destination_ids = &travel_times.1;
    let destination_travel_times = &travel_times.2;
    let node_sequences = &travel_times.3;
    let mut node_values_contributed_each_purpose_hashmap: HashMap<u32, [f64; 5]> = HashMap::new();

    
    for i in 0..destination_ids.len() {
        let current_node = destination_ids[i];
        let current_cost = destination_travel_times[i];
        
        // if the node id is not a p2 node (ie, above count_nodes_no_value), then it will have an associated value
        if current_node <= count_original_nodes && current_node >= count_nodes_no_value {
            
            let mut purpose_scores_this_node: [f64; 5] = [0.0; 5];
            
            for subpurpose_score_pair in node_values_2d[current_node as usize].iter() {
                
                // store scores for each subpurpose for this node
                let subpurpose_ix = subpurpose_score_pair[0];
                let vec_start_pos_this_purpose = (subpurpose_purpose_lookup[subpurpose_ix as usize] as i32) * 3601;
                let multiplier = travel_time_relationships[(vec_start_pos_this_purpose + current_cost as i32) as usize];
                let score_to_add = (subpurpose_score_pair[1] as f64) * (multiplier as f64);
                subpurpose_scores[subpurpose_ix as usize] += score_to_add;
                
                // To get purpose level contribution to scores for each node: used for finding key destinations
                if !subpurposes_to_ignore.contains(&(subpurpose_ix as i8)) {
                    let purpose_ix = subpurpose_purpose_lookup_integer[subpurpose_ix as usize];
                    purpose_scores_this_node[purpose_ix as usize] += score_to_add;
                }
            }
            
            node_values_contributed_each_purpose_hashmap.insert(current_node, purpose_scores_this_node);
        }

    }
    
    // Loops through each subpurpose, scaling them and getting the purpose level scores for the start node
    let mut overall_purpose_scores: [f64; 5] = [0.0; 5];
    for subpurpose_ix in 0..subpurpose_scores.len() {
        
        // skip if subpurpose in ['Residential', 'Motor sports', 'Allotment']
        if subpurposes_to_ignore.contains(&(subpurpose_ix as i8)) {
            continue;
        }
        
        // Apply score_multipler to get purpose level scores for this start node. This does what s39 would do in python: faster to do it here as so many tiles
        // getting log of score for this subpurpose
        let mut subpurpose_score = ((subpurpose_scores[subpurpose_ix] as f64) * score_multipler[subpurpose_ix]).ln();
        
        // make negative values zero: this corrects for an effect of using log()
        if subpurpose_score < 0.0 {
            subpurpose_score = 0.0;
        }
        
        // add to purpose level scores
        let purpose_ix = subpurpose_purpose_lookup_integer[subpurpose_ix];
        overall_purpose_scores[purpose_ix as usize] += subpurpose_score;
        
    }
    
    
    // Get contributions to scores: to tell us the relative importance of each link
    // For each sequence, find the scores which were reached: this involves looking to the final node in the sequence
    let mut link_score_contributions: HashMap<u32, [f64; 5]>  = HashMap::new();
    let mut link_start_end_nodes: HashMap<u32, [u32; 2]> = HashMap::new();
    for sequence in node_sequences.iter() {
        
        let end_node_purpose_scores = node_values_contributed_each_purpose_hashmap[sequence.last().unwrap()];  // without .unwrap() you will get an Option<&u32> type that you can use to check if the vector is empty or not
        
        // loop through each link in the sequence, as defined by the pair of nodes at each end of the link: these are i32 values
        for i in 1..sequence.len() {
            
            let node_start_of_link = sequence[i-1];
            let node_end_of_link = sequence[i];
            let unique_link_id = cantor_pairing(node_start_of_link, node_end_of_link);
            
            // add to scores_impacted_by_link for each purpose for said link
            if link_score_contributions.contains_key(&unique_link_id) {
                
                // might be able to use get_mut on the dict to speed up the lines below
                //*my_map.get_mut("a").unwrap() += 10;
                let mut purpose_scores = link_score_contributions[&unique_link_id];
                for k in 0..5 {
                    purpose_scores[k] += end_node_purpose_scores[k];
                }
                // line below overwrites existing value mapping to unique_link_id
                link_score_contributions.insert(unique_link_id, purpose_scores);
                
            } else {
                link_score_contributions.insert(unique_link_id, end_node_purpose_scores);
                link_start_end_nodes.insert(unique_link_id, [node_start_of_link, node_end_of_link]);
            }
            
        }
        
    }
    
    
    // get key destinations for each purpose
    // for each purpose, 5 nodes with their ID and score contributed. Initialised with all zeros
    let mut top_nodes_by_purposes: [HashMap<FloatBinHeap, u32>; 5] = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];
    
    // and have a record of the lowest value for each purpose
    let mut lowest_values_in_top_by_purposes: [f64; 5] = [0.0; 5];
    
    // initialise vector of heaps for top 5 scores for each purpose. Wrap writes to heap in Reverse() so smallest values are at front of heap
    let mut top_5_scores_by_purpose_heaps: [BinaryHeap<FloatBinHeap>; 5] = [
        BinaryHeap::new(),
        BinaryHeap::new(),
        BinaryHeap::new(),
        BinaryHeap::new(),
        BinaryHeap::new(),
    ];
    
    for k in 0..5 {
        for z in 0..5 {
            top_5_scores_by_purpose_heaps[k].push(FloatBinHeap(0.0));
        }
    }
    
    let mut rng = rand::thread_rng();
    for node_reached_id in destination_ids {
        
        let near_nodes = nodes_to_neighbouring_nodes[*node_reached_id as usize].to_vec();
        let mut purpose_scores: [f64; 5] = [0.0; 5];
        
        // get total scores by purpose, of nodes within 120s of this node
        for neighbouring_node in near_nodes {
            let scores_one_node = &node_values_contributed_each_purpose_hashmap[&neighbouring_node];
            for k in 0..5 {
                purpose_scores[k] += scores_one_node[k];
            }
        }
        
        // Look through each of the purposes, and add to the top 5 if it qualifies for any of them
        for k in 0..5 {
            
            if purpose_scores[k] > lowest_values_in_top_by_purposes[k] {
                
                // to avoid hash collision, add random decimal to values when adding to top_nodes_by_purposes and lowest_values_in_top_by_purposes
                let new_score = purpose_scores[k] + rng.gen::<f64>();
                                
                // replace value in hashmap
                top_nodes_by_purposes[k].remove(&FloatBinHeap(lowest_values_in_top_by_purposes[k]));
                top_nodes_by_purposes[k].insert(FloatBinHeap(new_score), *node_reached_id);
                
                // update heap
                top_5_scores_by_purpose_heaps[k].pop();
                top_5_scores_by_purpose_heaps[k].push(FloatBinHeap(new_score));
                
                // find new lowest value in heap for following iterations // use .0 to extract f64 value from FloatBinHeap
                lowest_values_in_top_by_purposes[k] = top_5_scores_by_purpose_heaps[k].peek().unwrap().0;
                
            }
        }
    }
    
    
    // link_score_contributions: hashmap of total purpose-level scores trips across that link that fed into
    // link_start_end_nodes: hashmap of link ID to the nodes at either end of the link
    // top_nodes_by_purposes: array of 5 hashmaps of scores and node IDs
    return (
        travel_times.1.len() as i32,
        start,
        overall_purpose_scores,
        link_score_contributions,
        link_start_end_nodes,
        top_nodes_by_purposes,
    );

}

