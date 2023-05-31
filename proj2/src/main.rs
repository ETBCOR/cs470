use rand::Rng;
use std::{
    collections::VecDeque,
    fmt::{Display, Formatter, Result},
    io::{self, Write},
};

const HEIGHT_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    row: isize,
    col: isize,
}

#[derive(Debug)]
struct Area {
    min: Vec2,
    max: Vec2,
}

const DIM: Vec2 = Vec2 { row: 6, col: 7 };

#[derive(Clone, Copy, PartialEq, Debug)]
enum Spot {
    None,
    O,
    X,
}

impl Default for Spot {
    fn default() -> Self {
        Self::None
    }
}

struct Grid<T: Clone + Copy + Default> {
    dim: Vec2,
    grid: Vec<Vec<T>>,
}

impl<T: Clone + Copy + Default> Grid<T> {
    fn new(dim: &Vec2) -> Option<Self> {
        if dim.row > 0 && dim.col > 0 {
            return Some(Self {
                dim: *dim,
                grid: vec![vec![T::default(); dim.col as usize]; dim.row as usize],
            });
        }
        None
    }

    fn at(&self, loc: &Vec2) -> Option<T> {
        if loc.row >= 0 && loc.col >= 0 && loc.row < self.dim.row && loc.col < self.dim.col {
            return Some(self.grid[loc.row as usize][loc.col as usize]);
        }
        None
    }

    fn at_mut(&mut self, loc: &Vec2) -> Option<&mut T> {
        if loc.row >= 0 && loc.col >= 0 && loc.row < self.dim.row && loc.col < self.dim.col {
            return Some(&mut self.grid[loc.row as usize][loc.col as usize]);
        }
        None
    }
}

#[derive(Debug)]
enum CheckDir {
    N,
    E,
    NE,
    SE,
}

#[derive(PartialEq)]
enum Score {
    None(isize),
    O,
    X,
}

struct GameBoard {
    grid: Grid<Spot>,
}

impl GameBoard {
    fn new(dim: &Vec2) -> Self {
        assert!(dim.row > 3, "Board doesn't have enough rows");
        assert!(dim.col > 3, "Board doesn't have enough columns");
        Self {
            grid: Grid::<Spot>::new(dim).expect("Invalid board dimensions"),
        }
    }

    fn play(&mut self) -> Spot {
        println!("Starting a game of connect 4!\n{}", self.as_text());

        loop {
            self.turn_human();
            match self.score(Spot::O) {
                Score::None(s) => {
                    println!("Human's current score: {}", s);
                }
                Score::O => {
                    println!("You won!");
                    return Spot::O;
                }
                Score::X => unreachable!(),
            }

            self.turn_bot();
            match self.score(Spot::X) {
                Score::None(s) => {
                    println!("Bot's current score: {}", s);
                }
                Score::X => {
                    println!("Bot won!");
                    return Spot::X;
                }
                Score::O => unreachable!(),
            }
        }
    }

    fn as_text(&self) -> String {
        let divider = &format!(
            "\n▐{}━━━▌\n▐",
            String::from_iter(std::iter::repeat("━━━╋").take(DIM.col as usize - 1))
        );
        let mut s = format!(
            "▗{}▄▄▄▖\n▐",
            String::from_iter(std::iter::repeat("▄▄▄▄").take(DIM.col as usize - 1))
        );

        for (r, row) in self.grid.grid.iter().rev().enumerate() {
            for (c, spot) in row.iter().enumerate() {
                s += match spot {
                    Spot::None => "   ",
                    Spot::O => " O ",
                    Spot::X => " X ",
                };
                s += if c == row.len() - 1 { "▌" } else { "┃" }
            }
            if r != DIM.row as usize - 1 {
                s += divider;
            }
        }
        s += &format!(
            "\n▝{}▀▀▀▘\n",
            String::from_iter(std::iter::repeat("▀▀▀▀").take(DIM.col as usize - 1))
        );
        s
    }

