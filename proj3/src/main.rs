use core::num;
use graph::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
    Empty,
    Red,
    Green,
    Blue,
    Yellow,
}

enum NumColors {
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug)]
struct Choices {
    red: bool,
    green: bool,
    blue: bool,
    yellow: bool,
}

impl Choices {
    fn new(numColors: NumColors) -> Self {
        match numColors {
            NumColors::Two => Self {
                red: true,
                green: true,
                blue: false,
                yellow: false,
            },
            NumColors::Three => Self {
                red: true,
                green: true,
                blue: true,
                yellow: false,
            },
            NumColors::Four => Self {
                red: true,
                green: true,
                blue: true,
                yellow: true,
            },
        }
    }

    fn remove(&mut self, color: Color) {
        match color {
            Color::Empty => (),
            Color::Red => self.red = false,
            Color::Green => self.green = false,
            Color::Blue => self.blue = false,
            Color::Yellow => self.yellow = false,
        }
    }
}

fn main() {
    let graph = graph_from_file("CSPData.csv");
    println!(
        "Graph read (node count: {}, edge count: {})",
        graph.node_count(),
        graph.edge_count()
    );
    print_graph_edges(&graph);

    // explore the graph
    let mut vals = vec![Color::Empty; graph.node_count()];
    let mut queue = VecDeque::<usize>::new();
    queue.push_back(0);
    while let Some(node) = queue.pop_front() {
        println!("{:?}", vals);
        if vals[node] != Color::Empty {
            continue;
        }
        println!("Visiting node {node}");

        // decide color for this node
        // vals[node] = Color::Red;
        let mut choices = Choices::new(NumColors::Four);
        for &nb in graph.neighbors(node) {
            choices.remove(vals[nb]);
        }
        vals[node] = if choices.red {
            Color::Red
        } else if choices.green {
            Color::Green
        } else if choices.blue {
            Color::Blue
        } else if choices.yellow {
            Color::Yellow
        } else {
            panic!("no valid colors to choose");
        };

        // add neighbors to the queue
        for &nb in graph.neighbors(node) {
            if vals[nb] == Color::Empty {
                println!("\tPushing node {nb}");
                queue.push_back(nb);
            }
        }
    }
    println!("{:?}", vals);

    // check if the graph has been properly colored
    let mut done = true;
    'outer: for n1 in 0..graph.node_count() {
        if vals[n1] == Color::Empty {
            done = false;
            break 'outer;
        }
        for &n2 in graph.neighbors(n1) {
            if vals[n1] == vals[n2] {
                done = false;
                break 'outer;
            }
        }
    }
    println!("done: {done}");
}

fn graph_from_file(path: &str) -> UndirectedCsrGraph<usize> {
    GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .edges(edges_from_file(path))
        .build()
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
        let line: Vec<&str> = line.iter().map(|x| x.trim()).collect();
        // println!("{:?}", line);
        assert_eq!(line.len() - 1, num_vars, "line is the wrong size");
        for (col, string) in line.iter().skip(1).enumerate() {
            if string.trim() == "1" {
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

fn print_graph_edges(graph: &UndirectedCsrGraph<usize>) {
    println!("Printing graph edges:");
    for n1 in 0..graph.node_count() {
        for n2 in graph.neighbors(n1) {
            println!("n1: {n1}, n2: {n2}")
        }
    }
}
