use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::cmp::Ordering;
use typed_index_collections::TiVec;

use crate::structs::{Cost, NodeID, Angle, LinkID,Score, Multiplier, NodeWalkCyclingCar, FloodfillOutputOriginDestinationPairWalkCyclingCar, SubpurposeScore, PURPOSES_COUNT, SUBPURPOSES_COUNT, PreviousIterAndCurrentNodeId, SubpurposeSmallMediumLargeCount};
use crate::floodfill_funcs::{initialise_score_multiplers, initialise_subpurpose_purpose_lookup, calculate_purpose_scores_from_subpurpose_scores, add_to_subpurpose_scores_for_node_reached, get_cost_of_turn};


/// Use with `BinaryHeap`. Since it's a max-heap, reverse the comparison to get the smallest cost
/// first.
#[derive(PartialEq, Eq, Clone)]
pub struct PriorityQueueItem<K, V, A, L, P> {
    pub cost: K,
    pub node: V,
    pub angle_arrived_from: A,
    pub link_arrived_from: L,
    pub previous_node_reached_iter: P,
}

impl<K: Ord, V: Ord, A: Ord, L: Ord, P: Ord> PartialOrd for PriorityQueueItem<K, V, A, L, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Telling rust to order the heap by cost
impl<K: Ord, V: Ord, A: Ord, L: Ord, P: Ord> Ord for PriorityQueueItem<K, V, A, L, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = other.cost.cmp(&self.cost);
        if ord != Ordering::Equal {
            return ord;
        }
        // The tie-breaker is arbitrary, based on the node
        self.node.cmp(&other.node)
    }
}

