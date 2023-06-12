use rand::Rng;
use std::{
    fmt::{Debug, Display, Formatter, Result},
    io::{self, Write},
};

const MAX_DEPTH: usize = 4;

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
    Empty,
    O,
    X,
}

impl Default for Spot {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Clone)]
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

#[derive(Debug, PartialEq)]
enum Score {
    InProgress(isize),
    O,
    X,
}

impl PartialEq<Score> for Spot {
    fn eq(&self, other: &Score) -> bool {
        match other {
            Score::InProgress(_) if *self == Spot::Empty => true,
            Score::O if *self == Spot::O => true,
            Score::X if *self == Spot::X => true,
            _ => false,
        }
    }
}

impl PartialEq<Spot> for Score {
    fn eq(&self, other: &Spot) -> bool {
        match self {
            Score::InProgress(_) if *other == Spot::Empty => true,
            Score::O if *other == Spot::O => true,
            Score::X if *other == Spot::X => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
struct Move {
    col: isize,
    board: Box<GameBoard>,
    score: isize,
}

type Moves = Vec<Move>;

#[derive(Clone)]
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

    fn play(&mut self) -> Score {
        println!("Starting a game of connect 4!");
        if {
            loop {
                println!("Who should go first? (O: human, X: bot)");
                io::stdout().flush().expect("Couldn't flush stdout");
                let mut s = String::new();
                io::stdin().read_line(&mut s).expect("Read error");
                let s = s.trim().to_uppercase();
                if s == "O" {
                    break false;
                } else if s == "X" {
                    break true;
                } else {
                    println!("Invalid input.");
                    continue;
                }
            }
        } {
            self.turn_bot();
        } else {
            println!("{}", self.as_text());
        }
        loop {
            if self.open_spot() {
                self.turn_human();
                match self.score() {
                    Score::InProgress(_) => (),
                    s => return s,
                }
            } else {
                return self.score();
            }
            if self.open_spot() {
                self.turn_bot();
                match self.score() {
                    Score::InProgress(_) => (),
                    s => return s,
                }
            } else {
                return self.score();
            }
        }
    }

    fn as_text(&self) -> String {
        let divider = &format!(
            "\n▐{}━━━▌\n▐",
            String::from_iter(std::iter::repeat("━━━╋").take(self.grid.dim.col as usize - 1))
        );
        let mut s = format!(
            "\n▗{}▄▄▄▖\n▐",
            String::from_iter(std::iter::repeat("▄▄▄▄").take(self.grid.dim.col as usize - 1))
        );

        for (r, row) in self.grid.grid.iter().rev().enumerate() {
            for (c, spot) in row.iter().enumerate() {
                s += match spot {
                    Spot::Empty => "   ",
                    Spot::O => " O ",
                    Spot::X => " X ",
                };
                s += if c == row.len() - 1 { "▌" } else { "┃" }
            }
            if r != self.grid.dim.row as usize - 1 {
                s += divider;
            }
        }
        s += &format!(
            "\n▝{}▀▀▀▘\n",
            String::from_iter(std::iter::repeat("▀▀▀▀").take(self.grid.dim.col as usize - 1))
        );
        s
    }

    fn open_spot(&self) -> bool {
        self.grid.grid[self.grid.dim.row as usize - 1]
            .iter()
            .any(|&x| x == Spot::Empty)
    }

    fn turn_human(&mut self) {
        let col: isize = 'input: loop {
            print!("Your move (1-{} then ⏎): ", self.grid.dim.col);
            io::stdout().flush().expect("Couldn't flush stdout");
            let mut s = String::new();
            io::stdin().read_line(&mut s).expect("Read error");
            let vec: Vec<&str> = s.trim().split_whitespace().collect();
            if vec.len() != 1 {
                println!("Wrong number of inputs. Please input a single number.");
                continue;
            }
            match vec[0].parse::<isize>() {
                Ok(x) => {
                    if x <= 0 || x > DIM.col {
                        println!(
                            "Input is out of bounds. Please input a single number (1-{}).",
                            self.grid.dim.col
                        );
                        continue;
                    } else if self.grid.at(&Vec2 {
                        row: self.grid.dim.row - 1,
                        col: x - 1,
                    }) != Some(Spot::Empty)
                    {
                        println!(
                            "No space to play in column {}. Please input a different number (1-{}).",
                            x,
                            self.grid.dim.col
                        );
                        continue;
                    } else {
                        break 'input x - 1;
                    }
                }
                Err(_) => {
                    println!("Error parsing input. Please input a number.");
                    continue;
                }
            }
        };
        self.drop_piece(Spot::O, col);
        println!("{}", self.as_text());
    }

    fn turn_bot(&mut self) {
        let col = self
            .best_move(Spot::X, MAX_DEPTH)
            .expect("The bot has no valid positions to play!")
            .col;
        println!("Bot's move: {}", col + 1);
        self.drop_piece(Spot::X, col);
        println!("{}", self.as_text());
    }

    fn drop_piece(&mut self, spot: Spot, col: isize) -> bool {
        assert!(spot != Spot::Empty, "Cannot place a Spot::None");
        let mut loc = Vec2 {
            row: self.grid.dim.row - 1,
            col,
        };
        while self.grid.at(&loc) == Some(Spot::Empty) {
            loc.row -= 1;
        }
        loc.row += 1;
        if let Some(x) = self.grid.at_mut(&loc) {
            *x = spot;
            return true;
        }
        false
    }

    fn drop_piece_new_board(&self, spot: Spot, col: isize) -> Option<Box<GameBoard>> {
        let mut gb = Box::new(self.clone());
        if gb.drop_piece(spot, col) {
            Some(gb)
        } else {
            None
        }
    }

    fn score(&self, spot: Spot) -> Score {
        assert!(spot != Spot::Empty, "Cannont score for Spot::None");
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
        if let Score::InProgress(s) = s {
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
        if let Score::InProgress(s) = s {
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
        if let Score::InProgress(s) = s {
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
        if let Score::InProgress(s) = s {
            score += s;
        } else {
            return s;
        }
        Score::InProgress(score)
    }

    fn score_area(&self, area: Area, dir: CheckDir, spot: Spot) -> Score {
        let mut score = 0;
        let mut row = area.min.row;

        while row <= area.max.row {
            let mut col = area.min.col;
            while col <= area.max.col {
                let s = self.score_pos(Vec2 { row, col }, &dir, &spot);
                if let Score::InProgress(s) = s {
                    score += s;
                } else {
                    return s;
                }
                col += 1;
            }
            row += 1;
        }
        Score::InProgress(score)
    }

    fn score_pos(&self, loc: Vec2, dir: &CheckDir, spot: &Spot) -> Score {
        let row = loc.row as usize;
        let col = loc.col as usize;
        let mut line_vec = Vec::<Spot>::new();
        let mut under_vec = Vec::<bool>::new();
        let line: &[Spot];
        let under: &[bool];
        match dir {
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
                line = &line_vec[..];
                under_vec = vec![true; 4];
            }
            CheckDir::E => {
                line = &self.grid.grid[row][col..col + 4];
                if row > 0 {
                    for c in col..col + 4 {
                        let loc = Vec2 {
                            row: row as isize - 1,
                            col: c as isize,
                        };
                        match self.grid.at(&loc) {
                            Some(Spot::Empty) => under_vec.push(false),
                            Some(_) | None => under_vec.push(true),
                        }
                    }
                } else {
                    under_vec = vec![true; 4];
                }
            }
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
                    under_vec.push(
                        match self.grid.at(&Vec2 {
                            row: row as isize + x - 1,
                            col: col as isize + x,
                        }) {
                            Some(Spot::Empty) => false,
                            Some(_) | None => true,
                        },
                    );
                }
                line = &line_vec[..]
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
                    under_vec.push(
                        match self.grid.at(&Vec2 {
                            row: row as isize - x - 1,
                            col: col as isize + x,
                        }) {
                            Some(Spot::Empty) => false,
                            Some(_) | None => true,
                        },
                    );
                }
                line = &line_vec[..]
            }
        }
        under = &under_vec[..];
        let mut line_vec_rev = Vec::from(line);
        let mut under_vec_rev = Vec::from(under);
        line_vec_rev.reverse();
        under_vec_rev.reverse();
        let line_rev: &[Spot] = &line_vec_rev[..];
        let under_rev: &[bool] = &under_vec_rev[..];
        match (
            self.score_line(line, under, spot),
            self.score_line(line_rev, under_rev, spot),
        ) {
            (Score::InProgress(f), Score::InProgress(r)) => Score::InProgress(f + r),
            (w, Score::InProgress(_)) | (Score::InProgress(_), w) | (w, _) => w,
        }
    }

