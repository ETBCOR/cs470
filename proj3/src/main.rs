use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use graph::prelude::*;

fn main() {
    let graph: UndirectedCsrGraph<usize> = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .edges(edges_from_file("CSPData.csv"))
        .build();

    println!(
        "node count: {}, edge count: {}",
        graph.node_count(),
        graph.edge_count()
    );
}

fn edges_from_file(path: &str) -> Vec<(usize, usize)> {
    let file = File::open(path).expect("couldn't open file");
    let mut reader = BufReader::new(file);
    let mut labels = parse_line(&mut reader);
    labels.remove(0);
    // println!("    {:?}", labels);
    let num_vars = labels.len();
    assert!(num_vars > 1, "not enough variables");
    let mut edges = vec![];
    for row in 0..num_vars - 1 {
        let line = parse_line(&mut reader);
        // println!("{:?}", line);
        assert_eq!(line.len() - 1, num_vars, "line is the wrong size");
        for (col, string) in line.iter().skip(1).enumerate() {
            if string == "1" {
                edges.push((row, col));
            }
        }
    }
    edges
}

fn parse_line(reader: &mut BufReader<File>) -> Vec<String> {
    let mut line = String::new();
    reader.read_line(&mut line).expect("couldn't read line");
    line.trim().split(',').map(|x| x.to_string()).collect()
}
