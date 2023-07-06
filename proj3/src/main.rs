use graph::prelude::*;
use rand::{distributions::Standard, prelude::*};
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    fs::{read_dir, File},
    io::{stdout, BufRead, BufReader, Write},
};

const MAX_ITER: usize = 2048;

fn output(string: &str, file: &mut File) {
    let bytes = string.as_bytes();
    stdout().write(bytes).expect("stdio write failed");
    file.write(bytes).expect("file write failed");
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

    fn stuck(&self) -> bool {
        !self.red && !self.green && !self.blue && !self.yellow
    }

    fn amount(&self) -> usize {
        0 + if self.red { 1 } else { 0 }
            + if self.green { 1 } else { 0 }
            + if self.blue { 1 } else { 0 }
            + if self.yellow { 1 } else { 0 }
    }

    fn as_arr(&self) -> [(bool, Color); 4] {
        [
            (self.red, Color::Red),
            (self.green, Color::Green),
            (self.blue, Color::Blue),
            (self.yellow, Color::Yellow),
        ]
    }
}

// this trait defines the functions used to solve the map coloring problem
trait GraphForColoring {
    fn from_file(_: &str) -> Self;
    fn all_edges_text(&self) -> String;
    fn is_complete(&self, _: &GraphColoring) -> bool;
    fn count_confl(&self, _: &GraphColoring) -> usize;
    fn count_confl_idx(&self, _: usize, _: &GraphColoring) -> usize;
    fn naive_coloring(&self, _: NumColors) -> GraphColoring;
    fn local_search_itr(&self, _: &str) -> GraphColoring;
    fn depth_first_search_itr(&self, _: &str) -> GraphColoring;
    fn local_search(&self, _: NumColors, _: &mut File) -> GraphColoring;
    fn depth_first_search(&self, _: NumColors, _: &mut File) -> GraphColoring;
}

impl GraphForColoring for UndirectedCsrGraph<usize> {
    // creates a graph given the path to an input .csv file
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

    // formats graph nodes/edges for easy printing
    fn all_edges_text(&self) -> String {
        let mut s = String::new();
        for n1 in 0..self.node_count() {
            s += format!("X{} --> (", n1 + 1).as_str();
            for (idx, n2) in self.neighbors(n1).enumerate() {
                s += format!(
                    "X{}{}",
                    n2 + 1,
                    if idx < self.degree(n1) - 1 { ", " } else { "" }
                )
                .as_str();
            }
            s += ")\n";
        }
        s
    }

    // checks if the graph is complete relative to a coloring
    fn is_complete(&self, vals: &GraphColoring) -> bool {
        vals.len() == self.node_count()
            && !vals.into_iter().any(|(_, &x)| x == Color::Empty)
            && self.count_confl(vals) == 0
    }

    // counts all conflicts of the graph relative to a coloring
    fn count_confl(&self, vals: &GraphColoring) -> usize {
        let mut confl_count = 0;
        for n in 0..self.node_count() {
            if vals.contains_key(&n) {
                confl_count += self.count_confl_idx(n, vals);
            }
        }
        confl_count / 2
    }

    // counts conflicts of a specific variable relative to a coloring
    fn count_confl_idx(&self, idx: usize, vals: &GraphColoring) -> usize {
        let mut confl_count = 0;
        for &nb in self.neighbors(idx) {
            if vals.contains_key(&nb) && vals[&idx] == vals[&nb] {
                confl_count += 1;
            }
        }
        confl_count
    }

