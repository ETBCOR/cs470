use std::{
    collections::VecDeque,
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Write},
    vec,
};

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
    fn cost(&self) -> u32 {
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

type Spot = (Terrain, Status);
type Vec2 = (usize, usize);

#[derive(Clone)]
struct Map {
    map: Vec<Vec<Spot>>,
    dim: Vec2,
    start: Vec2,
    goal: Vec2,
}

impl Map {
    fn from_file_path(path: &str) -> Self {
        let file = File::open(path).expect("Couldn't open file");
        let mut reader = BufReader::new(file);
        let dim = parse_line(&mut reader);
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

        let start = parse_line(&mut reader);
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

        let goal = parse_line(&mut reader);
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
        let mut s = String::new();
        s += "\n┏";
        s += &String::from_iter(std::iter::repeat("━━━┳").take(width - 1));
        s += "━━━┓\n┃";

        let divider = "\n┣".to_string()
            + &String::from_iter(std::iter::repeat("━━━╋").take(width - 1))
            + "━━━┫\n┃";
        let divider = &divider;

        for (r, row) in self.map.iter().enumerate() {
            let check_row = r == self.start.1 || r == self.goal.1;
            for (c, tile) in row.iter().enumerate() {
                s += match tile.0 {
                    Terrain::Road => "R",
                    Terrain::Field => "f",
                    Terrain::Forest => "F",
                    Terrain::Hills => "h",
                    Terrain::River => "r",
                    Terrain::Mountians => "M",
                    Terrain::Water => "W",
                };
                s += match tile.1 {
                    Status::None => " ",
                    Status::Path => "█",
                    Status::Up(true)
                    | Status::Down(true)
                    | Status::Left(true)
                    | Status::Right(true) => "▒",
                    Status::Up(false) => "↑",    // ⬆
                    Status::Down(false) => "↓",  // ⬇
                    Status::Left(false) => "←",  // ⬅
                    Status::Right(false) => "→", // ⮕
                };
                s += if check_row {
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
                s += "┃";
            }
            s += divider;
        }
        s.truncate(s.len() - divider.len());
        s += "\n┗";
        s += &String::from_iter(std::iter::repeat("━━━┻").take(width - 1));
        s += "━━━┛\n ";
        s
    }

    fn at(&self, loc: Vec2) -> Option<Spot> {
        if loc.0 < self.dim.0 && loc.1 < self.dim.1 {
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

    fn up(&self, loc: &Vec2) -> Option<Vec2> {
        if loc.1 == 0 {
            return None;
        }
        let loc = (loc.0, loc.1 - 1);
        if let Some((t, s)) = self.at(loc) {
            if t != Terrain::Water && s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn down(&self, loc: &Vec2) -> Option<Vec2> {
        let loc = (loc.0, loc.1 + 1);
        if let Some((t, s)) = self.at(loc) {
            if t != Terrain::Water && s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn left(&self, loc: &Vec2) -> Option<Vec2> {
        if loc.0 == 0 {
            return None;
        }
        let loc = (loc.0 - 1, loc.1);
        if let Some((t, s)) = self.at(loc) {
            if t != Terrain::Water && s == Status::None {
                return Some(loc);
            }
        }
        None
    }

    fn right(&self, loc: &Vec2) -> Option<Vec2> {
        let loc = (loc.0 + 1, loc.1);
        if let Some((t, s)) = self.at(loc) {
            if t != Terrain::Water && s == Status::None {
                return Some(loc);
            }
        }
        None
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.map_text();
        write!(f, "{string}")
    }
}

fn parse_line(reader: &mut BufReader<File>) -> Vec<String> {
    let mut line = String::new();
    reader.read_line(&mut line).expect("Error reading line");
    line.trim().split(' ').map(|x| x.to_string()).collect()
}

fn breadth_first(map: &Map) {
    let mut f = File::create("results/breadth_first_resutls.txt").unwrap();

    let mut done = false;
    let mut map = map.clone();
    let start = map.start;
    let goal = map.goal;
    let mut q = VecDeque::<(usize, Vec2)>::new();

    map.map[start.1][start.0].1 = Status::Path;
    q.push_back((0, start));

    let mut layer_prev = 1;
    while let Some((layer, loc)) = q.pop_front() {
        if layer != layer_prev {
            println!("{:?}", map);
            f.write(map.map_text().as_bytes()).expect("Write failed");

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let tile = &mut map.map[loc.1][loc.0];

        tile.1 = match tile.1 {
            Status::Up(true) => Status::Up(false),
            Status::Down(true) => Status::Down(false),
            Status::Left(true) => Status::Left(false),
            Status::Right(true) => Status::Right(false),
            x => x,
        };

        if loc == goal {
            done = true;
            println!("{:?}", map);
            break;
        }

        if let Some(spot) = map.up(&loc) {
            map.at_mut(spot).unwrap().1 = Status::Down(true);
            q.push_back((layer + 1, spot));
        }
        if let Some(spot) = map.down(&loc) {
            map.at_mut(spot).unwrap().1 = Status::Up(true);
            q.push_back((layer + 1, spot));
        }
        if let Some(spot) = map.left(&loc) {
            map.at_mut(spot).unwrap().1 = Status::Right(true);
            q.push_back((layer + 1, spot));
        }
        if let Some(spot) = map.right(&loc) {
            map.at_mut(spot).unwrap().1 = Status::Left(true);
            q.push_back((layer + 1, spot));
        }
        layer_prev = layer;
    }

    if done {
        // Now do backtracking
        let mut cost = 0;
        let mut loc_opt = Some(goal);
        loop {
            match loc_opt {
                Some(loc) => {
                    loc_opt = map.follow(loc);
                    map.at_mut(loc).unwrap().1 = Status::Path;
                    cost += map.at(loc).unwrap().0.cost();
                    println!("{:?}", map);
                    f.write(map.map_text().as_bytes()).expect("Write failed");
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                None => break,
            }
        }
        println!("The path found costs {} and may not be optimal.", cost);
        f.write(format!("The path found costs {} and may not be optimal.", cost).as_bytes())
            .expect("Write failed");
    } else {
        println!("Breadth first search failed! No valid paths exist.");
        f.write("Breadth first search failed! No valid paths exist.".as_bytes())
            .expect("Write failed");
    }
}

/*
fn greedy_best_first(map: &Map) {
    let mut map = map.clone();
}
fn a_star_1(map: &Map) {
    let mut map = map.clone();
}
fn a_star_2(map: &Map) {
    let mut map = map.clone();
}
*/

fn main() {
    let map = Map::from_file_path("map-small.txt");
    println!("The map data has been read successfully: {:?}", map);
    breadth_first(&map);
}