    fn turn_human(&mut self) {
        let col: isize = 'input: loop {
            print!("Your move (1-7 then ⏎): ");
            io::stdout().flush().expect("Couldn't flush stdout");
            let mut s = String::new();
            io::stdin().read_line(&mut s).expect("Read error");
            let vec: Vec<&str> = s.trim().split_whitespace().collect();
            if vec.len() != 1 {
                println!("Wrong number of inputs. Please input a single number (1-7).");
                continue;
            }
            match vec[0].parse::<isize>() {
                Ok(x) => {
                    if x <= 0 || x > DIM.col {
                        println!("Input is out of bounds. Please input a single number (1-7).");
                        continue;
                    } else if self.grid.at(&Vec2 {
                        row: self.grid.dim.row - 1,
                        col: x - 1,
                    }) != Some(Spot::None)
                    {
                        println!(
                            "No space to play in column {x}. Please input a different number (1-7)."
                        );
                        continue;
                    } else {
                        break 'input x - 1;
                    }
                }
                Err(_) => {
                    println!("Error parsing input. Please input a single number (1-7).");
                    continue;
                }
            }
        };
        self.drop_piece(Spot::O, col);
        println!("{}", self.as_text());
    }

    fn turn_bot(&mut self) {
        println!("Bot's move: ");

        let col = self.best_move(Spot::X, HEIGHT_LIMIT);

        self.drop_piece(Spot::X, col);
        println!("{}", self.as_text());
    }

    fn drop_piece(&mut self, spot: Spot, col: isize) {
        assert!(spot != Spot::None, "Cannot place a Spot::None");
        let mut loc = Vec2 {
            row: self.grid.dim.row - 1,
            col,
        };
        while self.grid.at(&loc) == Some(Spot::None) {
            loc.row -= 1;
        }
        loc.row += 1;
        *self.grid.at_mut(&loc).unwrap() = spot;
    }

    fn score(&self, spot: Spot) -> Score {
        assert!(spot != Spot::None, "Cannont score for Spot::None");
        let mut score = 0;

        let s = self.score_area(
            Area {
                min: Vec2 { row: 0, col: 0 },
                max: Vec2 {
                    row: self.grid.dim.row - 4,
                    col: self.grid.dim.col - 1,
                },
            },
            CheckDir::N,
            spot,
        );
        if let Score::None(s) = s {
            score += s;
        } else {
            return s;
        }
        let s = self.score_area(
            Area {
                min: Vec2 { row: 0, col: 0 },
                max: Vec2 {
                    row: self.grid.dim.row - 1,
                    col: self.grid.dim.col - 4,
                },
            },
            CheckDir::E,
            spot,
        );
        if let Score::None(s) = s {
            score += s;
        } else {
            return s;
        }
        let s = self.score_area(
            Area {
                min: Vec2 { row: 0, col: 0 },
                max: Vec2 {
                    row: self.grid.dim.row - 4,
                    col: self.grid.dim.col - 4,
                },
            },
            CheckDir::NE,
            spot,
        );
        if let Score::None(s) = s {
            score += s;
        } else {
            return s;
        }
        let s = self.score_area(
            Area {
                min: Vec2 { row: 3, col: 0 },
                max: Vec2 {
                    row: self.grid.dim.row - 1,
                    col: self.grid.dim.col - 4,
                },
            },
            CheckDir::SE,
            spot,
        );
        if let Score::None(s) = s {
            score += s;
        } else {
            return s;
        }
        Score::None(score)
    }

    fn score_area(&self, area: Area, dir: CheckDir, spot: Spot) -> Score {
        // println!("Scoring area (area: {:?}, dir: {:?}, spot: {:?}", area, dir, spot);
        let mut score = 0;
        let mut row = area.min.row;
        let mut col = area.min.col;

        while row <= area.max.row {
            while col <= area.max.col {
                let s = self.score_pos(Vec2 { row, col }, &dir, &spot);
                if let Score::None(s) = s {
                    score += s;
                } else {
                    return s;
                }
                col += 1;
            }
            row += 1;
        }
        Score::None(score)
    }

    fn score_pos(&self, loc: Vec2, dir: &CheckDir, spot: &Spot) -> Score {
        // println!("Scoring spot (loc: {:?}, dir: {:?}, spot: {:?}", loc, dir, spot);
        let row = loc.row as usize;
        let col = loc.col as usize;
        let mut line_vec = Vec::<Spot>::new();
        let line: &[Spot] = match dir {
            CheckDir::N => {
                for x in 0..4 {
                    line_vec.push(
                        self.grid
                            .at(&Vec2 {
                                row: row as isize + x,
                                col: col as isize,
                            })
                            .unwrap(),
                    );
                }
                &line_vec[..]
            }
            CheckDir::E => &self.grid.grid[row][col..col + 4],
            CheckDir::NE => {
                for x in 0..4 {
                    line_vec.push(
                        self.grid
                            .at(&Vec2 {
                                row: row as isize + x,
                                col: col as isize + x,
                            })
                            .unwrap(),
                    );
                }
                &line_vec[..]
            }
            CheckDir::SE => {
                for x in 0..4 {
                    line_vec.push(
                        self.grid
                            .at(&Vec2 {
                                row: row as isize - x,
                                col: col as isize + x,
                            })
                            .unwrap(),
                    );
                }
                &line_vec[..]
            }
        };
        // println!("Line: {:?}", line);

        if line[0] != Spot::None && line[0] == line[1] && line[1] == line[2] && line[2] == line[3] {
            // four in a row
            if line[0] == Spot::O {
                return Score::O;
            } else {
                return Score::X;
            }
        } else {
            return Score::None(
                if line[0] != Spot::None && line[0] == line[1] && line[1] == line[2] {
                    // three in a row
                    (if line[3] == Spot::None {
                        // (forth spot open)
                        16
                    } else {
                        // (forth spot taken)
                        4
                    }) * if line[0] == *spot { 1 } else { -1 }
                } else if line[1] != Spot::None && line[1] == line[2] && line[2] == line[3] {
                    // three in a row (mirrored)
                    (if line[3] == Spot::None {
                        100 // (forth spot open)
                    } else {
                        8 // (forth spot taken)
                    }) * if line[1] == *spot { 1 } else { -1 }
                } else if line[0] != Spot::None && line[0] == line[1] {
                    // two in a row (start)
                    (if line[2] == Spot::None {
                        // (third spot open)
                        if line[3] == Spot::None {
                            // (fourth spot open)
                            4
                        } else {
                            // (fourth spot taken)
                            2
                        }
                    } else {
                        // (third spot taken)
                        1
                    }) * if line[0] == *spot { 1 } else { -1 }
                } else if line[1] != Spot::None && line[1] == line[2] {
                    // two in a row (middle)
                    (if line[0] == Spot::None && line[3] == Spot::None {
                        // (both ends open)
                        8
                    } else if line[0] == Spot::None || line[3] == Spot::None {
                        // (one end open)
                        2
                    } else {
                        // (both ends taken)
                        1
                    }) * if line[1] == *spot { 1 } else { -1 }
                } else if line[2] != Spot::None && line[2] == line[3] {
                    // two in a row (end)
                    (if line[1] == Spot::None {
                        // (third spot open)
                        if line[0] == Spot::None {
                            // (fourth spot open)
                            4
                        } else {
                            // (fourth spot taken)
                            2
                        }
                    } else {
                        // (third spot taken)
                        1
                    }) * if line[2] == *spot { 1 } else { -1 }
                } else {
                    // nothing special
                    0
                },
            );
        }
    }

    fn best_move(&self, spot: Spot, height: usize) -> isize {
        // randomly choose column (check that it's available)
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(0..7);
            if self.grid.at(&Vec2 {
                row: self.grid.dim.row - 1,
                col: x,
            }) == Some(Spot::None)
            {
                break x;
            }
        }
    }
}

impl Display for GameBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = self.as_text();
        write!(f, "{s}")
    }
}

fn main() {
    let mut board = GameBoard::new(&DIM);
    _ = board.play();
}
