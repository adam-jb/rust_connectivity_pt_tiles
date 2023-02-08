use std::time::Instant;
use std::collections::{BinaryHeap, HashSet};

use crate::priority_queue::PriorityQueueItem;
use crate::shared::{NodeID, Cost, GraphWalk, GraphPT};

pub fn floodfill(
    (
        graph_walk,
        start,
        node_values_1d,
        travel_time_relationships,
        subpurpose_purpose_lookup,
        graph_pt,
        trip_start_seconds,
        init_travel_time,
    ): (
        &GraphWalk,
        NodeID,
        &Vec<i32>, //&Vec<Vec<i32>>,
        &Vec<Vec<i32>>,
        &[i8; 32],
        &GraphPT,
        i32,
        Cost,
    ),

) -> (i32, [i64; 32]) {

    let time_limit: Cost = Cost(3600);
    let subpurposes_count: usize = 32 as usize;
    let now = Instant::now();

    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: init_travel_time,
        value: start,
    });
    let mut nodes_visited = HashSet::new();
    let mut total_iters = 0;

    let mut scores: [i64; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    while let Some(current) = queue.pop() {
        if nodes_visited.contains(&current.value) {
            continue;
        }
        if current.cost > time_limit {
            continue;
        }

        nodes_visited.insert(current.value);

        // if the node id is under 40m, then it will have an associated value
        if current.value.0 < 40_000_000 {
            get_scores(
                current.value.0,
                &node_values_1d,
                current.cost.0,
                travel_time_relationships,
                subpurpose_purpose_lookup,
                subpurposes_count,
                &mut scores,
            );
        }

        // Finding adjacent walk nodes
        // skip 1st edge as it has info on whether node also has a PT service
        for edge in &graph_walk.edges_per_node[&(current.value.0 as usize)][1..] {
            queue.push(PriorityQueueItem {
                cost: Cost(current.cost.0 + edge.cost.0),
                value: edge.to,
            });
        }

        // if node has a timetable associated with it: the first value in the first 'edge'
        // will be 1 if it does, and 0 if it doesn't
        if graph_walk.edges_per_node[&(current.value.0 as usize)][0].cost == Cost(1) {
            get_pt_connections(
                &graph_pt,
                current.cost.0,
                &mut queue,
                time_limit,
                trip_start_seconds,
                &current.value,
            );
        }

        total_iters += 1;
    }
    println!("total_iters: {}\t{:?}", total_iters, now.elapsed());

    return (total_iters, scores);
}

fn get_scores(
    node_id: u32,
    node_values_1d: &Vec<i32>,
    time_so_far: u16,
    travel_time_relationships: &Vec<Vec<i32>>,
    subpurpose_purpose_lookup: &[i8; 32],
    subpurposes_count: usize,
    scores: &mut [i64; 32],
    //scores: &mut ArrayVec<i64, 32>,
) {
    // to subset node_values_1d
    let start_pos = node_id * 32;

    // 32 subpurposes
    for i in 0..subpurposes_count {
        let ix_purpose = subpurpose_purpose_lookup[(i as usize)];
        let multiplier = travel_time_relationships[ix_purpose as usize][time_so_far as usize];

        // this line could be faster, eg if node_values_1d was an array
        scores[i] += (node_values_1d[(start_pos as usize) + i] * multiplier) as i64;
    }
}

fn get_pt_connections(
    graph_pt: &GraphPT,
    time_so_far: u16,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: i32,
    current_node: &NodeID,
) {
    // find time node is arrived at in seconds past midnight
    let time_of_arrival_current_node = trip_start_seconds as u32 + time_so_far as u32;

    // find time next service leaves
    let mut found_next_service = 0;
    let mut journey_time: u32 = 0;
    let mut next_leaving_time = 0;
    for edge in &graph_pt.edges_per_node[&(current_node.0 as usize)][1..] {
        if time_of_arrival_current_node <= edge.cost.0 as u32 {
            next_leaving_time = edge.cost.0;
            journey_time = edge.leavetime.0 as u32;
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
            //// Notice this uses 'leavingTime' as first 'edge' for each node stores ID
            //// of next node: this is legacy from our matrix-based approach in python
            let destination_node = &graph_pt.edges_per_node[&(current_node.0 as usize)][0]
                .leavetime
                .0;

            queue.push(PriorityQueueItem {
                cost: Cost(arrival_time_next_stop as u16),
                value: NodeID(*destination_node as u32),
            });
        };
    }
}

