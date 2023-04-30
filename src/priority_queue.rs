use std::cmp::Ordering;

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
        self.value.cmp(&other.value)
    }
}
