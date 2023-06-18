use graph::prelude::*;
use rand::{distributions::Standard, prelude::*};
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader, Write},
};

const MAX_ITER: usize = 2048;

fn output(string: &str, file: &mut File) {
    let bytes = string.as_bytes();
    std::io::stdout().write(bytes).expect("stdio write failed");
    file.write(bytes).expect("file write failed");
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Empty,
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    fn rand_from_choices<R: Rng + ?Sized>(choices: Choices, rng: &mut R) -> Self {
        assert!(
            choices.red || choices.green || choices.blue || choices.yellow,
            "at least one choice must be available in order to pick randomly from them",
        );
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

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Empty => f.write_str("E"),
            Color::Red => f.write_str("R"),
            Color::Green => f.write_str("G"),
            Color::Blue => f.write_str("B"),
            Color::Yellow => f.write_str("Y"),
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

type GraphColoring = HashMap<usize, Color>;

#[derive(Clone, Copy, Debug)]
enum NumColors {
    Two,
    Three,
    Four,
}

impl Display for NumColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumColors::Two => f.write_str("two colors"),
            NumColors::Three => f.write_str("three colors"),
            NumColors::Four => f.write_str("four colors"),
        }
    }
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
    fn print_all_edges(&self);
    fn is_complete(&self, _: &GraphColoring) -> bool;
    fn count_confl(&self, _: &GraphColoring) -> usize;
    fn count_confl_idx(&self, _: usize, _: &GraphColoring) -> usize;
    fn naive_coloring(&self, _: NumColors) -> GraphColoring;
    fn local_search(&self, _: &str) -> GraphColoring;
    fn local_search_num_colors(&self, _: NumColors, _: &mut File) -> GraphColoring;
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

    fn print_all_edges(&self) {
        println!("Printing graph edges:");
        for n1 in 0..self.node_count() {
            print!("X{} --> (", n1 + 1);
            for (idx, n2) in self.neighbors(n1).enumerate() {
                print!(
                    "X{}{}",
                    n2 + 1,
                    if idx < self.degree(n1) - 1 { ", " } else { "" }
                );
            }
            println!(")")
        }
    }

    fn is_complete(&self, vals: &GraphColoring) -> bool {
        vals[&0] != Color::Empty && self.count_confl(vals) == 0
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
            if vals[&idx] == vals[&nb] {
                confl_count += 1;
            }
        }
        confl_count
    }

    // naively choses a coloring in a DFS manner via proccess of elimination,
    // or randomness if no valid possition is found for a node
    fn naive_coloring(&self, num_colors: NumColors) -> GraphColoring {
        let mut rng = rand::thread_rng();
        let mut vals = HashMap::new();
        let mut visited = vec![false; self.node_count()];
        let mut queue = VecDeque::<usize>::new();

        queue.push_back(0);
        while let Some(idx) = queue.pop_front() {
            // println!("{:?}\n{:?}", vals, queue);

            // decide color for this node
            let mut choices = Choices::new(num_colors);
            for nb in self.neighbors(idx) {
                if let Some(&color) = vals.get(nb) {
                    choices.remove(color);
                }
            }
            vals.insert(
                idx,
                if choices.red {
                    Color::Red
                } else if choices.green {
                    Color::Green
                } else if choices.blue {
                    Color::Blue
                } else if choices.yellow {
                    Color::Yellow
                } else {
                    Color::rand_from_choices(Choices::new(num_colors), &mut rng)
                },
            );
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

    fn local_search(&self, graph_name: &str) -> GraphColoring {
        let mut f = File::create(format!("output/{graph_name}.txt")).expect("couldn't create file");
        output(format!("Starting local search for {graph_name} graph (first with two colors, then three, then four).\n\n").as_str(), &mut f);
        let vals = self.local_search_num_colors(NumColors::Two, &mut f);
        if self.is_complete(&vals) {
            return vals;
        }

        let vals = self.local_search_num_colors(NumColors::Three, &mut f);
        if self.is_complete(&vals) {
            return vals;
        }

        self.local_search_num_colors(NumColors::Four, &mut f)
    }

    fn local_search_num_colors(&self, num_colors: NumColors, mut f: &mut File) -> GraphColoring {
        let mut rng = rand::thread_rng();
        output(
            format!("Starting a local search with {num_colors}.\n").as_str(),
            &mut f,
        );

        // start with a naive coloring
        let mut vals = self.naive_coloring(num_colors);

        let mut itr: usize = 0;
        while !self.is_complete(&vals) {
            let num_conflicts = self.count_confl(&vals);
            if itr >= MAX_ITER {
                output(
                    format!(
                        "Local search with {num_colors} failed (iterations: {itr}, num_conflicts: {num_conflicts})\nFinal assignments: {:?}\n\n",
                        &vals
                    ).as_str(),
                    &mut f
                );
                return vals;
            }
            // output(format!("iteration {}, conflicts: {}\ncurrent vals: {:?}\n", itr, num_conflicts, vals), &mut f);

            // traverse the local problem space
            // determine an idx at which to minimize conflicts
            let idx_to_randomize: usize = (0..self.node_count())
                .into_iter()
                .filter(|&idx| self.count_confl_idx(idx, &vals) > 0)
                .choose_stable(&mut rng)
                .expect("couldn't find an idx to randomly change");

            // figure out what choices are available at this index
            let mut choices = Choices::new(num_colors);
            for nb in self.neighbors(idx_to_randomize) {
                if let Some(&color) = vals.get(nb) {
                    choices.remove(color);
                }
            }

            // randomize the chosen node
            vals.insert(
                idx_to_randomize,
                Color::rand_from_choices(Choices::new(num_colors), &mut rng),
            );
            itr += 1;
        }

        output(
            format!( "Local search with {num_colors} completed (iterations: {itr})\nFinal assignments: {:?}\n\n", &vals).as_str(),
            &mut f
        );
        vals
    }
}

fn main() {
    let graph_bipartite: UndirectedCsrGraph<usize> =
        GraphWithColoring::from_file("input/bipartite.csv");
    let graph_needs_three: UndirectedCsrGraph<usize> =
        GraphWithColoring::from_file("input/needs_three.csv");
    let graph_assignment: UndirectedCsrGraph<usize> =
        GraphWithColoring::from_file("input/CSPData.csv");

    _ = graph_bipartite.local_search("bipartite");
    _ = graph_needs_three.local_search("needs_three");
    _ = graph_assignment.local_search("CSPData");
}