pub fn floodfill_walk_cycling_car(
                travel_time_relationships: &[Multiplier],
                node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,
                graph_walk: &TiVec<NodeID, NodeWalkCyclingCar>,
                time_costs_turn: &[Cost; 4],
                start_node_id: NodeID,
                seconds_walk_to_start_node: Cost,
                od_pair_destinations_vector: &[NodeID],  // if you want to find OD pairs, destinations from here
                time_limit_seconds: Cost,   
                mode: &str,
                track_pt_nodes_reached: bool,    // do we output nodes_reached_sequence and nodes_reached_time_travelled?
                seconds_reclaimed_when_pt_stop_reached: usize,
                target_node: NodeID,
                car_nodes_is_closest_to_pt: &TiVec<NodeID, bool>,
                small_medium_large_subpurpose_destinations: &TiVec<NodeID, Vec<SubpurposeSmallMediumLargeCount>>,
                count_destinations_at_intervals: bool,
                original_time_intervals_to_store_destination_counts: &Vec<Cost>,
            ) -> FloodfillOutputOriginDestinationPairWalkCyclingCar {
    
    // making a copy we can edit            
    let mut time_intervals_to_store_destination_counts = original_time_intervals_to_store_destination_counts.to_vec();
    
    // initialise values
    let mut subpurpose_scores = [Score(0.0); SUBPURPOSES_COUNT];
                
    // lookup between subpurpose idx and purpose (eg: primary school -> education)
    let subpurpose_purpose_lookup = initialise_subpurpose_purpose_lookup();
                
    // multiplier to scale score to account for average size of destination 
    let score_multipliers = initialise_score_multiplers(&mode);
    
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, Angle, LinkID, usize>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: seconds_walk_to_start_node,
        node: start_node_id,
        angle_arrived_from: Angle(0),
        link_arrived_from: LinkID(99_999_999),
        previous_node_reached_iter: 0,
    });
         
    // storing for outputs
    let mut od_pairs_found: Vec<[usize;2]> = Vec::new();
    
    let mut iters: usize = 0;
    let mut links_visited = HashSet::new();
    let mut nodes_visited: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();
    
    // make boolean vec to quickly check if a node is a target node for OD pair finding
    let mut od_pair_destinations_binary_vec: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();                
    for node_id in od_pair_destinations_vector.into_iter() {
        od_pair_destinations_binary_vec[*node_id] = true;
    }
                
    // vec of previous iters reached. To make sequence 
    let mut previous_iters_and_current_node_ids: Vec<PreviousIterAndCurrentNodeId> = vec![];
    previous_iters_and_current_node_ids.push(PreviousIterAndCurrentNodeId{
        previous_iter: 0,
        current_node_id: start_node_id,
        time_travelled: Cost(0),
    });
                
    // Make empty vector to store destination counts at the intervals specified in time_intervals_to_store_destination_counts
    let mut destinations_reached_at_time_intervals = Vec::new();
    
    // These are only populated if seconds_reclaimed_when_pt_stop_reached is true; otherwise they are returned to the user as empty
    let mut nodes_reached_sequence: Vec<NodeID> = vec![];
    let mut nodes_reached_time_travelled: Vec<Cost> = vec![];
                
    // This stores the total destinations reached, of each subpurpose and size banding
    let number_of_size_bands = 3; // set to 3 because 3 size bands: small, medium, large
    let mut destination_counts_small_medium_large: Vec<Vec<Score>> = vec![vec![Score(0.0); number_of_size_bands]; SUBPURPOSES_COUNT];

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= Cost(3600) {
        let purpose_scores = [Score(0.0); PURPOSES_COUNT];
        return
            FloodfillOutputOriginDestinationPairWalkCyclingCar{
                start_node_id,
                seconds_walk_to_start_node,
                purpose_scores,
                od_pairs_found,
                iters,
                nodes_reached_sequence, 
                nodes_reached_time_travelled,
                final_cost: seconds_walk_to_start_node,
                destinations_reached_at_time_intervals,
        };
    }
                 
    while let Some(current) = queue.pop() {
        
        // Optional speed improvement: see if links_visited can be stored as a boolean vector rather than a hashset
        if links_visited.contains(&current.link_arrived_from) {
            continue
        }
        links_visited.insert(current.link_arrived_from);
        
        // 
        if track_pt_nodes_reached {
            if current.node == target_node {
                
                // Work through the sequence of pt nodes reached, creating a vector of these
                let mut previous_iter = current.previous_node_reached_iter;
                
                while previous_iter > 0 {
                    
                    let next_node_id_in_seq = previous_iters_and_current_node_ids[previous_iter].current_node_id;
                    let next_node_time_travelled = previous_iters_and_current_node_ids[previous_iter].time_travelled;
                    nodes_reached_sequence.push(next_node_id_in_seq);
                    nodes_reached_time_travelled.push(next_node_time_travelled);
                    previous_iter = previous_iters_and_current_node_ids[previous_iter].previous_iter;          
                    
                }
                
                let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
                    &subpurpose_scores,
                    &subpurpose_purpose_lookup,
                    &score_multipliers,
                );
                
                println!("Final iters: {:?}", iters);
                println!("Final cost: {:?}", current.cost);

                return FloodfillOutputOriginDestinationPairWalkCyclingCar{
                    start_node_id,
                    seconds_walk_to_start_node,
                    purpose_scores,
                    od_pairs_found,
                    iters,
                    nodes_reached_sequence,
                    nodes_reached_time_travelled,
                    final_cost: current.cost,
                    destinations_reached_at_time_intervals,
                }
            }
        }
        
        // so long as this is the first time a link is taken, we add the link; a node can be reached multiple times: once for each link
        for edge in graph_walk[current.node].edges.iter() {
            
            let time_turn_previous_node = get_cost_of_turn(
                edge.angle_leaving_node_from,
                current.angle_arrived_from,
                time_costs_turn, 
            );
            
            let mut new_cost = current.cost + edge.cost + time_turn_previous_node;
            
            // TODO drop maybe
            if track_pt_nodes_reached {
                // If is a PT node, take seconds_reclaimed_when_pt_stop_reached from new_cost
                // Is this out of whack with previous_iters_and_current_node_ids ?
                if car_nodes_is_closest_to_pt[edge.to] {
                    new_cost -= Cost(seconds_reclaimed_when_pt_stop_reached);
                }
            }
            
            if new_cost < time_limit_seconds {
                
                println!("cost: {:?}", new_cost);
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    node: edge.to,
                    angle_arrived_from: edge.angle_arrived_from,
                    link_arrived_from: edge.link_arrived_from,
                    previous_node_reached_iter: iters,
                });
                
                // !! iters might not align
                previous_iters_and_current_node_ids.push(PreviousIterAndCurrentNodeId{
                    previous_iter: iters,
                    current_node_id: edge.to,
                    time_travelled: new_cost,
                });
                
            }
        }
        
        iters += 1;
        
        // we only add scores and od pairs if this node has not been reached before: a node may be reached via multiple links
        if nodes_visited[current.node] {
            continue;
        }
        nodes_visited[current.node] = true;
        
        
        if od_pair_destinations_binary_vec[current.node] {
            od_pairs_found.push([current.cost.0,current.node.0]);
        }

        add_to_subpurpose_scores_for_node_reached(
              &mut subpurpose_scores,
              node_values_2d,
              &subpurpose_purpose_lookup,
              travel_time_relationships,
              current.cost.0,
              current.node,
        );
        
        // Only bother counting destinations if the API payload requested it
        if count_destinations_at_intervals {
        
            // add to our destinations counter for each subpurpose        
            for destination in &small_medium_large_subpurpose_destinations[current.node] {
                destination_counts_small_medium_large[destination.subpurpose_ix][0] += destination.small_destinations_count;
                destination_counts_small_medium_large[destination.subpurpose_ix][1] += destination.medium_destinations_count;
                destination_counts_small_medium_large[destination.subpurpose_ix][2] += destination.large_destinations_count;
            }

            // Push when a threshold is crossed in distance travelled, based on current.cost
            // And remove the threshold from time_intervals_to_store_destination_counts, as we have reached it
            if time_intervals_to_store_destination_counts.len() > 0 {
                if current.cost >= time_intervals_to_store_destination_counts[0] {

                    destinations_reached_at_time_intervals.push(destination_counts_small_medium_large.to_vec());
                    time_intervals_to_store_destination_counts.remove(0);

                }
            }
        }
        
        
        
    }
                
    let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
        &subpurpose_scores,
        &subpurpose_purpose_lookup,
        &score_multipliers,
    );
                    
    FloodfillOutputOriginDestinationPairWalkCyclingCar{
        start_node_id,
        seconds_walk_to_start_node,
        purpose_scores,
        od_pairs_found,
        iters,
        nodes_reached_sequence,
        nodes_reached_time_travelled,
        final_cost: time_limit_seconds,
        destinations_reached_at_time_intervals,
    }

}