    fn score_line(&self, line: &[Spot], under: &[bool], spot: &Spot) -> Score {
        match (
            line[0],
            line[0] == line[1],
            line[1],
            line[1] == line[2],
            line[2],
            line[2] == line[3],
            line[3],
        ) {
            // 4 in a row
            (Spot::O, true, _, true, _, true, _) => Score::O, // O wins
            (Spot::X, true, _, true, _, true, _) => Score::X, // X wins
            // 3 in a row, 4th empty
            (s, true, _, true, _, false, Spot::Empty) if s != Spot::Empty && under[3] => {
                Score::InProgress(16 * if s == *spot { 1 } else { -1 })
            }
            // two in a row, two empty spots
            (s, true, _, _, Spot::Empty, true, _) if s != Spot::Empty && under[2] && under[3] => {
                Score::InProgress(8 * if s == *spot { 1 } else { -1 })
            }
            // two in a row, one empty spot, then an ally piece
            (s, true, _, _, Spot::Empty, _, o) if s != Spot::Empty && s == o && under[2] => {
                Score::InProgress(16 * if s == *spot { 1 } else { -1 })
            }
            // two in a row, one empty spot, then an aponent's piece
            (s, true, _, _, Spot::Empty, _, o) if s != Spot::Empty && s != o && under[2] => {
                Score::InProgress(4 * if s == *spot { 1 } else { -1 })
            }
            // two in a row (middle), empty ends
            (Spot::Empty, _, s, true, _, _, Spot::Empty)
                if s != Spot::Empty && under[0] && under[3] =>
            {
                Score::InProgress(4 * if s == *spot { 1 } else { -1 })
            }
            // two in a row (middle), one empty end
            (l, _, s, true, _, _, r)
                if s != Spot::Empty
                    && (l == Spot::Empty && under[0] && r != s)
                    && (r == Spot::Empty && under[3] && l != s) =>
            {
                Score::InProgress(2 * if s == *spot { 1 } else { -1 })
            }
            // 1 spot next to 3 free
            (s, _, Spot::Empty, true, _, true, _)
                if s != Spot::Empty && under[1] && under[2] && under[3] =>
            {
                Score::InProgress(4 * if s == *spot { 1 } else { -1 })
            }
            // 1 spot next to 2 free, then ally piece
            (s, _, Spot::Empty, true, _, _, o)
                if s != Spot::Empty && s == o && under[1] && under[2] =>
            {
                Score::InProgress(8 / 2 * if s == *spot { 1 } else { -1 })
            }
            // 1 spot next to 2 free
            (s, _, Spot::Empty, _, Spot::Empty, _, _)
                if s != Spot::Empty && under[1] && under[2] =>
            {
                Score::InProgress(2 * if s == *spot { 1 } else { -1 })
            }
            _ => Score::InProgress(0),
        }
    }

