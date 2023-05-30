use std::{
    collections::VecDeque,
    fmt::{Display, Formatter, Result},
    io::{self, Write},
};

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    row: isize,
    col: isize,
}

const DIM: Vec2 = Vec2 { row: 6, col: 7 };

#[derive(Clone, Copy, PartialEq)]
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

struct GameBoard {
    grid: Grid<Spot>,
}

impl GameBoard {
    fn new(dim: &Vec2) -> Self {
        Self {
            grid: Grid::<Spot>::new(dim).expect("Invalid board dimensions"),
        }
    }

    fn play(&mut self) -> Spot {
        println!("Starting a game of connect 4!\n{}", self.as_text());

        loop {
            self.turn_human();
            if self.check_win() {
                println!("You won!");
                return Spot::O;
            }
            self.turn_bot();
            self.check_win();
            if self.check_win() {
                println!("The bot won!");
                return Spot::X;
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
        self.place(Spot::O, col);
        println!("{}", self.as_text());
    }

    fn turn_bot(&mut self) {
        println!("Bot's move: {}", 0);
        println!("{}", self.as_text());
    }

    fn place(&mut self, spot: Spot, col: isize) {
        assert!(spot != Spot::None, "Cannot place a Spot::None");
        let mut loc = Vec2 {
            row: self.grid.dim.row - 1,
            col,
        };
        println!("{:?}", loc);
        while self.grid.at(&loc) == Some(Spot::None) {
            println!("{:?}", loc);
            loc.row -= 1;
        }
        loc.row += 1;
        *self.grid.at_mut(&loc).unwrap() = spot;
    }

    fn check_win(&self) -> bool {
        let mut v = Grid::<bool>::new(&self.grid.dim).unwrap();
        let mut q: VecDeque<Vec2> = VecDeque::new();
        q.push_back(Vec2 {
            row: DIM.row - 1,
            col: 0,
        });
        while let Some(loc) = q.pop_front() {
            *v.at_mut(&loc).unwrap() = true;

            let up = Vec2 {
                row: loc.row + 1,
                col: loc.col,
            };
            let right = Vec2 {
                row: loc.row,
                col: loc.col + 1,
            };
            if let Some(true) = v.at(&up) {
                q.push_back(up);
            }
            if let Some(true) = v.at(&right) {
                q.push_back(right);
            }
        }
        false
    }

    fn check_win_n(&self, loc: Vec2) -> bool {
        false
    }

    fn check_win_ne(&self, loc: Vec2) -> bool {
        false
    }

    fn check_win_e(&self, loc: Vec2) -> bool {
        false
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
