use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::cmp::Ordering;
use typed_index_collections::TiVec;

use crate::structs::{Cost, NodeID, Angle, LinkID,Score, Multiplier, NodeWalkCyclingCar, FloodfillOutputOriginDestinationPair, SubpurposeScore, PURPOSES_COUNT, SUBPURPOSES_COUNT, PreviousIterAndCurrentNodeId};
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
                travel_time_relationships: &[Multiplier],  // travel_time_relationships: &[i32],
                node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,    // sparse_node_values: &Vec<Vec<[i32;2]>>,
                graph_walk: &TiVec<NodeID, NodeWalkCyclingCar>,  // graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
                time_costs_turn: &[Cost; 4],    //&[u16; 4],
                start_node_id: NodeID,
                seconds_walk_to_start_node: Cost,
                target_destinations_vector: &[NodeID], //&[u32],
                time_limit_seconds: Cost,   
                mode: &str,
                track_pt_nodes_reached: bool,
                seconds_reclaimed_when_pt_stop_reached: usize,
                target_node: NodeID,
                car_nodes_is_closest_to_pt: &TiVec<NodeID, bool>,
            ) -> FloodfillOutputOriginDestinationPair {
    
    // initialise values
    let mut subpurpose_scores = [Score(0.0); SUBPURPOSES_COUNT];
    let subpurpose_purpose_lookup = initialise_subpurpose_purpose_lookup();
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
    let mut target_destinations_binary_vec: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();                
    for node_id in target_destinations_vector.into_iter() {
        target_destinations_binary_vec[*node_id] = true;
    }
                
    // vec of previous iters reached. To make sequence 
    let mut previous_iters_and_current_node_ids: Vec<PreviousIterAndCurrentNodeId> = vec![];
    previous_iters_and_current_node_ids.push(PreviousIterAndCurrentNodeId{
        previous_iter: 0,
        current_node_id: start_node_id,
    });

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= Cost(3600) {
        let purpose_scores = [Score(0.0); PURPOSES_COUNT];
        let pt_nodes_reached_sequence: Vec<NodeID> = vec![];
        return
            FloodfillOutputOriginDestinationPair{
                start_node_id,
                seconds_walk_to_start_node,
                purpose_scores,
                od_pairs_found,
                iters,
                pt_nodes_reached_sequence,  // no pt nodes found en route as simulation never begins
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
                let mut pt_nodes_reached_sequence: Vec<NodeID> = vec![];
                let mut previous_iter = current.previous_node_reached_iter;
                
                while previous_iter > 0 {
                    //println!("previous_iter: {:?}", previous_iter);
                    
                    let next_node_id_in_seq = previous_iters_and_current_node_ids[previous_iter].current_node_id;
                    pt_nodes_reached_sequence.push(next_node_id_in_seq);
                    previous_iter = previous_iters_and_current_node_ids[previous_iter].previous_iter;
                    
                    /*
                    // For debug: bear in mind this is the sequence of PT stops passed through, so the nodes
                    // are unlikely to be adjacent
                    println!("next_node_id_in_seq: {:?}", next_node_id_in_seq);
                    for edge in &graph_walk[next_node_id_in_seq].edges {
                        println!("One next edge {:?}", edge.to);
                    }
                    */
                    
                }
                println!("previous_iter after while loop ends: {:?}", previous_iter);
                
                let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
                    &subpurpose_scores,
                    &subpurpose_purpose_lookup,
                    &score_multipliers,
                );

                return FloodfillOutputOriginDestinationPair{
                    start_node_id,
                    seconds_walk_to_start_node,
                    purpose_scores,
                    od_pairs_found,
                    iters,
                    pt_nodes_reached_sequence,
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
            
            if track_pt_nodes_reached {
                // If is a PT node, take seconds_reclaimed_when_pt_stop_reached from new_cost
                if car_nodes_is_closest_to_pt[edge.to] {
                    new_cost -= Cost(seconds_reclaimed_when_pt_stop_reached);
                }
            }
            
            if new_cost < time_limit_seconds {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    node: edge.to,
                    angle_arrived_from: edge.angle_arrived_from,
                    link_arrived_from: edge.link_arrived_from,
                    previous_node_reached_iter: iters,
                });
                
                previous_iters_and_current_node_ids.push(PreviousIterAndCurrentNodeId{
                    previous_iter: iters,
                    current_node_id: edge.to,
                })
            }
        }
        
        iters += 1;
        
        // we only add scroes and od pairs if this node has been reached before: a node may be reached via multiple links
        if nodes_visited[current.node] {
            continue;
        }
        nodes_visited[current.node] = true;
        
        if target_destinations_binary_vec[current.node] {
            od_pairs_found.push([current.cost.0,current.node.0]);
        }

        add_to_subpurpose_scores_for_node_reached(
              &mut subpurpose_scores,
              node_values_2d,
              &subpurpose_purpose_lookup,
              travel_time_relationships,
              current.cost.0,
              current.node,
        )
        
    }
                
    let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
        &subpurpose_scores,
        &subpurpose_purpose_lookup,
        &score_multipliers,
    );
    
    // if target_node hasn't been found don't record the sequence
    let pt_nodes_reached_sequence: Vec<NodeID> = vec![];
    
    FloodfillOutputOriginDestinationPair{
        start_node_id,
        seconds_walk_to_start_node,
        purpose_scores,
        od_pairs_found,
        iters,
        pt_nodes_reached_sequence,
    }

}
