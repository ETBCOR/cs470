use graph::prelude::*;
use rand::{distributions::Standard, prelude::*};
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
};

const MAX_ITER: usize = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
    Empty,
    Red,
    Green,
    Blue,
    Yellow,
}
impl Color {
    fn rand_from_choices<R: Rng + ?Sized>(choices: Choices, rng: &mut R) -> Self {
        match vec![choices.red, choices.green, choices.blue, choices.yellow]
            .iter()
            .enumerate()
            .filter(|(_, &x)| x)
            .choose_stable(rng)
            .unwrap_or((rng.gen_range(0..4), &true))
            .0
        {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            _ => Self::Yellow,
        }
    }
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..4) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            _ => Color::Yellow,
        }
    }
}

type GraphColoring = Vec<Color>;

#[derive(Clone, Copy, Debug)]
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
    fn new(num_colors: NumColors) -> Self {
        match num_colors {
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

trait GraphWithColoring {
    fn from_file(_: &str) -> Self;
    fn print_edges(&self);
    fn is_complete(&self, _: &GraphColoring) -> bool;
    fn count_confl(&self, _: &GraphColoring) -> usize;
    fn count_confl_idx(&self, _: usize, _: &GraphColoring) -> usize;
    fn naive_coloring(&self) -> GraphColoring;
    fn local_search(&self) -> Option<GraphColoring>;
}

impl GraphWithColoring for UndirectedCsrGraph<usize> {
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

    fn is_complete(&self, vals: &GraphColoring) -> bool {
        vals[0] != Color::Empty && self.count_confl(vals) == 0
    }

    fn count_confl(&self, vals: &GraphColoring) -> usize {
        let mut confl_count = 0;
        for n1 in 0..self.node_count() {
            confl_count += self.count_confl_idx(n1, vals);
        }
        confl_count / 2
    }

    fn count_confl_idx(&self, idx: usize, vals: &GraphColoring) -> usize {
        let mut confl_count = 0;
        for &nb in self.neighbors(idx) {
            if vals[idx] == vals[nb] {
                confl_count += 1;
            }
        }
        confl_count
    }

    fn naive_coloring(&self) -> GraphColoring {
        // explore the graph
        let mut vals = vec![Color::Empty; self.node_count()];
        let mut visited = vec![false; vals.len()];
        let mut queue = VecDeque::<usize>::new();
        queue.push_back(0);
        while let Some(idx) = queue.pop_front() {
            // println!("{:?}", vals);
            // println!("{:?}", queue);

            // decide color for this node
            let mut choices = Choices::new(NumColors::Four);
            for &nb in self.neighbors(idx) {
                choices.remove(vals[nb]);
            }
            vals[idx] = if choices.red {
                Color::Red
            } else if choices.green {
                Color::Green
            } else if choices.blue {
                Color::Blue
            } else if choices.yellow {
                Color::Yellow
            } else {
                rand::random()
            };
            // println!("Visiting node {idx} (set color to {:?})", vals[idx]);

            // add neighbors to the queue
            for &nb in self.neighbors(idx) {
                if !visited[nb] {
                    // println!("\tPushing node {nb}");
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
        vals
    }

    fn local_search(&self) -> Option<GraphColoring> {
        let mut rng = rand::thread_rng();

        // start with a naive coloring
        let mut vals = self.naive_coloring();

        let mut itr: usize = 0;
        while !self.is_complete(&vals) {
            if itr == MAX_ITER {
                return None;
            }
            println!(
                "still incomplete. current num of conflicts: {}\ncurrent vals: {:?}",
                self.count_confl(&vals),
                vals
            );
            // traverse the local problem space
            // determine an idx at which to minimize conflicts
            let idx_to_randomize: usize = (0..self.node_count())
                .into_iter()
                .filter(|&idx| self.count_confl_idx(idx, &vals) > 0)
                .choose_stable(&mut rng)
                .expect("couldn't find an idx to randomly change");

            // figure out what choices are available at this index
            let mut choices = Choices::new(NumColors::Four);
            for &nb in self.neighbors(idx_to_randomize) {
                choices.remove(vals[nb]);
            }

            // randomize the chosen node
            vals[idx_to_randomize] = Color::rand_from_choices(choices, &mut rng);
            itr += 1;
        }

        println!("complete! took {} iterations\nfinal vals: {:?}", itr, &vals);
        Some(vals)
    }
}

fn main() {
    let graph: UndirectedCsrGraph<usize> = GraphWithColoring::from_file("CSPData-small.csv");
    println!(
        "Graph read (node count: {}, edge count: {})",
        graph.node_count(),
        graph.edge_count()
    );
    // graph.print_edges();
    let vals = graph.local_search().expect("couldn't complete graph");
    println!("num conflicts: {}", graph.count_confl(&vals));
    println!("complete: {}", graph.is_complete(&vals));
}
