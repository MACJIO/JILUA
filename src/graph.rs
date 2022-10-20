use std::borrow::Borrow;
use crate::op::Op;
use crate::resolver::BranchKind;
use std::collections::btree_map::Iter;
use std::collections::BTreeMap;

#[derive(Debug)]
struct Node<N> {
    weight: N,
    next_outgoing_edge: Option<u32>,
    next_incoming_edge: Option<u32>,
}

impl<N: Clone> Clone for Node<N> {
    fn clone(&self) -> Self {
        Node {
            weight: self.weight.clone(),
            next_incoming_edge: self.next_incoming_edge,
            next_outgoing_edge: self.next_outgoing_edge,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.weight = source.weight.clone();
        self.next_incoming_edge = source.next_incoming_edge;
        self.next_outgoing_edge = source.next_outgoing_edge;
    }
}

#[derive(Debug)]
struct Edge<E> {
    weight: E,
    from: u32,
    to: u32,
    next_outgoing_edge: Option<u32>,
    next_incoming_edge: Option<u32>,
}

impl<E> Edge<E> {
    #[inline(always)]
    pub fn weight(&self) -> &E {
        &self.weight
    }
}

impl<E: Clone> Clone for Edge<E> {
    fn clone(&self) -> Self {
        Edge {
            weight: self.weight.clone(),
            from: self.from,
            to: self.to,
            next_incoming_edge: self.next_incoming_edge,
            next_outgoing_edge: self.next_outgoing_edge,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.weight = source.weight.clone();
        self.from = source.from;
        self.to = source.to;
        self.next_incoming_edge = source.next_incoming_edge;
        self.next_outgoing_edge = source.next_outgoing_edge;
    }
}

pub struct Outputs<'graph, E> {
    edges: &'graph [Edge<E>],
    next: Option<u32>,
}

impl<'graph, E> Iterator for Outputs<'graph, E> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        if let Some(idx) = next {
            self.next = self.edges[idx as usize].next_outgoing_edge
        }
        next
    }
}

pub struct Inputs<'graph, E> {
    edges: &'graph [Edge<E>],
    next: Option<u32>,
}

impl<'graph, E> Iterator for Inputs<'graph, E> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;
        if let Some(idx) = next {
            self.next = self.edges[idx as usize].next_incoming_edge
        }
        next
    }
}

#[derive(Debug)]
pub struct Graph<N, E> {
    nodes: BTreeMap<u32, Node<N>>,
    edges: Vec<Edge<E>>,
}

impl<N, E: Clone> Graph<N, E> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn structure_copy<T: Default>(&self) -> Graph<T, E> {
        let mut nodes: BTreeMap<u32, Node<T>> = BTreeMap::new();

        for (&idx, node) in &self.nodes {
            nodes.insert(
                idx,
                Node {
                    weight: T::default(),
                    next_outgoing_edge: node.next_outgoing_edge,
                    next_incoming_edge: node.next_incoming_edge,
                },
            );
        }