    fn best_move(&self, spot: Spot, depth: usize) -> Option<Move> {
        assert_ne!(
            spot,
            Spot::Empty,
            "Cannont evaluate the best move for Spot::Empty"
        );
        let mut moves: Moves = vec![];
        for col in 0..self.grid.dim.col {
            if let Some(new_board) = self.drop_piece_new_board(spot, col) {
                // piece drop successful
                let score: isize = match new_board.score(spot) {
                    Score::InProgress(s) => {
                        if depth == 0 {
                            // dont recurse
                            s
                        } else {
                            // recurse
                            let opospot = if spot == Spot::O { Spot::X } else { Spot::O };
                            if let Some(m) = new_board.best_move(opospot, depth - 1) {
                                -m.score
                            } else {
                                s // couldn't recurse
                            }
                        }
                    }
                    s => isize::MAX * if s == spot { 1 } else { -1 },
                };
                moves.push(Move {
                    col,
                    board: new_board,
                    score,
                });
            }
        }
        match moves.iter().max_by_key(|x| x.score) {
            Some(m) => Some(m.clone()),
            None => None,
        }
    }
}

impl Debug for GameBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = self.as_text();
        write!(f, "{s}")
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
    match board.play() {
        Spot::Empty => println!("Draw!"),
        Spot::O => println!("You won!"),
        Spot::X => println!("Bot won!"),
    }
}
