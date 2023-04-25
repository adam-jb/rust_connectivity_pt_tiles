use crate::priority_queue::PriorityQueueItem;
use crate::shared::{Cost, EdgePT, EdgeWalk, FloodfillOutput, NodeID};
use smallvec::SmallVec;
use std::collections::{BinaryHeap};



pub fn get_travel_times_and_scores(
    graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    start: NodeID,
    trip_start_seconds: i32,
    init_travel_time: Cost,
    max_travel_time: u16,
    node_values_2d: &Vec<Vec<[i32; 2]>>,
    travel_time_relationships: &Vec<i32>,
    subpurpose_purpose_lookup: &[i8; 32],
    
) -> FloodfillOutput {
    let time_limit = Cost(max_travel_time);

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: init_travel_time,
        value: start,
    });

    let mut nodes_visited = vec![false; graph_walk.len()];

    // catch where start node is over an hour from centroid
    if init_travel_time >= Cost(3600) {
        let scaled_purpose_scores: [f64; 5] = [0.0; 5];
        return FloodfillOutput {
            start_node_id: start,
            init_travel_time: init_travel_time.0,
            scaled_purpose_scores,
        };
    }
    
    // Got this from 'subpurpose_purpose_lookup_integer_list.json' in connectivity-processing-files
    let subpurpose_purpose_lookup_integer: [u8; 32] = [
        2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 1, 2, 2, 1, 2, 4, 3, 3, 1, 3, 2, 3, 1, 2, 3, 3, 3, 1,
        2, 1,
    ];

    // Get this from score_multipler_by_subpurpose_id_{mode_simpler}.json in connectivity-processing-files
    // Used to get relative importance of each subpurpose when aggregating them to purpose level
    let score_multipler: [f64; 32] = [
        0.00831415115437604,
        0.009586382150013575,
        0.00902817799219063,
        0.008461272650878338,
        0.008889733875203568,
        0.008921736222033676,
        0.022264233988222335,
        0.008314147237807904,
        0.010321099162180719,
        0.00850878998927169,
        0.008314150893271383,
        0.009256043337142108,
        0.008338366940103991,
        0.009181584368558857,
        0.008455731022360958,
        0.009124946989519319,
        0.008332774189837317,
        0.046128804773287346,
        0.009503140563990153,
        0.01198700845708387,
        0.009781270599036206,
        0.00832427047935188,
        0.008843645925786448,
        0.008531419360132648,
        0.009034318952510731,
        0.008829954505680167,
        0.011168757794031156,
        0.017255946829128663,
        0.008374145360142223,
        0.008578983146921768,
        0.008467735739894604,
        0.012110456385386992,
    ];

    // based on subpurpose_integers_to_ignore.json; they include ['Residential', 'Motor sports', 'Allotment']
    let subpurposes_to_ignore: [i8; 3] = [0, 10, 14];
    let mut subpurpose_scores: [f64; 32] = [0.0; 32];

    while let Some(current) = queue.pop() {
        if nodes_visited[current.value.0 as usize] {
            continue;
        }
        nodes_visited[current.value.0 as usize] = true;
        
        
        // ***** store subpurpose scores for this node
        let mut purpose_scores_this_node: [f64; 5] = [0.0; 5];

        for subpurpose_score_pair in node_values_2d[current.value.0 as usize].iter() {

            let subpurpose_ix = subpurpose_score_pair[0];
            let vec_start_pos_this_purpose =
                (subpurpose_purpose_lookup[subpurpose_ix as usize] as i32) * 3601;
            let multiplier = travel_time_relationships
                [(vec_start_pos_this_purpose + current.cost.0 as i32) as usize];
            let score_to_add = (subpurpose_score_pair[1] as f64) * (multiplier as f64);
            subpurpose_scores[subpurpose_ix as usize] += score_to_add;

            // To get purpose level contribution to scores for each node: used for finding key destinations
            if !subpurposes_to_ignore.contains(&(subpurpose_ix as i8)) {
                let purpose_ix = subpurpose_purpose_lookup_integer[subpurpose_ix as usize];
                purpose_scores_this_node[purpose_ix as usize] += score_to_add;
            }
        }


        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        for edge in &graph_walk[current.value.0 as usize][1..] {
            let new_cost = Cost(current.cost.0 + edge.cost.0);
            if new_cost < time_limit {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    value: edge.to,
                });
            }
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        if graph_walk[current.value.0 as usize][0].cost == Cost(1) {
            get_pt_connections(
                &graph_pt,
                current.cost.0,
                &mut queue,
                time_limit,
                trip_start_seconds,
                current.value,
            );
        }
    }
    
    let mut scaled_purpose_scores: [f64; 5] = [0.0; 5];
    for subpurpose_ix in 0..subpurpose_scores.len() {
        
        // skip if subpurpose in ['Residential', 'Motor sports', 'Allotment']
        if subpurposes_to_ignore.contains(&(subpurpose_ix as i8)) {
            continue;
        }
        
        // Apply score_multipler to get purpose level scores for this start node. This does what s39 would do in python: faster to do it here as so many tiles
        // getting log of score for this subpurpose
        let mut subpurpose_score =
            ((subpurpose_scores[subpurpose_ix] as f64) * score_multipler[subpurpose_ix]).ln();

        // make negative values zero: this corrects for an effect of using log()
        if subpurpose_score < 0.0 {
            subpurpose_score = 0.0;
        }

        // add to purpose level scores
        let purpose_ix = subpurpose_purpose_lookup_integer[subpurpose_ix];
        scaled_purpose_scores[purpose_ix as usize] += subpurpose_score;
    }

    FloodfillOutput {
        start_node_id: start,
        init_travel_time: init_travel_time.0,
        scaled_purpose_scores,
    }
}



fn get_pt_connections(
    graph_pt: &Vec<SmallVec<[EdgePT; 4]>>,
    time_so_far: u16,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: NodeID,
) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u16 = 0;
    let mut next_leaving_time = 0;

    for edge in &graph_pt[current_node.0 as usize][1..] {
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
            // TODO The first row is magically node IDs, just trust it
            let destination_node = NodeID(graph_pt[(current_node.0 as usize)][0].leavetime.0);

            queue.push(PriorityQueueItem {
                cost: Cost(arrival_time_next_stop as u16),
                value: destination_node,
            });
        };
    }
}
