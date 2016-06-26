
extern crate flow;
extern crate petgraph;

use petgraph::graph::Graph;
use petgraph::dot::Dot;
use flow::{residuals, find_augmenting_path, flow_from_residuals};
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::io::Result;

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

fn create_html_from_dot_files() -> Result<()> {
    let mut f = try!(File::create("flow.html"));
    try!(write!(&mut f, "<!doctype html>"));
    try!(write!(&mut f, "<body>"));
    try!(write!(&mut f, "<img src=\"{}\">", "original.svg"));
    try!(write!(&mut f, "<img src=\"{}\">", "res.svg"));
    try!(write!(&mut f, "<img src=\"{}\">", "aug_0.svg"));
    try!(write!(&mut f, "<img src=\"{}\">", "aug_1.svg"));
    try!(write!(&mut f, "<img src=\"{}\">", "aug_2.svg"));
    try!(write!(&mut f, "<img src=\"{}\">", "flow.svg"));
    try!(write!(&mut f, "</body>"));
    Ok(())
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

    let mut count = 0;

    while find_augmenting_path(&mut r, a, d) {
        print_dot(&r, &format!("aug_{}.dot", count));
        count += 1;
    }

    let f = flow_from_residuals(&g, &r);
    print_dot(&f, "flow.dot");

    create_html_from_dot_files();
}
