use std::collections::HashSet;
use crate::graph::Graph;

#[derive(Clone, Debug, Default)]
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
    pub fn new<N: Clone, E: Clone>(graph: &Graph<N, E>, start: u32) -> Self <> {
        let mut dfs = DfsPostOrder {
            stack: Vec::with_capacity(graph.node_count()),
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

    pub fn next<N: Clone, E: Clone>(&mut self, graph: &Graph<N, E>) -> Option<u32> {
        while let Some(&nx) = self.stack.last() {
            // check if already discovered
            if self.discovered.insert(nx) {
                // add neighbors to stack in not discovered
                for edge in graph.outputs(nx) {
                    let node_id = graph.edge_to(edge);

                    if !self.discovered.contains(&node_id) {
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


#[cfg(test)]
mod test {
    use crate::graph::Graph;
    use crate::graph::visit::DfsPostOrder;

    #[test]
    fn test_dfs_postorder() {
        // create test graph
        let mut graph: Graph<u32, u32> = Graph::new();

        graph.add_node(1, 1);
        graph.add_node(2, 2);
        graph.add_node(3, 3);
        graph.add_node(4, 4);
        graph.add_node(5, 5);
        graph.add_node(6, 6);
        graph.add_node(7, 7);
        graph.add_node(8, 8);
        graph.add_node(9, 9);
        graph.add_node(10, 10);
        graph.add_node(11, 11);

        graph.add_edge(0, 1, 2);
        graph.add_edge(0, 1, 8);
        graph.add_edge(0, 2, 3);
        graph.add_edge(0, 2, 6);
        graph.add_edge(0, 2, 7);
        graph.add_edge(0, 3, 4);
        graph.add_edge(0, 3, 5);
        graph.add_edge(0, 8, 9);
        graph.add_edge(0, 8, 10);
        graph.add_edge(0, 10, 11);

        // GRAPH VISUALIZATION
        //                1
        //             /     \
        //            2       8
        //         /  |  \   / \
        //        3   6   7 9   10
        //       / \             \
        //      4   5             11

        let mut dfs = graph.dfs_post_order_visitor(1);

        while let Some(idx) = dfs.next(&graph) {
            println!("idx {}", idx);
        }
    }
}