    // naively choses a coloring in a DFS manner via proccess of elimination,
    // or via randomness if no valid possition is found for that variable.
    fn naive_coloring(&self, num_colors: NumColors) -> GraphColoring {
        let mut rng = rand::thread_rng();
        let mut coloring = HashMap::new();
        let mut visited = vec![false; self.node_count()];
        let mut queue = VecDeque::<usize>::new();

        queue.push_back(0);

        while let Some(idx) = queue.pop_front() {
            // decide color for this variable
            let mut choices = Choices::new(num_colors);
            for nb in self.neighbors(idx) {
                if let Some(&color) = coloring.get(nb) {
                    choices.remove(color);
                }
            }

            // add it
            coloring.insert(
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

            // add neighbors to the queue
            for &nb in self.neighbors(idx) {
                if !visited[nb] {
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
        coloring
    }

    // wrapper for local_search that handles trying with 2 then 3 then 4 colors
    fn local_search_itr(&self, graph_name: &str) -> GraphColoring {
        let mut f = File::create(format!("output/local_search/{graph_name}.txt")).unwrap();
        output(format!("Starting local search for {graph_name} graph (first with two colors, then three, then four).\nGraph:\n{}\n", self.all_edges_text()).as_str(), &mut f);

        // try with 2 colors allowed
        let vals = self.local_search(NumColors::Two, &mut f);
        if self.is_complete(&vals) {
            return vals;
        }

        // try with 3 colors allowed
        let vals = self.local_search(NumColors::Three, &mut f);
        if self.is_complete(&vals) {
            return vals;
        }

        // try with 4 colors allowed
        self.local_search(NumColors::Four, &mut f)
    }

    // wrapper for depth_first_search that handles trying with 2 then 3 then 4 colors
    fn depth_first_search_itr(&self, graph_name: &str) -> GraphColoring {
        let mut f = File::create(format!("output/depth_first_search/{graph_name}.txt"))
            .expect("couldn't create file");
        output(format!("Starting depth first search for {graph_name} graph (first with two colors, then three, then four).\nGraph:\n{}\n", self.all_edges_text()).as_str(), &mut f);

        // try with 2 colors allowed
        let vals = self.depth_first_search(NumColors::Two, &mut f);
        if self.is_complete(&vals) {
            return vals;
        }

        // try with 3 colors allowed
        let vals = self.depth_first_search(NumColors::Three, &mut f);
        if self.is_complete(&vals) {
            return vals;
        }

        // try with 4 colors allowed
        self.depth_first_search(NumColors::Four, &mut f)
    }

    // starts with a naive coloring, then iteratively searches locally by randomly changing
    // a variable that is in high conflict. gives up after a certian amoutn of iterations.
    fn local_search(&self, num_colors: NumColors, mut f: &mut File) -> GraphColoring {
        let mut rng = rand::thread_rng();
        output(
            format!("Starting a local search with {num_colors}...").as_str(),
            &mut f,
        );

        // start with a naive coloring
        let mut coloring = self.naive_coloring(num_colors);

        let mut itr: usize = 0;
        while !self.is_complete(&coloring) {
            let num_conflicts = self.count_confl(&coloring);

            if itr >= MAX_ITER {
                output(
                    format!(" failed with {num_conflicts} conflicts (iterations: {itr})\nFinal coloring: {:?}\n\n", &coloring).as_str(),
                    &mut f
                );
                return coloring;
            }

            // determine an idx at which to minimize conflicts
            let idx_to_randomize: usize = (0..self.node_count())
                .into_iter()
                .filter(|&idx| self.count_confl_idx(idx, &coloring) > 0)
                .choose_stable(&mut rng)
                .expect("couldn't find an idx to randomly change");

            // figure out what choices are available at this index
            let mut choices = Choices::new(num_colors);
            for nb in self.neighbors(idx_to_randomize) {
                if let Some(&color) = coloring.get(nb) {
                    choices.remove(color);
                }
            }

            // randomize the chosen variable
            coloring.insert(
                idx_to_randomize,
                Color::rand_from_choices(
                    if choices.stuck() {
                        Choices::new(num_colors)
                    } else {
                        choices
                    },
                    &mut rng,
                ),
            );
            itr += 1;
        }

        output(
            format!(
                " completed (iterations: {itr})\nFinal coloring: {:#?}\n\n",
                &coloring
            )
            .as_str(),
            &mut f,
        );
        coloring
    }

    // starts with a blank coloring, then fills in the graph in a depth-first manner.
    //     at each step the most constrained variable is changed to
    //     the color that restricts the other variables the least.
    fn depth_first_search(&self, num_colors: NumColors, mut f: &mut File) -> GraphColoring {
        output(
            format!("Starting a depth first search with {num_colors}...").as_str(),
            &mut f,
        );
        let mut coloring = GraphColoring::new();
        let mut choices_vec = vec![Choices::new(num_colors); self.node_count()];

        while !self.is_complete(&coloring) {
            // choose the remaining variable with the least legal values
            let idx_to_assign: usize = (0..self.node_count())
                .into_iter()
                .filter(|x| !coloring.contains_key(x))
                .map(|x| (x, choices_vec[x]))
                .min_by(|(_, x), (_, y)| x.amount().cmp(&y.amount()))
                .unwrap()
                .0;

            if choices_vec[idx_to_assign].stuck() {
                // failed
                output(
                    format!(" failed\nFinal coloring: {:?}\n\n", coloring).as_str(),
                    &mut f,
                );
                return coloring;
            }

            // choose the color that restricts the other variables the least
            let color_choice = choices_vec[idx_to_assign]
                .as_arr()
                .iter()
                .filter(|(a, _)| *a)
                .map(|(_, c)| {
                    // make a copy of choices vec to test choosing this color
                    let mut choices_vec = choices_vec.clone();

                    // remove the color from the choices of the neighbors
                    for &nb in self.neighbors(idx_to_assign) {
                        choices_vec[nb].remove(*c);
                    }

                    (choices_vec.into_iter().fold(0, |a, x| a + x.amount()), *c)
                })
                // choose the color that yields the most options for other variables
                .max_by(|(x, _), (y, _)| x.cmp(y))
                .unwrap()
                .1;

            // commit to the color assignment that was chosen
            coloring.insert(idx_to_assign, color_choice);
            for &nb in self.neighbors(idx_to_assign) {
                choices_vec[nb].remove(color_choice);
            }
        }

        output(
            format!(" completed\nFinal coloring: {:#?}\n\n", coloring).as_str(),
            &mut f,
        );
        coloring
    }
}

fn main() -> Result<(), std::io::Error> {
    // execute the searches for every file in the input directory
    for path in read_dir("input/")? {
        match path {
            Ok(path) => {
                let path = path.file_name();
                let path = path.to_str().unwrap();
                let name_vec: Vec<&str> = path.split('.').collect();
                let name = name_vec[0];
                let graph: UndirectedCsrGraph<usize> =
                    GraphForColoring::from_file(format!("input/{path}").as_str());

                _ = graph.local_search_itr(name);
                _ = graph.depth_first_search_itr(name);
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