        Graph {
            nodes,
            edges: self.edges.clone(),
        }
    }

    pub fn iter_node_weights(&self) -> impl Iterator<Item=(u32, &N)> {
        self.nodes.iter().map(|(k, v)| (*k, &v.weight))
    }

    #[inline(always)]
    fn edge_mut(&mut self, index: u32) -> Option<&mut Edge<E>> {
        self.edges.get_mut(index as usize)
    }

    #[inline(always)]
    pub fn edge_weight(&self, index: u32) -> Option<&E> {
        self.edges.get(index as usize).map(|e| e.weight())
    }

    #[inline(always)]
    fn node_mut(&mut self, index: u32) -> Option<&mut Node<N>> {
        self.nodes.get_mut(&index)
    }

    #[inline(always)]
    pub fn try_prev_node(&self, index: u32) -> Option<u32> {
        self.nodes
            .range(..=index)
            .next_back()
            .map_or(None, |(&index, _)| Some(index))
    }

    #[inline(always)]
    pub fn try_next_node(&self, index: u32) -> Option<u32> {
        self.nodes
            .range(index..)
            .next()
            .map_or(None, |(&index, _)| Some(index))
    }

    #[inline(always)]
    fn node(&self, index: u32) -> Option<&Node<N>> {
        self.nodes.get(&index)
    }

    #[inline(always)]
    pub fn exists(&self, index: u32) -> bool {
        self.nodes.get(&index).is_some()
    }

    #[inline(always)]
    pub fn node_weight(&self, index: u32) -> Option<&N> {
        self.nodes.get(&index).map(|n| &n.weight)
    }

    #[inline(always)]
    pub fn node_weight_mut(&mut self, index: u32) -> Option<&mut N> {
        self.nodes.get_mut(&index).map(|n| &mut n.weight)
    }

    pub fn add_edge(&mut self, weight: E, from: u32, to: u32) -> u32 {
        let index = self.edges.len() as u32;

        let from_node = self.nodes.get_mut(&from).unwrap();
        let next_outgoing_edge = from_node.next_outgoing_edge.replace(index);

        let to_node = self.nodes.get_mut(&to).unwrap();
        let next_incoming_edge = to_node.next_incoming_edge.replace(index);

        self.edges.push(Edge {
            weight,
            from,
            to,
            next_incoming_edge,
            next_outgoing_edge,
        });

        index
    }

    pub fn add_node(&mut self, index: u32, weight: N) -> Option<u32> {
        self.nodes
            .insert(
                index,
                Node {
                    weight,
                    next_outgoing_edge: None,
                    next_incoming_edge: None,
                },
            )
            .map_or(Some(index), |_| None)
    }

    pub fn next_outgoing_node_idx(&self, node_idx: u32) -> Option<u32> {
        let node = self.node(node_idx).unwrap();
        if let Some(edge_idx) = node.next_outgoing_edge {
            let edge = self.edges.get(edge_idx as usize).unwrap();

            return Some(edge.to)
        }

        None
    }

    pub fn split_node<F: FnOnce(&mut N) -> N>(
        &mut self,
        index: u32,
        new_index: u32,
        splitter: F,
    ) -> u32 {
        // get node with passed index
        let node = self.node_mut(index).unwrap();
        // take outgoing edge from this node
        let mut next_outgoing_edge = node.next_outgoing_edge.take();

        // get new node weight
        let new_weight = splitter(&mut node.weight);
        // add node with new weight
        self.add_node(new_index, new_weight).unwrap();
        // get just added node object
        let new_node = self.node_mut(new_index).unwrap();
        // add previous outgoing edge to new node
        new_node.next_outgoing_edge = next_outgoing_edge;

        // iterate throw all outgoing edges
        while let Some(edge_index) = next_outgoing_edge {
            // get edge with current index
            let edge = self.edge_mut(edge_index).unwrap();
            // add new node to them
            edge.from = new_index;
            // iterate
            next_outgoing_edge = edge.next_outgoing_edge;
        }

        new_index
    }

    #[inline(always)]
    pub fn outputs(&self, node_idx: u32) -> Outputs<E> {
        Outputs {
            edges: &self.edges,
            next: self.node(node_idx).unwrap().next_outgoing_edge,
        }
    }

    #[inline(always)]
    pub fn inputs(&self, node_idx: u32) -> Inputs<E> {
        Inputs {
            edges: &self.edges,
            next: self.node(node_idx).unwrap().next_incoming_edge,
        }
    }
}

#[derive(Clone, Debug)]
/// Visit nodes in a depth-first-search (DFS) emitting nodes in postorder
pub struct DfsPostOrder {
    /// Vec of nodes to visit
    pub stack: Vec<u32>,
    /// Set of visited node indices
    pub discovered: HashSet<u32>,
    /// Set of finished node indices
    pub finished: HashSet<u32>,
}

impl DfsPostOrder {
    pub fn new<N, E: Clone>(graph: &Graph<N, E>, start: u32) -> Self <> {
        let mut dfs = DfsPostOrder {
            stack: vec![],
            discovered: HashSet::with_capacity(graph.node_count()),
            finished: HashSet::with_capacity(graph.node_count()),
        };

        dfs.move_to(start);
        dfs
    }

    /// Keep the discovered and finished map, but clear the visit stack and restart
    /// the dfs from a particular node.
    pub fn move_to(&mut self, start: u32) {
        self.stack.clear();
        self.stack.push(start);
    }

    pub fn next<N, E: Clone>(&mut self, graph: &Graph<N, E>) -> Option<u32> {
        while let Some(&nx) = self.stack.last() {
            // check if already discovered
            if self.discovered.insert(nx) {
                // add neighbors to stack in not discovered
                for edge in graph.outputs(nx) {
                    let node_id = graph.edge_to(edge);

                    if !self.discovered.insert(node_id) {
                        self.stack.push(node_id)
                    }
                }
            } else {
                // pop from "to visit" stack if already discovered
                self.stack.pop();

                // try to visit node
                if self.finished.insert(nx) {
                    return Some(nx);
                }
            }
        }
        None
    }
}


