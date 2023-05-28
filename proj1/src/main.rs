use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    vec,
};

#[derive(Debug)]
enum Tile {
    Road,
    Field,
    Forest,
    Hills,
    River,
    Mountians,
    Water,
}

impl Tile {
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
    fn cost(&self) -> Option<u32> {
        match self {
            Self::Road => Some(1),
            Self::Field => Some(2),
            Self::Forest => Some(4),
            Self::Hills => Some(5),
            Self::River => Some(7),
            Self::Mountians => Some(10),
            Self::Water => None,
        }
    }
}

enum Status {
    None,
    Up,
    Down,
    Left,
    Right,
    Explored,
    Open,
}

struct Map {
    map: Vec<Vec<(Tile, Status)>>,
    start: (u32, u32),
    goal: (u32, u32),
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.map.get(0).unwrap().len();
        let mut string = String::new();
        string += "\n┏";
        string += &String::from_iter(std::iter::repeat("━━━┳").take(width - 1));
        string += "━━━┓\n┃";

        let divider = "\n┣".to_string()
            + &String::from_iter(std::iter::repeat("━━━╋").take(width - 1))
            + "━━━┫\n┃";
        let divider = &divider;

        for (r, row) in (0u32..).zip(self.map.iter()) {
            let check_row = r == self.start.1 || r == self.goal.1;
            for (c, tile) in (0u32..).zip(row.iter()) {
                string += if check_row {
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
                string += match tile.0 {
                    Tile::Road => "R",
                    Tile::Field => "f",
                    Tile::Forest => "F",
                    Tile::Hills => "h",
                    Tile::River => "r",
                    Tile::Mountians => "M",
                    Tile::Water => "W",
                };
                string += match tile.1 {
                    Status::None => " ",
                    Status::Up => "⬆",
                    Status::Down => "⬇",
                    Status::Left => "⬅",
                    Status::Right => "⮕",
                    Status::Explored => "█",
                    Status::Open => "▒",
                };
                string += "┃";
            }
            string += divider;
        }
        string.truncate(string.len() - divider.len());
        string += "\n┗";
        string += &String::from_iter(std::iter::repeat("━━━┻").take(width - 1));
        string += "━━━┛\n ";

        write!(f, "{string}")
    }
}

fn parse_line(reader: &mut BufReader<File>) -> Vec<String> {
    let mut line = String::new();
    reader.read_line(&mut line).expect("Error reading line");
    line.trim().split(' ').map(|x| x.to_string()).collect()
}

fn read_map(path: &str) -> Map {
    let file = File::open(path).expect("Couldn't open file");
    let mut reader = BufReader::new(file);
    let dim = parse_line(&mut reader);
    assert_eq!(
        dim.len(),
        2,
        "Invalid number of arguments for map dimensions"
    );
    let dim: (u32, u32) = (
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
    let start: (u32, u32) = (
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
    let goal: (u32, u32) = (
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
        let mut row: Vec<(Tile, Status)> = vec![];
        for c in line.chars() {
            row.push((
                Tile::from(&c).expect("Could not parse map character"),
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

fn main() {
    let map = read_map("map.txt");

    println!("The map data has been read successfully: {:?}", map);
}
