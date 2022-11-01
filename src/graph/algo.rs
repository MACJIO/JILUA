use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use crate::graph::Graph;

// constant value for undefined dominator
const UNDEFINED: usize = ::std::usize::MAX;

struct DominatorTree {
    /// Control flow graph entry node index
    root: u32,
    /// `HashMap<a, b>` where a, b node indexes => `b idom a`
    dominators: HashMap<u32, u32>,
}

impl DominatorTree {
    fn root(&self) -> u32 { self.root }

    /// Returns immediate dominator for passed node index
    pub fn idom(&self, node: u32) -> Option<u32> {
        if node == self.root {
            None
        } else {
            self.dominators.get(&node).cloned()
        }
    }

    /// Return `DominatorsIter` for passed node index
    pub fn dominators(&self, node: u32) -> Option<DominatorsIter> {
        if self.dominators.contains_key(&node) {
            Some(DominatorsIter {
                dominators: self,
                node: Some(node),
            })
        } else {
            None
        }
    }
}

struct DominatorsIter<'a> {
    dominators: &'a DominatorTree,
    node: Option<u32>,
}

impl<'a> Iterator for DominatorsIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.node.take();
        if let Some(next) = next {
            self.node = self.dominators.idom(next);
        }
        next
    }
}


// filtering function for intersection
fn intersect(dominators: &[usize], mut finger1: usize, mut finger2: usize) -> usize {
    loop {
        match finger1.cmp(&finger2) {
            Ordering::Less => finger1 = dominators[finger1],
            Ordering::Greater => finger2 = dominators[finger2],
            Ordering::Equal => return finger1,
        }
    }
}


fn dominator_tree<N: Clone, E: Clone>(graph: &Graph<N, E>, root: u32) -> DominatorTree {
    // visit graph in dfs postorder and collect predecessors for every visited node
    let mut dfs = graph.dfs_post_order_visitor(root);

    let mut post_order = vec![];
    let mut predecessor_sets = HashMap::new();

    while let Some(node_idx) = dfs.next(&graph) {
        post_order.push(node_idx);

        for edge in graph.outputs(node_idx) {
            let successor = graph.edge_to(edge);

            predecessor_sets
                .entry(successor)
                .or_insert_with(HashSet::new)
                .insert(node_idx);
        }
    }

    println!("POST ORDER: {:?}", post_order);

    let length = post_order.len();

    debug_assert!(post_order.last() == Some(&root));

    let node_to_post_order_idx: HashMap<_, _> = post_order
        .iter()
        .enumerate()
        .map(|(idx, &node)| (node, idx))
        .collect();

    // predecessor sets to postorder index as Vec
    let predecessor_sets_to_post_order_index: Vec<Vec<_>> = post_order
        .iter()
        .map(|node| {
            predecessor_sets
                .remove(node)
                .map(|predecessors| {
                    predecessors
                        .into_iter()
                        .map(|p| *node_to_post_order_idx.get(&p).unwrap())
                        .collect()
                })
                .unwrap_or_else(Vec::new)
        })
        .collect();

    let mut dominators = vec![UNDEFINED; length];
    dominators[length - 1] = length - 1;

    let mut changed = true;
    while changed {
        changed = false;

        // iterate over post order in reverser
        for idx in (0..length - 1).rev() {
            debug_assert!(post_order[idx] != root);

            let new_idom_idx = {
                let mut predecessors = predecessor_sets_to_post_order_index[idx]
                    .iter()
                    .filter(|&&p| dominators[p] != UNDEFINED);
                let new_idom_idx = predecessors.next().expect(
                    "Because the root is initialized to dominate itself, and is the \
                     first node in every path, there must exist a predecessor to this \
                     node that also has a dominator",
                );
                predecessors.fold(*new_idom_idx, |new_idom_idx, &predecessor_idx| {
                    intersect(&dominators, new_idom_idx, predecessor_idx)
                })
            };

            debug_assert!(new_idom_idx < length);

            if new_idom_idx != dominators[idx] {
                dominators[idx] = new_idom_idx;
                changed = true;
            }
        }
    }

    debug_assert!(!dominators.iter().any(|&dom| dom == UNDEFINED));

    // back up actual node indexes

    DominatorTree {
        root,
        dominators: dominators
            .into_iter()
            .enumerate()
            .map(|(idx, dom_idx)| (post_order[idx], post_order[dom_idx]))
            .collect(),
    }
}
