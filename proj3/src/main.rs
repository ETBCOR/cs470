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

trait MyBufReader {
    fn parse_line(&mut self) -> Vec<String>;
}

impl MyBufReader for BufReader<File> {
    fn parse_line(&mut self) -> Vec<String> {
        let mut line = String::new();
        self.read_line(&mut line).expect("couldn't read line");
        line.trim().split(',').map(|x| x.to_string()).collect()
    }
}

trait ColoringGraph {
    fn from_file(_: &str) -> Self;
    fn print_edges(&self);
    fn naive_coloring(&self) -> Vec<Color>;
    fn check_completion(&self, _: &Vec<Color>) -> bool;
}

impl ColoringGraph for UndirectedCsrGraph<usize> {
    fn from_file(path: &str) -> Self {
        let file = File::open(path).expect("couldn't open file");
        let mut reader = BufReader::new(file);
        let mut labels = reader.parse_line();
        labels.remove(0);
        // println!("    {:?}", labels);
        let num_vars = labels.len();
        assert!(num_vars > 1, "not enough variables");
        let mut edges = vec![];
        for row in 0..num_vars - 1 {
            let line = reader.parse_line();
            let line: Vec<&str> = line.iter().map(|x| x.trim()).collect();
            // println!("{:?}", line);
            assert_eq!(line.len() - 1, num_vars, "line is the wrong size");
            for (col, string) in line.iter().skip(1).enumerate() {
                if string.trim() == "1" {
                    edges.push((row, col));
                }
            }
        }
        GraphBuilder::new()
            .csr_layout(CsrLayout::Sorted)
            .edges(edges)
            .build()
    }

    fn print_edges(&self) {
        println!("Printing graph edges:");
        for n1 in 0..self.node_count() {
            for n2 in self.neighbors(n1) {
                println!("n1: {n1}, n2: {n2}")
            }
        }
    }

    fn check_completion(&self, vals: &Vec<Color>) -> bool {
        // check if the graph has been properly colored
        for n1 in 0..self.node_count() {
            if vals[n1] == Color::Empty {
                return false;
            }
            for &n2 in self.neighbors(n1) {
                if vals[n1] == vals[n2] {
                    return false;
                }
            }
        }
        true
    }

    fn naive_coloring(&self) -> Vec<Color> {
        // explore the graph
        let mut vals = vec![Color::Empty; self.node_count()];
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
            for &nb in self.neighbors(node) {
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
            for &nb in self.neighbors(node) {
                if vals[nb] == Color::Empty {
                    println!("\tPushing node {nb}");
                    queue.push_back(nb);
                }
            }
        }
        vals
    }
}

fn main() {
    let graph: UndirectedCsrGraph<usize> = ColoringGraph::from_file("CSPData-small.csv");
    println!(
        "Graph read (node count: {}, edge count: {})",
        graph.node_count(),
        graph.edge_count()
    );
    graph.print_edges();
    let vals = graph.naive_coloring();
    println!("done: {}", graph.check_completion(&vals));
}
