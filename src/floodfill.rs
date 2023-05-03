use crate::priority_queue::PriorityQueueItem;
use crate::shared::{
    Cost, DestinationReached, FinalOutput, FloodfillOutput, Multiplier, NodeID, NodePT, NodeWalk,
    Score, SecondsPastMidnight, SubpurposeScore,
};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Instant;
use typed_index_collections::TiVec;
use std::sync::{Mutex};

pub fn get_travel_times(
    graph_walk: &TiVec<NodeID, NodeWalk>,
    graph_pt: &TiVec<NodeID, NodePT>,
    start_node_id: NodeID,
    trip_start_seconds: SecondsPastMidnight,
    seconds_walk_to_start_node: Cost,
    walk_only: bool,
    time_limit: Cost,
) -> FloodfillOutput {
    let previous_node = start_node_id;
    let mut iters_count: usize = 0;

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, NodeID, usize, u8>> =
        BinaryHeap::new();
    queue.push(PriorityQueueItem {
        node: start_node_id,
        cost: seconds_walk_to_start_node,
        previous_node: previous_node,
        previous_node_iters_taken: iters_count,
        arrived_at_node_by_pt: 0,
    });

    let mut nodes_visited: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();
    let mut destinations_reached: Vec<DestinationReached> = vec![];

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= Cost(3600) {
        return FloodfillOutput {
            start_node_id,
            seconds_walk_to_start_node,
            destinations_reached,
        };
    }

    while let Some(current) = queue.pop() {
        if nodes_visited[current.node] {
            continue;
        }
        nodes_visited[current.node] = true;

        // First destination reached is to itself: this is fine as we later ignore first val in destinations_reached
        destinations_reached.push(DestinationReached {
            cost: current.cost,
            node: current.node,
            previous_node: current.previous_node,
            previous_node_iters_taken: current.previous_node_iters_taken,
            arrived_at_node_by_pt: current.arrived_at_node_by_pt,
        });
        
        // Finding adjacent walk nodes
        for edge in &graph_walk[current.node].node_connections {
            let new_cost = current.cost + edge.cost;
            if new_cost < time_limit {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    node: edge.to,
                    previous_node: current.node,
                    previous_node_iters_taken: iters_count,
                    arrived_at_node_by_pt: 0,
                });
            }
        }

        // Find next PT route if there is one
        if !walk_only {
            if graph_walk[current.node].has_pt {
                take_next_pt_route(
                    &graph_pt,
                    current.cost,
                    &mut queue,
                    time_limit,
                    trip_start_seconds,
                    current.node,
                    iters_count,
                );
            }
        }
        iters_count += 1;
    }

    FloodfillOutput {
        start_node_id,
        seconds_walk_to_start_node,
        destinations_reached,
    }
}

fn take_next_pt_route(
    graph_pt: &TiVec<NodeID, NodePT>,
    time_so_far: Cost,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID, NodeID, usize, u8>>,
    time_limit: Cost,
    trip_start_seconds: SecondsPastMidnight,
    current_node: NodeID,
    iters_count: usize,
) {
    let time_of_arrival_current_node = trip_start_seconds.add(&time_so_far);

    // find time next service leaves
    let mut found_next_service = false;
    let mut journey_time_to_next_node = Cost(0);
    let mut next_leaving_time = SecondsPastMidnight(0);

    // Could try: test switching from scanning search to binary search
    // See 'Binary search timetable' under Rust in Notion (Adam's notes, April 2023)
    for edge in &graph_pt[current_node].timetable {
        if time_of_arrival_current_node <= edge.leavetime {
            next_leaving_time = edge.leavetime;
            journey_time_to_next_node = edge.cost;
            found_next_service = true;
            break;
        }
    }

    // add to queue
    if found_next_service {
        // wait_time_this_stop as Cost, as the difference between two SecondsPastMidnight objects
        let wait_time_this_stop = (next_leaving_time - time_of_arrival_current_node).into();
        let time_since_start_next_stop_arrival =
            time_so_far + journey_time_to_next_node + wait_time_this_stop;

        if time_since_start_next_stop_arrival < time_limit {
            let destination_node = graph_pt[current_node].next_stop_node;

            queue.push(PriorityQueueItem {
                cost: time_since_start_next_stop_arrival,
                node: destination_node,
                previous_node: current_node,
                previous_node_iters_taken: iters_count,
                arrived_at_node_by_pt: 1,
            });
        };
    }
}

