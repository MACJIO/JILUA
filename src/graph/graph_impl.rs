use std::collections::BTreeMap;
use crate::graph::visit::DfsPostOrder;

#[derive(Debug)]
pub struct Node<N> {
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

impl<N: Clone> Node<N> {
    pub fn weight(&self) -> N {
        self.weight.clone()
    }
}

#[derive(Debug)]
pub struct Edge<E> {
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

    #[inline(always)]
    pub fn to(&self) -> u32 { self.to }

    #[inline(always)]
    pub fn from(&self) -> u32 { self.from }
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

/// Node outgoing edges iterator. `Outputs` implements `Iterator`.
pub struct Outputs<'graph, E> {
    edges: &'graph [Edge<E>],
    next: Option<u32>,
}

/// Iterates through outgoing edge list
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

/// Node incoming edges iterator. `Inputs` implements `Iterator`.
pub struct Inputs<'graph, E> {
    edges: &'graph [Edge<E>],
    next: Option<u32>,
}

/// Iterates through incoming edge list
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

impl<N: Clone, E: Clone> Graph<N, E> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn nodes(&self) -> &BTreeMap<u32, Node<N>> {
        &self.nodes
    }

    #[inline(always)]
    pub fn edges(&self) -> Vec<Edge<E>> {
        self.edges.clone()
    }

    #[inline(always)]
    pub fn edge(&self, idx: u32) -> Option<&Edge<E>> {
        self.edges.get(idx as usize)
    }

    #[inline(always)]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
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
    /// Returns index of outgoing node
    pub fn edge_to(&self, index: u32) -> u32 {
        self.edges.get(index as usize).unwrap().to
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
            .map(|(&index, _)| index)
    }

    #[inline(always)]
    pub fn try_next_node(&self, index: u32) -> Option<u32> {
        self.nodes
            .range(index..)
            .next()
            .map(|(&index, _)| index)
    }

    #[inline(always)]
    /// Get node from graph by index in BTreeMap
    pub fn node(&self, index: u32) -> Option<&Node<N>> {
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
    /// Returns Iterator `Outputs` of outgoing node edges by node index
    pub fn outputs(&self, node_idx: u32) -> Outputs<E> {
        Outputs {
            edges: &self.edges,
            next: self.node(node_idx).unwrap().next_outgoing_edge,
        }
    }

    #[inline(always)]
    /// Returns Iterator `Inputs` of incoming node edges by node index
    pub fn inputs(&self, node_idx: u32) -> Inputs<E> {
        Inputs {
            edges: &self.edges,
            next: self.node(node_idx).unwrap().next_incoming_edge,
        }
    }

    pub fn dfs_post_order_visitor(&self, start: u32) -> DfsPostOrder {
        DfsPostOrder::new(self, start)
    }
}




