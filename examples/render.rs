
extern crate flow;
extern crate petgraph;

use petgraph::graph::Graph;
use petgraph::dot::Dot;
use flow::residuals;

fn main() {
    // All arrows going from top to bottom, except B -> C
    // which is left to right.
    //
    //          A
    //      7      2
    //  B       2      C
    //      1      5
    //          D

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

    println!("{}", Dot::new(&g));

    let r = residuals(&g);

    println!("{}", Dot::new(&r));
}
