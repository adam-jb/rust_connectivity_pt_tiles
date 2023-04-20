use std::cmp::Ordering;

/// Use with `BinaryHeap`. Since it's a max-heap, reverse the comparison to get the smallest cost
/// first.
#[derive(PartialEq, Eq, Clone)]
pub struct PriorityQueueItem<K, V, NT> {
    pub cost: K,
    pub value: V,
    pub nodes_taken: NT,
}

impl<K: Ord, V: Ord, NT: Ord> PartialOrd for PriorityQueueItem<K, V, NT> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V: Ord, NT: Ord> Ord for PriorityQueueItem<K, V, NT> {
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
