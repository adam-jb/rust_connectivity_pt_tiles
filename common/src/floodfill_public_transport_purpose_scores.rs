use crate::structs::{
    Cost, FloodfillOutputOriginDestinationPair, Multiplier, NodeID, NodeRoute,
    NodeWalk, Score, SecondsPastMidnight, SubpurposeScore,
};
use crate::floodfill_funcs::{initialise_score_multiplers, initialise_subpurpose_purpose_lookup, calculate_purpose_scores_from_subpurpose_scores, 
    add_to_subpurpose_scores_for_node_reached};

use std::collections::{BinaryHeap};
use typed_index_collections::TiVec;
use std::cmp::Ordering;

// ****** Spec BinaryHeap
/// Use with `BinaryHeap`. Since it's a max-heap, reverse the comparison to get the smallest cost
/// first.
#[derive(PartialEq, Eq, Clone)]
pub struct PriorityQueueItem<K, V> {
    pub cost: K,
    pub node: V,
}

impl<K: Ord, V: Ord> PartialOrd for PriorityQueueItem<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Ord> Ord for PriorityQueueItem<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = other.cost.cmp(&self.cost);
        if ord != Ordering::Equal {
            return ord;
        }
        // The tie-breaker is arbitrary, based on the value. Here it's NodeID, which is guaranteed
        // to differ for the one place this is used, so it's safe
        self.node.cmp(&other.node)
    }
}
// ***** BinaryHeap specc'ed

pub fn floodfill_public_transport_purpose_scores(
    graph_walk: &TiVec<NodeID, NodeWalk>,
    graph_routes: &TiVec<NodeID, NodeRoute>,
    start_node_id: NodeID,
    trip_start_seconds: SecondsPastMidnight,
    seconds_walk_to_start_node: Cost,
    walk_only: bool,
    time_limit: Cost,
    node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,
    travel_time_relationships: &[Multiplier],
    destination_nodes: &Vec<NodeID>,
) -> FloodfillOutputOriginDestinationPair {
    
    let mut iters: usize = 0;
    
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID>> = BinaryHeap::new();
    queue.push( PriorityQueueItem{
        cost: seconds_walk_to_start_node,
        node: start_node_id,
    });
    
    let target_destinations = vec![false; graph_walk.len()];
    let mut target_destinations: TiVec<NodeID, bool> = TiVec::from(target_destinations);
    for node_id in destination_nodes.into_iter() {
        target_destinations[*node_id] = true;
    }
    
    let mut subpurpose_scores = [Score(0.0); 32];
    let subpurpose_purpose_lookup = initialise_subpurpose_purpose_lookup();
    let score_multipliers = initialise_score_multiplers();
    let mut nodes_visited: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();
    let mut od_pairs_found: Vec<[usize;2]> = vec![];

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= time_limit {
        let purpose_scores = [Score(0.0); 5];
        return FloodfillOutputOriginDestinationPair {
            start_node_id,
            seconds_walk_to_start_node,
            purpose_scores,
            od_pairs_found,
            iters,
        };
    }

    while let Some(current) = queue.pop() {
        
        if nodes_visited[current.node] {
            continue;
        }
        nodes_visited[current.node] = true;

        if target_destinations[current.node] {
            od_pairs_found.push([current.cost.0,current.node.0]);
        }
        
        // get scores
        add_to_subpurpose_scores_for_node_reached(
            &mut subpurpose_scores, 
            &node_values_2d,
            &subpurpose_purpose_lookup,
            &travel_time_relationships,
            current.cost.0,
            current.node,
        );

        // Finding adjacent walk nodes
        for edge in &graph_walk[current.node].edges {
            let new_cost = current.cost + edge.cost;
            if new_cost < time_limit {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    node: edge.to,
                });
            }
        }

        // Find next PT route if there is one
        if !walk_only {
            if graph_walk[current.node].has_pt {
                take_next_pt_route(
                    &graph_routes,
                    current.cost,
                    &mut queue,
                    time_limit,
                    trip_start_seconds,
                    current.node,
                );
            }
        }
        iters += 1;
    }
    
    // get purpose level scores
    let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
        &subpurpose_scores,
        &subpurpose_purpose_lookup,
        &score_multipliers,
    );

    FloodfillOutputOriginDestinationPair {
        start_node_id,
        seconds_walk_to_start_node,
        purpose_scores,
        od_pairs_found,
        iters,
    }
}

fn take_next_pt_route(
    graph_routes: &TiVec<NodeID, NodeRoute>,
    time_so_far: Cost,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID>>,
    time_limit: Cost,
    trip_start_seconds: SecondsPastMidnight,
    current_node: NodeID,
) {
    let time_of_arrival_current_node = trip_start_seconds.add(&time_so_far);

    // find time next service leaves
    let mut found_next_service = false;
    let mut journey_time_to_next_node = Cost(0);
    let mut next_leaving_time = SecondsPastMidnight(0);

    // Could try: test switching from scanning search to binary search
    // See 'Binary search timetable' under Rust in Notion (Adam's notes, April 2023)
    for edge in &graph_routes[current_node].timetable {
        if time_of_arrival_current_node <= edge.leavetime {
            next_leaving_time = edge.leavetime;
            journey_time_to_next_node = edge.cost;
            found_next_service = true;
            break;
        }
    }

    // add to queue
    if found_next_service {
        
        // wait_time_this_stop is Cost; the difference between two SecondsPastMidnight objects
        let wait_time_this_stop = (next_leaving_time - time_of_arrival_current_node).into();
        let time_since_start_next_stop_arrival =
            time_so_far + journey_time_to_next_node + wait_time_this_stop;

        if time_since_start_next_stop_arrival < time_limit {
            let destination_node = graph_routes[current_node].next_stop_node;

            queue.push(PriorityQueueItem {
                cost: time_since_start_next_stop_arrival,
                node: destination_node,
            });


        };
    }
}

