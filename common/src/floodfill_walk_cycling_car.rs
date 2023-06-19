use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::cmp::Ordering;
use typed_index_collections::TiVec;

use crate::structs::{Cost, NodeID, Angle, LinkID,Score, Multiplier, NodeWalkCyclingCar, FloodfillOutputOriginDestinationPair, SubpurposeScore, PURPOSES_COUNT, SUBPURPOSES_COUNT};
use crate::floodfill_funcs::{initialise_score_multiplers, initialise_subpurpose_purpose_lookup, calculate_purpose_scores_from_subpurpose_scores, add_to_subpurpose_scores_for_node_reached, get_cost_of_turn};


/// Use with `BinaryHeap`. Since it's a max-heap, reverse the comparison to get the smallest cost
/// first.
#[derive(PartialEq, Eq, Clone)]
pub struct PriorityQueueItem<K, V, A, L> {
    pub cost: K,
    pub node: V,
    pub angle_arrived_from: A,
    pub link_arrived_from: L,
}

impl<K: Ord, V: Ord, A: Ord, L: Ord> PartialOrd for PriorityQueueItem<K, V, A, L> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Ord, A: Ord, L: Ord> Ord for PriorityQueueItem<K, V, A, L> {
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
            ) -> FloodfillOutputOriginDestinationPair {

    // initialise values
    let mut subpurpose_scores = [Score(0.0); SUBPURPOSES_COUNT];
    let subpurpose_purpose_lookup = initialise_subpurpose_purpose_lookup();
    let score_multipliers = initialise_score_multiplers(&mode);
        
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, Angle, LinkID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: seconds_walk_to_start_node,
        node: start_node_id,
        angle_arrived_from: Angle(0),
        link_arrived_from: LinkID(99_999_999),
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

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= Cost(3600) {
        let purpose_scores = [Score(0.0); PURPOSES_COUNT];
        return
            FloodfillOutputOriginDestinationPair{
                start_node_id,
                seconds_walk_to_start_node,
                purpose_scores,
                od_pairs_found,
                iters,
        };
    }
                
    // declared here to widen scoep (ie, the availability). May be able to delete this (Adam 17th May)
    //let mut time_turn_previous_node: Cost;
                
    while let Some(current) = queue.pop() {
        
        // Optional speed improvement: see if links_visited can be stored as a boolean vector rather than a hashset
        if links_visited.contains(&current.link_arrived_from) {
            continue
        }
        links_visited.insert(current.link_arrived_from);
        
        // so long as this is the first time a link is taken, we add the link; a node can be reached multiple times: once for each link
        for edge in graph_walk[current.node].edges.iter() {
            
            let time_turn_previous_node = get_cost_of_turn(
                edge.angle_leaving_node_from,
                current.angle_arrived_from,
                time_costs_turn, 
            );
            
            let new_cost = current.cost + edge.cost + time_turn_previous_node;
            
            if new_cost < time_limit_seconds {
                queue.push(PriorityQueueItem {
                    cost: new_cost,
                    node: edge.to,
                    angle_arrived_from: edge.angle_arrived_from,
                    link_arrived_from: edge.link_arrived_from,
                });
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
    
    FloodfillOutputOriginDestinationPair{
        start_node_id,
        seconds_walk_to_start_node,
        purpose_scores,
        od_pairs_found,
        iters,
    }

}
