use crate::priority_queue::PriorityQueueItem;
use crate::shared::{
    Cost, DestinationReached, FinalOutput, FloodfillOutput, Multiplier, NodeID, NodeRoute, NodeScore,
    NodeWalk, Score, SecondsPastMidnight, SubpurposeScore, TOP_CLUSTERS_COUNT,
};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;
use typed_index_collections::TiVec;
use std::cmp::Ordering;

// ****** Spec BinaryHeap
/// Use with `BinaryHeap`. Since it's a max-heap, reverse the comparison to get the smallest cost
/// first.
#[derive(PartialEq, Eq, Clone)]
pub struct PriorityQueueItem<K, V, NT, IT, NM> {
    pub cost: K,
    pub node: V,
    pub previous_node: NT,
    pub previous_node_iters_taken: IT,
    pub arrived_at_node_by_pt: NM,
}

impl<K: Ord, V: Ord, NT: Ord, IT: Ord, NM: Ord> PartialOrd for PriorityQueueItem<K, V, NT, IT, NM> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Ord, NT: Ord, IT: Ord, NM: Ord> Ord for PriorityQueueItem<K, V, NT, IT, NM> {
    fn cmp(&self, other: &Self) -> Ordering {
        // let ord = self.cost.cmp(&other.cost);    // subing this line for the line below reverses the ordering by cost so highest come first
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


pub fn floodfill_public_transport(
    graph_walk: &TiVec<NodeID, NodeWalk>,
    graph_pt: &TiVec<NodeID, NodeRoute>,
    start_node_id: NodeID,
    trip_start_seconds: SecondsPastMidnight,
    seconds_walk_to_start_node: Cost,
    walk_only: bool,
    time_limit: Cost,
    store_od_pairs: bool,
    store_route_trace: bool,
    node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,
    travel_time_relationships: &[Multiplier],
    find_scores: bool,
) -> FloodfillOutput {
    
    let previous_node = start_node_id;
    let mut iters_count: usize = 0;

    // Notable change (Adam 11th May): changed PriorityQueueItem to accept 'unit' primitive type so dont have to pass things around if not needed
    //let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, NodeID, usize, u8>> =
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, (), (), ()>> =
        BinaryHeap::new();
        node: start_node_id,
        cost: seconds_walk_to_start_node,
        previous_node: previous_node,
        previous_node_iters_taken: iters_count,
        arrived_at_node_by_pt: 0,
    });
    
    let mut subpurpose_scores = [Score(0.0); 32];
    let subpurpose_purpose_lookup = initialise_subpurpose_purpose_lookup();
    let score_multipliers = initialise_score_multiplers();
    let mut nodes_visited: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();
    let mut destinations_reached: Vec<DestinationReached> = vec![];

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= Cost(3600) {
        return FloodfillOutput {
            start_node_id,
            seconds_walk_to_start_node,
            purpose_scores,
            destinations_reached,
        };
    }

    while let Some(current) = queue.pop() {
        
        if nodes_visited[current.node] {
            continue;
        }
        nodes_visited[current.node] = true;
        
        if find_scores {
            add_to_subpurpose_scores_for_node_reached(
                  subpurpose_scores,
                  node_values_2d,
                  subpurpose_purpose_lookup,
                  travel_time_relationships: &[Multiplier],
            )
        }

        // First destination reached is to itself: this is fine as we later ignore first val in destinations_reached
        if store_od_pairs {
            destinations_reached.push(DestinationReached {
                cost: current.cost,
                node: current.node,
                previous_node: current.previous_node,
                previous_node_iters_taken: current.previous_node_iters_taken,
                arrived_at_node_by_pt: current.arrived_at_node_by_pt,
            });
        }

        // Finding adjacent walk nodes
        for edge in &graph_walk[current.node].node_connections {
            let new_cost = current.cost + edge.cost;
            if new_cost < time_limit {
                
                if store_route_trace {
                    queue.push(PriorityQueueItem {
                        cost: new_cost,
                        node: edge.to,
                        previous_node: current.node,
                        previous_node_iters_taken: iters_count,
                        arrived_at_node_by_pt: 0,
                    });
                } else {
                     queue.push(PriorityQueueItem {
                        cost: new_cost,
                        node: edge.to,
                        ..           // all other values are default: 'unit'
                    });   
                }
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
                    store_route_trace,
                );
            }
        }
        iters_count += 1;
    }
    
    let mut purpose_scores = [Score(0.0); 5];
    if find_scores {
        let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
            subpurpose_scores,
            subpurpose_purpose_lookup,
            score_multipliers,
        )
    }

    FloodfillOutput {
        start_node_id,
        seconds_walk_to_start_node,
        purpose_scores,
        destinations_reached,
    }
}

fn take_next_pt_route(
    graph_routes: &TiVec<NodeID, NodeRoute>,
    time_so_far: Cost,
    queue: &mut BinaryHeap<PriorityQueueItem<Cost, NodeID, (), (), ()>>, //BinaryHeap<PriorityQueueItem<Cost, NodeID, NodeID, usize, u8>>,
    time_limit: Cost,
    trip_start_seconds: SecondsPastMidnight,
    current_node: NodeID,
    iters_count: usize,
    store_route_trace: bool,
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
        
        // wait_time_this_stop is Cost; the difference between two SecondsPastMidnight objects
        let wait_time_this_stop = (next_leaving_time - time_of_arrival_current_node).into();
        let time_since_start_next_stop_arrival =
            time_so_far + journey_time_to_next_node + wait_time_this_stop;

        if time_since_start_next_stop_arrival < time_limit {
            let destination_node = graph_pt[current_node].next_stop_node;

            if store_route_trace {
                queue.push(PriorityQueueItem {
                    cost: time_since_start_next_stop_arrival,
                    node: destination_node,
                    previous_node: current.node,
                    previous_node_iters_taken: iters_count,
                    arrived_at_node_by_pt: 1,
                });
            } else {
                 queue.push(PriorityQueueItem {
                    cost: time_since_start_next_stop_arrival,
                    node: destination_node,
                    ..           // all other values are default: 'unit'
                });   
            }
            
            
        };
    }
}

