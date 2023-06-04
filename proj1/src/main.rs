use priority_queue::PriorityQueue;
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader, Write},
    vec,
};

const SLEEPER_TIME: std::time::Duration = std::time::Duration::from_millis(0);

fn output(string: &str, file: &mut File) {
    let bytes = string.as_bytes();
    std::io::stdout().write(bytes).expect("stdio write failed");
    file.write(bytes).expect("file write failed");
    std::thread::sleep(SLEEPER_TIME);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Terrain {
    Road,
    Field,
    Forest,
    Hills,
    River,
    Mountians,
    Water,
}

impl Terrain {
    fn from(c: &char) -> Option<Self> {
        match c {
            'R' => Some(Self::Road),
            'f' => Some(Self::Field),
            'F' => Some(Self::Forest),
            'h' => Some(Self::Hills),
            'r' => Some(Self::River),
            'M' => Some(Self::Mountians),
            'W' => Some(Self::Water),
            _ => None,
        }
    }
    fn cost(&self) -> usize {
        match self {
            Self::Road => 1,
            Self::Field => 2,
            Self::Forest => 4,
            Self::Hills => 5,
            Self::River => 7,
            Self::Mountians => 10,
            Self::Water => 0, // unreachable
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Status {
    None,
    Path,
    Up(bool),
    Down(bool),
    Left(bool),
    Right(bool),
}

impl Status {
    fn deactivate(&mut self) {
        match self {
            Status::Up(true) => *self = Status::Up(false),
            Status::Down(true) => *self = Status::Down(false),
            Status::Left(true) => *self = Status::Left(false),
            Status::Right(true) => *self = Status::Right(false),
            _ => (),
        };
    }
}

type Spot = (Terrain, Status);
type Vec2 = (usize, usize);

#[derive(Debug, Hash, PartialEq, Eq)]
struct Visit {
    step: usize,
    loc: Vec2,
    cost: usize,
}

impl Visit {
    fn new(step: usize, loc: Vec2, cost: usize) -> Self {
        Self { step, loc, cost }
    }
}

#[derive(Clone)]
struct Map {
    map: Vec<Vec<Spot>>,
    costs: Vec<Vec<usize>>,
    display_costs: bool,
    dim: Vec2,
    start: Vec2,
    goal: Vec2,
}

impl Map {
    fn parse_line(reader: &mut BufReader<File>) -> Vec<String> {
        let mut line = String::new();
        reader.read_line(&mut line).expect("Error reading line");
        line.trim().split(' ').map(|x| x.to_string()).collect()
    }

    fn from_file_path(path: &str) -> Self {
        let file = File::open(path).expect("Couldn't open file");
        let mut reader = BufReader::new(file);
        let dim = Self::parse_line(&mut reader);
        assert_eq!(
            dim.len(),
            2,
            "Invalid number of arguments for map dimensions"
        );
        let dim: Vec2 = (
            dim.get(0)
                .expect("Couldn't read width")
                .parse()
                .expect("Couldn't parse width"),
            dim.get(1)
                .expect("Couldn't read height")
                .parse()
                .expect("Couldn't parse height"),
        );
        if dim.0 < 1 || dim.1 < 1 {
            panic!("Dimensions are not large enough");
        }

        let start = Self::parse_line(&mut reader);
        assert_eq!(
            start.len(),
            2,
            "Invalid number of arguments for start position"
        );
        let start: Vec2 = (
            start
                .get(0)
                .expect("Couldn't read start X")
                .parse()
                .expect("Couldn't parse start X"),
            start
                .get(1)
                .expect("Couldn't read start Y")
                .parse()
                .expect("Couldn't parse start Y"),
        );
        if start.0 >= dim.0 || start.1 >= dim.1 {
            panic!("Start position is out of bounds");
        }

        let goal = Self::parse_line(&mut reader);
        assert_eq!(
            goal.len(),
            2,
            "Invalid number of arguments for goal position"
        );
        let goal: Vec2 = (
            goal.get(0)
                .expect("Couldn't read goal X")
                .parse()
                .expect("Couldn't parse goal X"),
            goal.get(1)
                .expect("Couldn't read goal Y")
                .parse()
                .expect("Couldn't parse goal Y"),
        );
        if goal.0 >= dim.0 || goal.1 >= dim.1 {
            panic!("Goal position is out of bounds");
        }

        let mut line_num = 0;
        let mut map = Map {
            map: vec![],
            costs: vec![],
            display_costs: false,
            dim,
            start,
            goal,
        };
        while line_num < dim.1 {
            let mut line = String::new();
            if reader.read_line(&mut line).expect("Error reading line") == 0 {
                break;
            }
            let line = line.trim();
            assert_eq!(line.len(), dim.0 as usize, "Map line is the wrong length");
            let mut row: Vec<(Terrain, Status)> = vec![];
            for c in line.chars() {
                row.push((
                    Terrain::from(&c).expect("Could not parse map character"),
                    Status::None,
                ));
            }
            map.map.push(row);
            line_num += 1;
        }
        if line_num != dim.1 {
            panic!("Not enough map data was provided");
        }
        map
    }

    fn map_text(&self) -> String {
        let width = self.map.get(0).unwrap().len();
        let divider = &format!(
            "\n▐{}━━━▌\n▐",
            String::from_iter(std::iter::repeat("━━━╋").take(width - 1))
        );
        let mut s = format!(
            "▗{}▄▄▄▖\n▐",
            String::from_iter(std::iter::repeat("▄▄▄▄").take(width - 1))
        );

        for (r, row) in self.map.iter().enumerate() {
            let check_row = r == self.start.1 || r == self.goal.1;
            for (c, tile) in row.iter().enumerate() {
                let s_terrain = match tile.0 {
                    Terrain::Road => "R",
                    Terrain::Field => "f",
                    Terrain::Forest => "F",
                    Terrain::Hills => "h",
                    Terrain::River => "r",
                    Terrain::Mountians => "M",
                    Terrain::Water => "W",
                };
                let s_status = match tile.1 {
                    Status::None => " ",
                    Status::Path => "█",        //
                    Status::Up(true) => "▲",    // "⇑",
                    Status::Down(true) => "▼",  // "⇓",
                    Status::Left(true) => "◄",  // "«",
                    Status::Right(true) => "►", // "»",
                    Status::Up(false) => "↑",
                    Status::Down(false) => "↓",
                    Status::Left(false) => "←",
                    Status::Right(false) => "→",
                };
                let s_start_goal = if check_row {
                    if c == self.start.0 && r == self.start.1 {
                        "S"
                    } else if c == self.goal.0 && r == self.goal.1 {
                        "G"
                    } else {
                        " "
                    }
                } else {
                    " "
                };
                if self.display_costs && self.costs[r][c] < 99 {
                    let s_cost = &format!("{:0width$}", self.costs[r][c], width = 2);
                    s += s_cost;
                    if s_start_goal == "S" {
                        s += "S";
                    } else if s_start_goal == "G" && s_status == " " {
                        s += "G";
                    } else {
                        s += s_status;
                    }
                } else {
                    s += s_terrain;
                    s += s_status;
                    s += s_start_goal;
                }
                s += if c == row.len() - 1 { "▌" } else { "┃" }
            }
            if r != self.map.len() - 1 {
                s += divider;
            }
        }
        s += &format!(
            "\n▝{}▀▀▀▘\n",
            String::from_iter(std::iter::repeat("▀▀▀▀").take(width - 1))
        );
        s
    }

    fn at(&self, loc: Vec2) -> Option<Spot> {
        if loc.0 < self.dim.0 && loc.1 < self.dim.1 && self.map[loc.1][loc.0].0 != Terrain::Water {
            return Some(self.map[loc.1][loc.0]);
        }
        None
    }

    fn at_mut(&mut self, loc: Vec2) -> Option<&mut Spot> {
        if loc.0 < self.dim.0 && loc.1 < self.dim.1 {
            return Some(&mut self.map[loc.1][loc.0]);
        }
        None
    }

    fn follow(&self, loc: Vec2) -> Option<Vec2> {
        match self.at(loc).expect("Followed path to invalid position") {
            (_, Status::Up(_)) => Some((loc.0, loc.1 - 1)),
            (_, Status::Down(_)) => Some((loc.0, loc.1 + 1)),
            (_, Status::Left(_)) => Some((loc.0 - 1, loc.1)),
            (_, Status::Right(_)) => Some((loc.0 + 1, loc.1)),
            _ => None,
        }
    }

    fn go_up(&self, loc: &Vec2) -> Option<Vec2> {
        if loc.1 == 0 {
            return None;
        }
        let loc = (loc.0, loc.1 - 1);
        if let Some((_, s)) = self.at(loc) {
            if s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn go_down(&self, loc: &Vec2) -> Option<Vec2> {
        let loc = (loc.0, loc.1 + 1);
        if let Some((_, s)) = self.at(loc) {
            if s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn go_left(&self, loc: &Vec2) -> Option<Vec2> {
        if loc.0 == 0 {
            return None;
        }
        let loc = (loc.0 - 1, loc.1);
        if let Some((_, s)) = self.at(loc) {
            if s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn go_right(&self, loc: &Vec2) -> Option<Vec2> {
        let loc = (loc.0 + 1, loc.1);
        if let Some((_, s)) = self.at(loc) {
            if s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn get_up(&self, loc: &Vec2) -> Option<Vec2> {
        if loc.1 == 0 {
            return None;
        }
        let loc = (loc.0, loc.1 - 1);
        match self.at(loc) {
            Some(_) => Some(loc),
            _ => None,
        }
    }

    fn get_down(&self, loc: &Vec2) -> Option<Vec2> {
        let loc = (loc.0, loc.1 + 1);
        match self.at(loc) {
            Some(_) => Some(loc),
            _ => None,
        }
    }

    fn get_left(&self, loc: &Vec2) -> Option<Vec2> {
        if loc.0 == 0 {
            return None;
        }
        let loc = (loc.0 - 1, loc.1);
        match self.at(loc) {
            Some(_) => Some(loc),
            _ => None,
        }
    }

    fn get_right(&self, loc: &Vec2) -> Option<Vec2> {
        let loc = (loc.0 + 1, loc.1);
        match self.at(loc) {
            Some(_) => Some(loc),
            _ => None,
        }
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.map_text();
        write!(f, "{string}")
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.map_text();
        write!(f, "{string}")
    }
}

enum DistMode {
    TaxiCab,
    Euclidean,
}

fn dist(a: Vec2, b: Vec2, mode: DistMode) -> usize {
    let d = (a.0.abs_diff(b.0), a.1.abs_diff(b.1));
    match mode {
        DistMode::TaxiCab => d.0 + d.1,
        DistMode::Euclidean => ((d.0 * d.0 + d.1 * d.1) as f64).sqrt().floor() as usize,
    }
}

// ---- THE ALGORITHMS ---- //
fn breadth_first(map: &Map) {
    let mut f = File::create("results/breadth_first_results.txt").unwrap();
    output("Running breadth first search\n", &mut f);
    let mut done = false;
    let mut map = map.clone();
    let mut q = VecDeque::<(usize, Vec2)>::new();
    let start = map.start;
    let goal = map.goal;

    map.map[start.1][start.0].1 = Status::Path;
    q.push_back((0, start));

    let mut step_prev = 1;
    let mut pops = 0;
    'main_loop: while let Some((step, loc)) = q.pop_front() {
        pops += 1;
        if step != step_prev {
            output(&map.map_text(), &mut f);
        }

        let tile = &mut map.map[loc.1][loc.0];
        tile.1.deactivate();

        // For each untraversed valid neighbor, update its direction
        let nbs = vec![
            (map.go_up(&loc), Status::Down(true)),
            (map.go_down(&loc), Status::Up(true)),
            (map.go_left(&loc), Status::Right(true)),
            (map.go_right(&loc), Status::Left(true)),
        ];
        for n in nbs.into_iter() {
            // (it came from here), and add it to the visit queue.
            if let (Some(loc_new), dir) = n {
                map.at_mut(loc_new).unwrap().1 = dir;
                q.push_back((step + 1, loc_new));

                if loc_new == goal {
                    done = true;
                    while let Some((_, loc)) = q.pop_front() {
                        pops += 1;
                        map.at_mut(loc).unwrap().1.deactivate();
                    }
                    output(&map.map_text(), &mut f);
                    break 'main_loop;
                }
            }
        }

        step_prev = step;
    }

    if done {
        // Now do backtracking
        output("Doing backtracking\n", &mut f);

        let mut dist: usize = 0;
        let mut cost: usize = 0;
        let mut loc_opt = Some(goal);
        loop {
            match loc_opt {
                Some(loc) => {
                    loc_opt = map.follow(loc);
                    map.at_mut(loc).unwrap().1 = Status::Path;
                    dist += 1;
                    cost += map.at(loc).unwrap().0.cost();
                    if loc != start {
                        output(&map.map_text(), &mut f);
                    }
                }
                None => break,
            }
        }
        output(
            &format!(
                "Path found (dist: {dist} cost: {cost} iterations: {pops}) by breadth first alg\n"
            ),
            &mut f,
        );
    } else {
        output(
            "Breadth first search failed! No valid paths exist\n",
            &mut f,
        );
    }
    f.flush().expect("Couldn't flush to file");
}

fn lowest_cost_path(map: &Map) {
    let mut f = File::create("results/lowest_cost_results.txt").unwrap();
    output("Running lowest cost search\n", &mut f);
    let mut done = false;
    let mut map = map.clone();
    let mut q = PriorityQueue::<Visit, usize>::new();
    let start = map.start;
    let goal = map.goal;

    map.costs = vec![vec![usize::MAX; map.dim.0]; map.dim.1];
    map.display_costs = true;
    map.map[start.1][start.0].1 = Status::Path;
    map.costs[start.1][start.0] = map.map[start.1][start.0].0.cost();
    q.push(Visit::new(0, start, map.costs[start.1][start.0]), 0);

    let mut pops = 0;
    'main_loop: while let Some((v, _)) = q.pop() {
        pops += 1;
        let (step, loc, cost) = (v.step, v.loc, v.cost);
        output(&map.map_text(), &mut f);

        let tile = &mut map.map[loc.1][loc.0];
        tile.1.deactivate();

        // For each valid unvisited neighbor, check if it would have been better to come from here.
        // if so, update its cost and direction, the add it to the visit queue.
        let nbs = vec![
            (map.go_up(&loc), Status::Down(true)),
            (map.go_down(&loc), Status::Up(true)),
            (map.go_left(&loc), Status::Right(true)),
            (map.go_right(&loc), Status::Left(true)),
        ];
        for n in nbs.into_iter() {
            if let (Some(loc_new), dir) = n {
                let maybe_cost = cost + map.at(loc_new).unwrap().0.cost();
                if maybe_cost < map.costs[loc_new.1][loc_new.0] {
                    map.costs[loc_new.1][loc_new.0] = maybe_cost;
                    map.at_mut(loc_new).unwrap().1 = dir;
                    q.push(
                        Visit::new(step + 1, loc_new, maybe_cost),
                        usize::MAX - maybe_cost,
                    );
                }
                if loc_new == goal {
                    done = true;
                    while let Some((v, _)) = q.pop() {
                        pops += 1;
                        map.at_mut(v.loc).unwrap().1.deactivate();
                    }
                    output(&map.map_text(), &mut f);
                    break 'main_loop;
                }
            }
        }
    }

    if done {
        // Now do backtracking
        output("Doing backtracking\n", &mut f);

        let mut dist: usize = 0;
        let mut cost: usize = 0;
        let mut loc_opt = Some(goal);
        loop {
            match loc_opt {
                Some(loc) => {
                    loc_opt = map.follow(loc);
                    map.at_mut(loc).unwrap().1 = Status::Path;
                    dist += 1;
                    cost += map.at(loc).unwrap().0.cost();
                    if loc == start {
                        map.display_costs = false;
                    }
                    output(&map.map_text(), &mut f);
                }
                None => break,
            }
        }

        output(
            &format!(
                "Path found (dist: {dist} cost: {cost} iterations: {pops}) by lowest cost alg\n"
            ),
            &mut f,
        );
    } else {
        output("Lowest cost search failed! No valid paths exist\n", &mut f);
    }
    f.flush().expect("Couldn't flush to file");
}

fn greedy_best_first(map: &Map) {
    let mut f = File::create("results/greedy_best_first_results.txt").unwrap();
    output("Running greedy best first search\n", &mut f);
    let mut done = false;
    let mut map = map.clone();
    let mut q = PriorityQueue::<Visit, usize>::new();
    let start = map.start;
    let goal = map.goal;

    map.costs = vec![vec![usize::MAX; map.dim.0]; map.dim.1];
    map.display_costs = true;
    map.map[start.1][start.0].1 = Status::Path;
    map.costs[start.1][start.0] = map.map[start.1][start.0].0.cost();
    q.push(Visit::new(0, start, map.costs[start.1][start.0]), 0);

    let mut pops = 0;
    'main_loop: while let Some((v, _)) = q.pop() {
        pops += 1;
        let (step, loc, cost) = (v.step, v.loc, v.cost);
        output(&map.map_text(), &mut f);

        let tile = &mut map.map[loc.1][loc.0];
        tile.1.deactivate();

        // For each untraversed valid neighbor, update its cost and direction,
        // and add it to the priority queue (priority based on TaxiCab dist to goal).
        let nbs = vec![
            (map.go_up(&loc), Status::Down(true)),
            (map.go_down(&loc), Status::Up(true)),
            (map.go_left(&loc), Status::Right(true)),
            (map.go_right(&loc), Status::Left(true)),
        ];
        for n in nbs.into_iter() {
            if let (Some(loc_new), dir) = n {
                map.costs[loc_new.1][loc_new.0] = cost + map.at(loc_new).unwrap().0.cost();
                map.at_mut(loc_new).unwrap().1 = dir;
                q.push(
                    Visit::new(step + 1, loc_new, map.costs[loc_new.1][loc_new.0]),
                    usize::MAX - dist(loc_new, goal, DistMode::TaxiCab),
                );
                if loc_new == goal {
                    done = true;
                    while let Some((v, _)) = q.pop() {
                        pops += 1;
                        map.at_mut(v.loc).unwrap().1.deactivate();
                    }
                    output(&map.map_text(), &mut f);
                    break 'main_loop;
                }
            }
        }
    }

    if done {
        // Now do backtracking
        output("Doing backtracking\n", &mut f);

        let mut dist: usize = 0;
        let mut cost: usize = 0;
        let mut loc_opt = Some(goal);
        loop {
            match loc_opt {
                Some(loc) => {
                    loc_opt = map.follow(loc);
                    map.at_mut(loc).unwrap().1 = Status::Path;
                    dist += 1;
                    cost += map.at(loc).unwrap().0.cost();
                    if loc == start {
                        map.display_costs = false;
                    }
                    output(&map.map_text(), &mut f);
                }
                None => break,
            }
        }
        output(
            &format!(
            "Path found (dist: {dist} cost: {cost} iterations: {pops}) by greedy best first alg\n"
        ),
            &mut f,
        );
    } else {
        output(
            "Greedy best first search failed! No valid paths exist\n",
            &mut f,
        );
    }
    f.flush().expect("Couldn't flush to file");
}

fn a_star_1(map: &Map) {
    let mut f = File::create("results/a_star_1_results.txt").unwrap();
    output("Running A* search (heuristic: taxicab dist)\n", &mut f);
    let mut done = false;
    let mut map = map.clone();
    let mut q = PriorityQueue::<Visit, usize>::new();
    let start = map.start;
    let goal = map.goal;

    map.costs = vec![vec![usize::MAX; map.dim.0]; map.dim.1];
    map.display_costs = true;
    map.map[start.1][start.0].1 = Status::Path;
    map.costs[start.1][start.0] = map.map[start.1][start.0].0.cost();
    q.push(Visit::new(0, start, map.costs[start.1][start.0]), 0);

    let mut pops = 0;
    'main_loop: while let Some((v, _)) = q.pop() {
        pops += 1;
        let (step, loc, cost) = (v.step, v.loc, v.cost);
        output(&map.map_text(), &mut f);

        let tile = &mut map.map[loc.1][loc.0];
        tile.1.deactivate();

        // For each valid unvisited neighbor, check if it would have been better to come from here.
        // if so, update its cost and direction, the add it to the visit queue.
        let nbs = vec![
            (map.go_up(&loc), Status::Down(true)),
            (map.go_down(&loc), Status::Up(true)),
            (map.go_left(&loc), Status::Right(true)),
            (map.go_right(&loc), Status::Left(true)),
        ];
        for n in nbs.into_iter() {
            if let (Some(loc_new), dir) = n {
                let maybe_cost = cost + map.at(loc_new).unwrap().0.cost();
                if maybe_cost < map.costs[loc_new.1][loc_new.0] {
                    map.costs[loc_new.1][loc_new.0] = maybe_cost;
                    map.at_mut(loc_new).unwrap().1 = dir;
                    q.push(
                        Visit::new(step + 1, loc_new, maybe_cost),
                        usize::MAX - (maybe_cost + dist(loc_new, goal, DistMode::TaxiCab)),
                    );
                }
                if loc_new == goal {
                    done = true;
                    while let Some((v, _)) = q.pop() {
                        pops += 1;
                        map.at_mut(v.loc).unwrap().1.deactivate();
                    }
                    output(&map.map_text(), &mut f);
                    break 'main_loop;
                }
            }
        }
    }

    if done {
        // Now do backtracking
        output("Doing backtracking\n", &mut f);

        let mut dist: usize = 0;
        let mut cost: usize = 0;
        let mut loc_opt = Some(goal);
        loop {
            match loc_opt {
                Some(loc) => {
                    loc_opt = map.follow(loc);
                    map.at_mut(loc).unwrap().1 = Status::Path;
                    dist += 1;
                    cost += map.at(loc).unwrap().0.cost();
                    if loc == start {
                        map.display_costs = false;
                    }
                    output(&map.map_text(), &mut f);
                }
                None => break,
            }
        }
        output(
            &format!(
                "Path found (dist: {dist} cost: {cost} iterations: {pops}) by A* (taxicab) alg\n"
            ),
            &mut f,
        );
    } else {
        output("A* search failed! No valid paths exist\n", &mut f);
    }
    f.flush().expect("Couldn't flush to file");
}

fn a_star_2(map: &Map) {
    let mut f = File::create("results/a_star_2_results.txt").unwrap();
    output("Running A* search (heuristic: euclidean dist)\n", &mut f);
    let mut done = false;
    let mut map = map.clone();
    let mut q = PriorityQueue::<Visit, usize>::new();
    let start = map.start;
    let goal = map.goal;

    map.costs = vec![vec![usize::MAX; map.dim.0]; map.dim.1];
    map.display_costs = true;
    map.map[start.1][start.0].1 = Status::Path;
    map.costs[start.1][start.0] = map.map[start.1][start.0].0.cost();
    q.push(Visit::new(0, start, map.costs[start.1][start.0]), 0);

    let mut pops = 0;
    'main_loop: while let Some((v, _)) = q.pop() {
        pops += 1;
        let (step, loc, cost) = (v.step, v.loc, v.cost);
        output(&map.map_text(), &mut f);

        let tile = &mut map.map[loc.1][loc.0];
        tile.1.deactivate();

        // For each valid unvisited neighbor, check if it would have been better to come from here.
        // if so, update its cost and direction, the add it to the visit queue.
        let nbs = vec![
            (map.go_up(&loc), Status::Down(true)),
            (map.go_down(&loc), Status::Up(true)),
            (map.go_left(&loc), Status::Right(true)),
            (map.go_right(&loc), Status::Left(true)),
        ];
        for n in nbs.into_iter() {
            if let (Some(loc_new), dir) = n {
                let maybe_cost = cost + map.at(loc_new).unwrap().0.cost();
                if maybe_cost < map.costs[loc_new.1][loc_new.0] {
                    map.costs[loc_new.1][loc_new.0] = maybe_cost;
                    map.at_mut(loc_new).unwrap().1 = dir;
                    q.push(
                        Visit::new(step + 1, loc_new, maybe_cost),
                        usize::MAX - (maybe_cost + dist(loc_new, goal, DistMode::Euclidean)),
                    );
                }
                if loc_new == goal {
                    done = true;
                    while let Some((v, _)) = q.pop() {
                        pops += 1;
                        map.at_mut(v.loc).unwrap().1.deactivate();
                    }
                    output(&map.map_text(), &mut f);
                    break 'main_loop;
                }
            }
        }
    }

    if done {
        // Now do backtracking
        output("Doing backtracking\n", &mut f);

        let mut dist: usize = 0;
        let mut cost: usize = 0;
        let mut loc_opt = Some(goal);
        loop {
            match loc_opt {
                Some(loc) => {
                    loc_opt = map.follow(loc);
                    map.at_mut(loc).unwrap().1 = Status::Path;
                    dist += 1;
                    cost += map.at(loc).unwrap().0.cost();
                    if loc == start {
                        map.display_costs = false;
                    }
                    output(&map.map_text(), &mut f);
                }
                None => break,
            }
        }
        output(
            &format!(
                "Path found (dist: {dist} cost: {cost} iterations: {pops}) by A* (euclid) alg\n"
            ),
            &mut f,
        );
    } else {
        output("A* search failed! No valid paths exist\n", &mut f);
    }
    f.flush().expect("Couldn't flush to file");
}

fn main() {
    let map = Map::from_file_path("map-small.txt");
    println!("The map data has been read successfully:\n{:?}", map);

    breadth_first(&map);
    lowest_cost_path(&map);
    greedy_best_first(&map);
    a_star_1(&map);
    a_star_2(&map);
}
