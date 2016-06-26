
#![cfg_attr(test, feature(test))]

#[cfg(test)]
extern crate test;
extern crate fnv;
extern crate petgraph;

use petgraph::graph::{Graph, NodeIndex};
use petgraph::EdgeDirection::Outgoing;
use std::collections::vec_deque::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;

type Hasher = std::hash::BuildHasherDefault<fnv::FnvHasher>;

/// Edge weights in input graph are capacities.
/// Edge weights in output graph are residual capacities.
/// All residual graphs contain every edge in the input capacity graph,
/// as well as their reverses. i.e. we include an edge even if its residual
/// capacity is 0.
pub fn residuals<N: Clone>(g: &Graph<N, f32>) -> Graph<N, f32> {
    let mut res = Graph::<N, f32>::with_capacity(g.node_count(), g.edge_count());

    for n in g.node_indices() {
        res.add_node(g.node_weight(n).unwrap().clone());
    }

    for e in g.edge_indices() {
        let endpoints = g.edge_endpoints(e).unwrap();

        if let Some(_) = g.find_edge(endpoints.1, endpoints.0) {
            panic!("Graph contains opposite edges: ({x}, {y}) and ({y}, {x})",
                x=endpoints.0.index(), y=endpoints.1.index());
        }

        let weight = g.edge_weight(e).unwrap();
        res.add_edge(endpoints.0, endpoints.1, *weight);
        res.add_edge(endpoints.1, endpoints.0, 0f32);
    }

    res
}

/// Finds a min cut in g and returns the set of nodes in the source half.
pub fn min_cut<N: Clone>(g: &Graph<N, f32>, src: NodeIndex, dst: NodeIndex) -> Vec<N> {
    let mut r = residuals(g);
    while find_augmenting_path(&mut r, src, dst) {
    }
    cut_from_residual(&r, src)
}

pub fn flow_from_residuals<N: Clone>(g: &Graph<N, f32>, r: &Graph<N, f32>) -> Graph<N, f32> {
    let mut f = g.clone();
    for e in f.edge_indices() {
        let endpoints = f.edge_endpoints(e).unwrap();
        let residual = r.find_edge(endpoints.1, endpoints.0).unwrap();
        *f.edge_weight_mut(e).unwrap() = *r.edge_weight(residual).unwrap();
    }
    f
}

/// Returns the set of nodes in the source half of the cut.
pub fn cut_from_residual<N: Clone>(r: &Graph<N, f32>, src: NodeIndex) -> Vec<N> {
    let mut queue = VecDeque::new();
    queue.push_front(src);

    let mut visited: HashSet<_, Hasher> = HashSet::default();

    while let Some(v) = queue.pop_back() {
        let neighbours: Vec<(NodeIndex, &f32)> = r
            .edges_directed(v, Outgoing)
            .filter(|ne| *ne.1 > 0f32 && !visited.contains(&ne.0))
            .collect::<Vec<_>>();

        for n in neighbours {
            queue.push_front(n.0);
            visited.insert(n.0);
        }
    }

    visited.iter().map(|n| r.node_weight(*n).unwrap().clone()).collect::<Vec<_>>()
}

/// Attempts to find an augmenting path in the provided residual graph.
/// Returns false if no such path is found. Otherwise pushes the maximum possible
/// flow through this path and returns true.
pub fn find_augmenting_path<N: Clone>(r: &mut Graph<N, f32>, src: NodeIndex, dst: NodeIndex)
    -> bool {
    let mut visited: HashSet<_, Hasher> = HashSet::default();

    // NodeIndex -> (NodeIndex, EdgeWeight)
    let mut predecessors: HashMap<_, _, Hasher> = HashMap::default();

    let mut queue = VecDeque::new();
    queue.push_front(src);

    while let Some(v) = queue.pop_back() {
        let neighbours: Vec<(NodeIndex, &f32)> = r
            .edges_directed(v, Outgoing)
            .filter(|ne| *ne.1 > 0f32 && !visited.contains(&ne.0))
            .collect::<Vec<_>>();

        for n in neighbours {
            predecessors.insert(n.0, (v, *n.1));
            queue.push_front(n.0);
            visited.insert(n.0);
        }
    }

    if !visited.contains(&dst) {
        return false;
    }

    let mut current = dst;
    let mut min = std::f32::MAX;
    let mut path: Vec<(NodeIndex, NodeIndex, f32)> = vec![];
    while current != src {
        let pred: (NodeIndex, f32) = predecessors[&current];
        min = min.min(pred.1);
        path.push((pred.0, current, pred.1));
        current = pred.0;
    }

    for nn in path.iter() {
        let fwd = r.find_edge(nn.0, nn.1).unwrap();
        *r.edge_weight_mut(fwd).unwrap() -= min;

        let bwd = r.find_edge(nn.1, nn.0).unwrap();
        *r.edge_weight_mut(bwd).unwrap() += min;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{min_cut, residuals};
    use petgraph::graph::{Graph};
    use test::Bencher;

    #[test]
    #[should_panic]
    fn residuals_rejects_opposite_edges() {
        let g = Graph::<(), f32>::from_edges(&[(0, 1), (1, 0)]);
        let _ = residuals(&g);
    }

    #[test]
    fn cut_small() {
        let mut g = Graph::<&str, f32>::new();
        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let d = g.add_node("D");
        g.extend_with_edges(&[
            (a, b, 7f32),
            (a, c, 2f32),
            (b, c, 2f32),
            (b, d, 1f32),
            (c, d, 5f32)
        ]);

        let mut cut = min_cut(&g, a, d);
        cut.sort();
        assert_eq!(cut, vec!["A", "B"]);
    }

    #[bench]
    fn bench_small(t: &mut Bencher) {
        let mut g = Graph::<&str, f32>::new();
        let a = g.add_node("A");
        let b = g.add_node("B");
        let c = g.add_node("C");
        let d = g.add_node("D");
        g.extend_with_edges(&[
            (a, b, 7f32),
            (a, c, 2f32),
            (b, c, 2f32),
            (b, d, 1f32),
            (c, d, 5f32)
        ]);

        t.iter(|| {
            min_cut(&g, a, d)
        });
    }
}
