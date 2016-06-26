
extern crate flow;
extern crate petgraph;

use petgraph::graph::Graph;
use petgraph::dot::Dot;
use flow::{residuals, find_augmenting_path};
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn print_dot<N: Display, E: Display>(g: &Graph<N, E>, path: &str) {
    use std::process::Command;

    let mut f = File::create(path).unwrap();
    write!(&mut f, "{}", Dot::new(g));

    let svg_path = Path::new(path).with_extension("svg");
    let output = Command::new("dot")
        .arg(path)
        .arg("-Tsvg")
        .arg("-o")
        .arg(svg_path)
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

    println!("{:?}", output);
}

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
    print_dot(&g, "original.dot");

    let mut r = residuals(&g);
    print_dot(&r, "res.dot");

    find_augmenting_path(&mut r, a, d);
    print_dot(&r, "aug.dot");
}
