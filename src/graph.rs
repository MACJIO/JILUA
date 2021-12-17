use std::collections::BTreeMap;

#[derive(Debug)]
struct Node<N: Sized> {
    weight: N,
    next_outgoing_edge: Option<u32>,
    next_incoming_edge: Option<u32>,
}

impl<N: Sized + Clone> Clone for Node<N> {
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
struct Edge<E: Sized> {
    weight: E,
    from: u32,
    to: u32,
    next_outgoing_edge: Option<u32>,
    next_incoming_edge: Option<u32>,
}

impl<E: Sized + Clone> Clone for Edge<E> {
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

#[derive(Debug)]
pub struct Graph<N: Sized, E: Sized> {
    nodes: BTreeMap<u32, Node<N>>,
    edges: Vec<Edge<E>>,
}

impl<N: Sized, E: Sized> Graph<N, E> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    #[inline(always)]
    fn edge_mut(&mut self, index: u32) -> Option<&mut Edge<E>> {
        self.edges.get_mut(index as usize)
    }

    #[inline(always)]
    pub fn node_mut(&mut self, index: u32) -> Option<&mut Node<N>> {
        self.nodes.get_mut(&index)
    }

    #[inline(always)]
    pub fn try_node_with_max_index_less_then_or_equal(&self, index: u32) -> Option<u32> {
        self.nodes.get(&index).map_or_else(
            || {
                self.nodes
                    .range(..index)
                    .next_back()
                    .map_or_else(|| None, |entry| Some(*entry.0))
            },
            |_| Some(index),
        )
    }

    #[inline(always)]
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
}

#[cfg(test)]
mod tests {
    use crate::Graph;

    #[test]
    fn add_node_block() {
        let mut graph: Graph<u32, u32> = Graph::new();

        assert_eq!(graph.add_node(0, 1).unwrap(), 0);
        // assert_eq!(graph.add_node(0, 2), 0);
    }
}