pub fn get_all_scores_links_and_key_destinations(
    floodfill_output: &FloodfillOutput,
    node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,
    travel_time_relationships: &[Multiplier],
    subpurpose_purpose_lookup: &[usize; 32],
    nodes_to_neighbouring_nodes: &TiVec<NodeID, Vec<NodeID>>,
    rust_node_longlat_lookup: &TiVec<NodeID, [f64; 2]>,
    route_info: &TiVec<NodeID, HashMap<String, String>>, //&TiVec<NodeID, String>,
    mutex_sparse_node_values_contributed: &Mutex<TiVec<NodeID, [Score; 5]>>,
) -> FinalOutput {
    // Get this from score_multipler_by_subpurpose_id_{mode_simpler}.json in connectivity-processing-files
    // Used to get relative importance of each subpurpose when aggregating them to purpose level
    // This has subpurposes ['Residential', 'Motor sports', 'Allotment'] set to zero, which pertain to subpurpose indices of [0, 10, 14]
    let score_multipler: [Multiplier; 32] = [
        Multiplier(0.000000000000000), // set to 0
        Multiplier(0.009586382150013575),
        Multiplier(0.00902817799219063),
        Multiplier(0.008461272650878338),
        Multiplier(0.008889733875203568),
        Multiplier(0.008921736222033676),
        Multiplier(0.022264233988222335),
        Multiplier(0.008314147237807904),
        Multiplier(0.010321099162180719),
        Multiplier(0.00850878998927169),
        Multiplier(0.00000000000000), // set to 0
        Multiplier(0.009256043337142108),
        Multiplier(0.008338366940103991),
        Multiplier(0.009181584368558857),
        Multiplier(0.000000000000000), // set to 0
        Multiplier(0.009124946989519319),
        Multiplier(0.008332774189837317),
        Multiplier(0.046128804773287346),
        Multiplier(0.009503140563990153),
        Multiplier(0.01198700845708387),
        Multiplier(0.009781270599036206),
        Multiplier(0.00832427047935188),
        Multiplier(0.008843645925786448),
        Multiplier(0.008531419360132648),
        Multiplier(0.009034318952510731),
        Multiplier(0.008829954505680167),
        Multiplier(0.011168757794031156),
        Multiplier(0.017255946829128663),
        Multiplier(0.008374145360142223),
        Multiplier(0.008578983146921768),
        Multiplier(0.008467735739894604),
        Multiplier(0.012110456385386992),
    ];

    // subpurpose_scores includes the 3 subpurposes to ignore as postproc scripts are expecting them ['Residential', 'Motor sports', 'Allotment'] though they will always be 0
    let mut subpurpose_scores: [Score; 32] = [Score(0.0); 32];

    let start = floodfill_output.start_node_id;
    let seconds_walk_to_start_node = floodfill_output.seconds_walk_to_start_node;
    let destinations_reached = &floodfill_output.destinations_reached;

    // load 
    let mut sparse_node_values_contributed = mutex_sparse_node_values_contributed.lock().unwrap();
    
    
    let mut node_values_contributed_each_purpose_vec: Vec<[Score; 5]> = vec![];

    // ********* Get subpurpose level scores overall, and purpose level contribution of each individual node reached
    let mut now = Instant::now();

    for DestinationReached { node, cost, .. } in destinations_reached.iter() {
        let mut purpose_scores_this_node = [Score(0.0); 5];

        for SubpurposeScore {
            subpurpose_ix,
            subpurpose_score,
        } in node_values_2d[*node].iter()
        {
            // store scores for each subpurpose for this node
            let vec_start_pos_this_purpose = subpurpose_purpose_lookup[*subpurpose_ix] * 3601;
            let multiplier = travel_time_relationships[vec_start_pos_this_purpose + (cost.0)];
            let score_to_add = subpurpose_score.multiply(multiplier);
            subpurpose_scores[*subpurpose_ix] += score_to_add;

            // To get purpose level contribution to scores for each node: used for finding key destinations
            let purpose_ix = subpurpose_purpose_lookup[*subpurpose_ix];
            purpose_scores_this_node[purpose_ix] += score_to_add;
        }

        sparse_node_values_contributed[*node] = purpose_scores_this_node;
        node_values_contributed_each_purpose_vec.push(purpose_scores_this_node);
    }

    println!(
        "Getting destinations purpose_scores_this_node took {:?}",
        now.elapsed()
    );

    // **** Loops through each subpurpose, scaling them and getting the purpose level scores for the start node
    now = Instant::now();

    let mut overall_purpose_scores: [Score; 5] = [Score(0.0); 5];
    for subpurpose_ix in 0..subpurpose_scores.len() {
        // Apply score_multipler and apply logarithm to get subpurpose level scores
        let mut subpurpose_score = subpurpose_scores[subpurpose_ix]
            .multiply(score_multipler[subpurpose_ix])
            .ln();

        // make negative values zero: this corrects for an effect of using log()
        if subpurpose_score < Score(0.0) {
            subpurpose_score = Score(0.0);
        }

        // add to purpose level scores
        let purpose_ix = subpurpose_purpose_lookup[subpurpose_ix];
        overall_purpose_scores[purpose_ix] += subpurpose_score;
    }

    println!("Getting overall_purpose_scores took {:?}", now.elapsed());

    // ****** Overall scores obtained ******

    // ******* Get each link contributions to scores: tells us the relative importance of each link *******

    let now = Instant::now();

    // initialise link data to populate
    let mut link_score_contributions: Vec<[Score; 5]> =
        vec![[Score(0.0); 5]; destinations_reached.len()];
    let mut link_start_end_nodes_string: Vec<Vec<String>> = vec![];
    let mut link_is_pt: Vec<u8> = vec![];
    let mut link_route_details: Vec<HashMap<String, String>> = Vec::new();

    // Skip first node reached as this is the start node to itself
    for (
        node_reached_iteration,
        DestinationReached {
            node,
            previous_node,
            arrived_at_node_by_pt,
            ..
        },
    ) in destinations_reached.iter().skip(1).enumerate()
    {
        // copying iter as it gets changed during the loop below. This should be an implicit clone() without using *
        let mut link_ix = node_reached_iteration.clone();
        
        println!("New seq"); 

        // loop until all links taken to node reached have the score for this node added to their score contributions
        loop {
            for k in 0..5 {
                link_score_contributions[link_ix][k] +=
                    node_values_contributed_each_purpose_vec[node_reached_iteration][k];
            }

            if link_ix == 0 {
                break;
            }

            // get previous node iter in sequence to reach this node
            link_ix = destinations_reached[link_ix].previous_node_iters_taken;
            
            let previous_node_ix = destinations_reached[link_ix].previous_node;
            println!("{:?}", route_info[previous_node_ix]);
            
        }

        // add coords from previous node to this node
        let longlat_previous_node = rust_node_longlat_lookup[*previous_node];
        let longlat_node = rust_node_longlat_lookup[*node];

        // convert floats to strings and store
        link_start_end_nodes_string.push(vec![
            longlat_previous_node
                .iter()
                .map(|n| format!("{:.6}", n))
                .collect::<Vec<String>>()
                .join(","),
            longlat_node
                .iter()
                .map(|n| format!("{:.6}", n))
                .collect::<Vec<String>>()
                .join(","),
        ]);

        link_is_pt.push(*arrived_at_node_by_pt);

        if *arrived_at_node_by_pt == 1 {
            link_route_details.push(route_info[*previous_node].clone())
        } else {
            let empty_map: HashMap<String, String> = HashMap::new();
            link_route_details.push(empty_map);
        }
    }

    println!("Getting link contributions took {:?}", now.elapsed());

    // ****** Contributions to scores obtained ******

    // ****** Get top 3 clusters destinations for each purpose *******

    let now = Instant::now();

    // dicts of which of the 3 top 3 nodes, the nodes in the sets above correspond to keys in these hashmaps; each value will be the ID of one of the top 3 nodes
    let mut nearby_nodes_to_current_highest_node_hashmap: [HashMap<NodeID, NodeID>; 5] = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];

    // sets of all nodes which are close to those in the top 3 (eg: if there are 24 nodes within 120s of the 3 top nodes for business, those 24 node ids will be in the set corresponding to business)
    let mut nearby_nodes_top_3_scores_sets: [HashSet<NodeID>; 5] = [
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    ];

    // to log minimum scores for each purpose: this is the threshold to exceed to get into the running top 3
    let mut id_and_min_scores: [(NodeID, Score); 5] = [(NodeID(0), Score(0.0)); 5];

    let mut id_and_scores_top_3: [[(NodeID, Score); 3]; 5] = [[
        (NodeID(0), Score(0.0)),
        (NodeID(0), Score(0.0)),
        (NodeID(0), Score(0.0)),
    ]; 5];

    // Dicts of nodeID to adjacent nodes (each Dict will have 3 keys of node IDs, corresponding to vec of Node IDs in each cluster)
    let mut highest_nodes_hashmap_to_adjacent_nodes_vec: [HashMap<NodeID, Vec<NodeID>>; 5] = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];

    for DestinationReached { node, .. } in destinations_reached.iter() {
        //for node_reached_id in destination_ids {    // original
        let near_nodes = &nodes_to_neighbouring_nodes[*node];
        let mut purpose_scores = [Score(0.0); 5];

        // get total scores by purpose, of nodes within 120s of this node
        // node_values_contributed_each_purpose_hashmap tells you score contributed by each node
        for neighbouring_node in near_nodes {
            let scores_one_node = sparse_node_values_contributed[*neighbouring_node];
            for nth_purpose in 0..5 {
                purpose_scores[nth_purpose] += scores_one_node[nth_purpose];
            }
        }

        // Look through each of the purposes, and add to the top 3 if it qualifies for any of them
        // "Adjacent" here means: within 120s of that node via walking
        for nth_purpose in 0..5 {
            if purpose_scores[nth_purpose] >= id_and_min_scores[nth_purpose].1 {
                // test if node is an adjacent one
                let node_to_replace: NodeID;
                let is_in_adjacent: bool =
                    nearby_nodes_top_3_scores_sets[nth_purpose].contains(node);
                if is_in_adjacent {
                    node_to_replace =
                        nearby_nodes_to_current_highest_node_hashmap[nth_purpose][node];
                } else {
                    node_to_replace = id_and_min_scores[nth_purpose].0;
                }

                // find position of the node we want to replace
                let mut node_to_replace_ix: usize = 0;
                for i in 0..3 {
                    if id_and_scores_top_3[nth_purpose][i].0 == node_to_replace {
                        node_to_replace_ix = i;
                    }
                }

                // if node is adjacent to one of the top 3 nodes, and the cluster score of the adjacent node is above the new one, do nothing
                let mut do_nothing_as_existing_adjacent_score_larger: bool = false;
                if is_in_adjacent {
                    do_nothing_as_existing_adjacent_score_larger = purpose_scores[nth_purpose]
                        > id_and_scores_top_3[nth_purpose][node_to_replace_ix].1;
                }

                // If node is adjacent to one of the top 3 nodes AND the cluster score of the adjacent node is same as new node AND the new node has a higher score
                // than the current "reigning node", then we want the node with the higher score to become the new reigning node
                let mut do_nothing_as_existing_node_score_larger: bool = false;
                if !do_nothing_as_existing_adjacent_score_larger {
                    if purpose_scores[nth_purpose]
                        == id_and_scores_top_3[nth_purpose][node_to_replace_ix].1
                    {
                        let purpose_value_node_to_replace =
                            sparse_node_values_contributed[node_to_replace][nth_purpose];
                        let purpose_value_node_reached =
                            sparse_node_values_contributed[*node][nth_purpose];
                        if purpose_value_node_to_replace > purpose_value_node_reached {
                            do_nothing_as_existing_node_score_larger = true;
                        }
                    }
                }

                if do_nothing_as_existing_adjacent_score_larger
                    || do_nothing_as_existing_node_score_larger
                {
                    continue;
                }

                // Use highest_nodes_hashmap_to_adjacent_nodes_vec to find adjacent nodes to get rid of (and the node ID of itself)
                // Don't run this if node_to_replace is 0, as node_to_replace=0 is the initialised node ID
                if node_to_replace != NodeID(0) {
                    let vec_nodes_to_drop_from_set_and_dict =
                        &highest_nodes_hashmap_to_adjacent_nodes_vec[nth_purpose][&node_to_replace];
                    for node_id in vec_nodes_to_drop_from_set_and_dict {
                        nearby_nodes_to_current_highest_node_hashmap[nth_purpose].remove(&node_id);
                        nearby_nodes_top_3_scores_sets[nth_purpose].remove(&node_id);
                    }
                    highest_nodes_hashmap_to_adjacent_nodes_vec[nth_purpose]
                        .remove(&node_to_replace);
                }

                // overwrite current top 3: this is fine to do inplace as id_and_scores_top_3 isn't ordered
                id_and_scores_top_3[nth_purpose][node_to_replace_ix].0 = *node;
                id_and_scores_top_3[nth_purpose][node_to_replace_ix].1 =
                    purpose_scores[nth_purpose];

                // recalculate current minimum
                let mut current_minimum = Score(999_999_999_999_999_999.0);
                let mut current_min_ix: usize = 0;
                for i in 0..3 {
                    let current_score = id_and_scores_top_3[nth_purpose][i].1;
                    if current_score < current_minimum {
                        current_minimum = current_score;
                        current_min_ix = i;
                    }
                }

                let minimum_node_id = id_and_scores_top_3[nth_purpose][current_min_ix].0;
                id_and_min_scores[nth_purpose].0 = minimum_node_id;
                id_and_min_scores[nth_purpose].1 = current_minimum;

                // add new adjacaents to both hashmap and the set
                for node_id in near_nodes.to_vec() {
                    nearby_nodes_to_current_highest_node_hashmap[nth_purpose]
                        .insert(node_id, *node);
                    nearby_nodes_top_3_scores_sets[nth_purpose].insert(node_id);
                }

                // add new adjacent nodes to highest_nodes_hashmap_to_adjacent_nodes_vec
                highest_nodes_hashmap_to_adjacent_nodes_vec[nth_purpose]
                    .insert(*node, near_nodes.to_vec());
            }
        }
    }

    println!("Getting node clusters took {:?}", now.elapsed());

    // ******* Clusters obtained *******

    // **** Extract keys from each of the 5 of highest_nodes_hashmap_to_adjacent_nodes_vec
    let mut most_important_nodes_longlat: [[[f64; 2]; 3]; 5] = [[[0.0; 2]; 3]; 5];
    for i in 0..5 {
        for (inner_iter, rust_node_id) in highest_nodes_hashmap_to_adjacent_nodes_vec[i]
            .keys()
            .enumerate()
        {
            let node_longlat = rust_node_longlat_lookup[*rust_node_id];
            most_important_nodes_longlat[i][inner_iter] = node_longlat;
        }
    }
    
    // ****** Reset sparse_node_values_contributed for next query
    for DestinationReached { node, .. } in destinations_reached.iter() {
        sparse_node_values_contributed[*node] = [Score(0.0); 5];
    }
    
    
    FinalOutput {
        num_iterations: destinations_reached.len() as u32,
        start_node: start,
        score_per_purpose: overall_purpose_scores,
        per_link_score_per_purpose: link_score_contributions,
        link_coordinates: link_start_end_nodes_string,
        link_is_pt: link_is_pt,
        key_destinations_per_purpose: most_important_nodes_longlat,
        init_travel_time: seconds_walk_to_start_node,
        link_route_details: link_route_details,
    }
}
