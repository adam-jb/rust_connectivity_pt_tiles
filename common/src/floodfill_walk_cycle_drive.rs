use std::collections::BinaryHeap;
use crate::priority_queue::PriorityQueueItem;
use crate::shared::{Cost, NodeID, Angle, LinkID,Score, Multiplier, NodeWalkCyclingCar, OriginDestinationPair, FloodfillWalkCyclingCarOutput, SubpurposeScore};
use std::collections::HashSet;
use std::cmp::Ordering;
use typed_index_collections::TiVec;

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

pub fn get_scores_and_od_pairs(
                travel_time_relationships: &[Multiplier],  // travel_time_relationships: &[i32],
                node_values_2d: &TiVec<NodeID, Vec<SubpurposeScore>>,    // sparse_node_values: &Vec<Vec<[i32;2]>>,
                graph_walk: &TiVec<NodeID, NodeWalkCyclingCar>,  // graph_walk: &Vec<SmallVec<[EdgeWalk; 4]>>,
                time_costs_turn: &[Cost; 4],    //&[u16; 4],
                start_node_id: NodeID,
                seconds_walk_to_start_node: Cost,
                target_destinations_vector: &[Node], //&[u32],
                time_limit_seconds: Cost,    
            ) -> FloodfillWalkCyclingCarOutput {

    // initialise values
    let mut subpurpose_scores = [Score(0.0); 32];
    let subpurpose_purpose_lookup = initialise_subpurpose_purpose_lookup();
    let score_multipliers = initialise_score_multiplers();
        
    let mut queue: BinaryHeap<PriorityQueueItem<Cost, NodeID, Angle, LinkID>> = BinaryHeap::new();
    queue.push(PriorityQueueItem {
        cost: seconds_walk_to_start_node,
        node: start_node_id,
        angle_arrived_from: Angle(0),
        link_arrived_from: LinkID(99_999_999),
    });
         
    // storing for outputs
    let mut subpurpose_scores = [Score(0.0); 32];
    let mut od_pairs: Vec<OriginDestinationPair>;

    let mut iters: u32 = 0;
    let mut links_visited = HashSet::new();
    let mut nodes_visited: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();

    // make boolean vec to quickly check if a node is a target node for OD pair finding
    let mut target_destinations_binary_vec: TiVec<NodeID, bool> = vec![false; graph_walk.len()].into();                
    for node_id in target_destinations_vector.into_iter() {
        target_destinations_binary_vec[node_id] = true;
    }

    // catch where start node is over an hour from centroid
    if seconds_walk_to_start_node >= Cost(3600) {
        return (
            FloodfillWalkCyclingCarOutput{
                start_node_id,
                seconds_walk_to_start_node,
                iters,
                purpose_scores,
                od_pairs,
        });
    }
                
    // declared here to widen scoep (ie, the availability)
    let mut time_turn_previous_node: Cost;
                
    while let Some(current) = queue.pop() {
        
        // Optional speed improvement: see if links_visited can be stored as a boolean vector rather than a hashset
        if links_visited.contains(&current.link_arrived_from) {
            continue
        }
        links_visited.insert(current.link_arrived_from);
        
        // so long as this is the first time a link is taken, we add the link; a node can be reached multiple times: once for each link
        for edge in &graph_walk[(current.node.0 as usize)] {
            
            let time_turn_previous_node = get_cost_of_turn(
                angle_leaving_node_from: edge.angle_leaving_node_from,
                angle_arrived_from: current.angle_arrived_from,
                time_costs_turn, 
            );
            
            let new_cost = Cost(current.cost.0 + edge.cost.0 + time_turn_previous_node);
            
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
            od_pairs.push(OriginDestinationPair{
                NodeID: current.node
                Cost: current.cost
            })
        }

        add_to_subpurpose_scores_for_node_reached(
              subpurpose_scores,
              node_values_2d,
              subpurpose_purpose_lookup,
              travel_time_relationships,
        )
        
    }
                
    let purpose_scores = calculate_purpose_scores_from_subpurpose_scores(
        subpurpose_scores,
        subpurpose_purpose_lookup,
        score_multipliers,
    )
    
    FloodfillWalkCyclingCarOutput{
        start_node_id,
        seconds_walk_to_start_node,
        iters,
        purpose_scores,
        od_pairs,
    }

}
