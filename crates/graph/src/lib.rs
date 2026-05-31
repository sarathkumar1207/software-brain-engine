use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction::Incoming;
use sbe_common::{Edge, RelationType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SemanticGraph {
    graph: DiGraph<u64, RelationType>,
    node_map: HashMap<u64, NodeIndex>,
}

impl SemanticGraph {
    pub fn build(edges: &[Edge]) -> Self {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        for edge in edges {
            let from = get_or_insert(&mut graph, &mut node_map, edge.from);
            let to = get_or_insert(&mut graph, &mut node_map, edge.to);
            graph.add_edge(from, to, edge.relation);
        }

        Self { graph, node_map }
    }

    pub fn dependencies(&self, symbol_id: u64) -> Vec<u64> {
        self.node_map
            .get(&symbol_id)
            .map(|node| {
                self.graph
                    .neighbors(*node)
                    .filter_map(|neighbor| self.graph.node_weight(neighbor).copied())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn dependents(&self, symbol_id: u64) -> Vec<u64> {
        self.node_map
            .get(&symbol_id)
            .map(|node| {
                self.graph
                    .neighbors_directed(*node, Incoming)
                    .filter_map(|neighbor| self.graph.node_weight(neighbor).copied())
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn get_or_insert(
    graph: &mut DiGraph<u64, RelationType>,
    node_map: &mut HashMap<u64, NodeIndex>,
    symbol_id: u64,
) -> NodeIndex {
    *node_map
        .entry(symbol_id)
        .or_insert_with(|| graph.add_node(symbol_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbe_common::SourceRange;

    fn edge(from: u64, to: u64) -> Edge {
        Edge {
            from,
            to,
            relation: RelationType::References,
            range: Some(SourceRange {
                start_line: 1,
                end_line: 1,
                start_col: 0,
                end_col: 1,
            }),
        }
    }

    #[test]
    fn traverses_forward_and_reverse() {
        let graph = SemanticGraph::build(&[edge(1, 2), edge(3, 2)]);

        assert_eq!(graph.dependencies(1), vec![2]);
        let mut dependents = graph.dependents(2);
        dependents.sort_unstable();
        assert_eq!(dependents, vec![1, 3]);
    }
}
