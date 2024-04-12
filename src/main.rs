use std::io::stdout;
use std::time::Duration;

use termion::clear;
use termion::color::Rgb;
// use termion::color::Rgb;
use rand::Rng;
use std::io::Write;
use std::thread;
use termion::async_stdin;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::style;

fn main() {
    // println!("Hello, world!");
    let mut out = stdout().into_raw_mode().unwrap();
    let mut keys = async_stdin().keys();
    let mut board = Board::new(15, 3);
    let mut elapsed = Duration::from_millis(0);
    let fall_rate = Duration::from_millis(500);
    write!(
        out,
        "{}{}{}{}",
        clear::All,
        style::Reset,
        cursor::Goto(1, 1),
        cursor::Hide
    );
    let mut tetrimino = Tetrimino::new(&board);
    let mut finished = false;
    'main: loop {
        if finished {
            if let Some(Ok(Key::Char('q'))) = keys.next() {
                continue;
            }
        }
        thread::sleep(Duration::from_millis(50));
        elapsed += Duration::from_millis(50);
        match keys.next() {
            Some(Ok(key)) => match key {
                Key::Char('q') => break,
                Key::Down => tetrimino.move_down(&board),
                Key::Left => tetrimino.move_left(&board),
                Key::Right => tetrimino.move_right(&board),
                Key::Up => tetrimino.rotate(&board),
                Key::Char(' ') => tetrimino.move_down_to_bottom(&board),
                _ => {}
            },
            _ => {}
        }

        if elapsed >= fall_rate {
            elapsed -= fall_rate;
            tetrimino.move_down(&board);
        }

        if !tetrimino.is_can_down(&board) {
            for (x, y) in tetrimino.blocks_not_free().iter() {
                board.add_block(Block::On(tetrimino.color()), *x, *y);
            }
            if usize::max_value() == board.erase() {
                finished = true;
                continue;
            }
            tetrimino = Tetrimino::new(&board);
        }
        board.draw(&mut out);
        tetrimino.draw(&mut out);
        write!(out, "{}", style::Reset);
        out.flush();
    }
    write!(
        out,
        "{}{}{}{}",
        clear::All,
        style::Reset,
        cursor::Goto(1, 1),
        cursor::Show
    );
    // board.draw_window();
}

#[derive(Clone, Copy)]
enum Block {
    Free,
    On(color::Rgb),
}

impl Block {
    fn new() -> Block {
        Block::Free
    }

    fn is_free(&self) -> bool {
        match *self {
            Block::Free => true,
            _ => false,
        }
    }

    fn draw<W: Write>(&self, w: &mut W, x: u16, y: u16) {
        let _ = match self {
            Block::Free => write!(w, "{}{}  ", cursor::Goto(x, y), style::Reset),
            Block::On(c) => write!(w, "{}{}  ", cursor::Goto(x, y), color::Bg(*c)),
        };
    }

    // fn erase(&mut self) {
    //     let mut b = Block::Free;
    //     self = b;
    // }
}

#[derive(Clone, Copy)]
struct Line {
    blocks: [Block; 10],
}

impl Line {
    fn new() -> Line {
        Line {
            blocks: [Block::new(); 10],
        }
    }

    fn draw<W: Write>(&self, w: &mut W, x: u16, y: u16) {
        self.blocks.iter().enumerate().for_each(|(b_x, b)| {
            b.draw(w, x + b_x as u16 * 2, y);
        });
    }
    fn draw_<W: Write>(&self, w: &mut W, x: u16, y: u16) {
        for (b_x, b) in self.blocks.iter().enumerate() {
            b.draw(w, x + (b_x as u16) * 2, y)
        }
    }

    fn is_should_erase(&self) -> bool {
        for b in self.blocks {
            if b.is_free() {
                return false;
            }
        }
        return true;
    }

    fn is_all_free(&self) -> bool {
        for b in self.blocks {
            if !b.is_free() {
                return false;
            }
        }
        return true;
    }

    fn add(&mut self, b: Block, x: u16) {
        self.blocks[x as usize] = b;
    }

    fn is_free_on_x(&self, x: u16) -> bool {
        self.blocks[x as usize].is_free()
    }
}

struct Board {
    lines: [Line; 20],
    x: u16,
    y: u16,
}

impl Board {
    fn new(x: u16, y: u16) -> Self {
        Board {
            lines: [Line::new(); 20],
            x,
            y,
        }
    }

    fn is_conflict(&self, blocks: &[(u16, u16); 4]) -> bool {
        for (x, y) in blocks.iter() {
            if let Block::On(_) = self.lines[*y as usize].blocks[*x as usize] {
                return true;
            }
        }
        return false;
    }
    fn draw<W: Write>(&self, w: &mut W) {
        draw_window(
            w,
            self.x - 1 as u16,
            self.y - 1,
            (self.lines[0].blocks.len() * 2 + 2) as u16,
            (self.lines.len() + 2) as u16,
        );
        self.lines
            .iter()
            .enumerate()
            .for_each(|(y_l, l)| l.draw(w, self.x, self.y + (y_l as u16)));
    }

    fn add_block(&mut self, b: Block, x: u16, y: u16) {
        self.lines[y as usize].add(b, x);
    }

    fn is_free_on_xy(&self, x: u16, y: u16) -> bool {
        self.lines[y as usize].is_free_on_x(x)
    }

    fn erase(&mut self) -> usize {
        let num_free: usize = self.lines.iter().filter(|l| l.is_all_free()).count();
        if num_free < 1 {
            return usize::max_value();
        }
        let mut num_erase: usize = 0;
        let mut y = self.lines.len() - 1;
        while y > num_free {
            if self.lines[y].is_should_erase() {
                num_erase += 1;
            } else {
                y -= 1;
            }
            if num_erase > 0 {
                self.lines[y] = self.lines[y - num_erase];
            }
        }
        num_erase
    }
}

const BLOCK_I: [[[u8; 4]; 4]; 4] = [
    [[0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0], [0, 0, 0, 0]],
    [[0, 0, 1, 0], [0, 0, 1, 0], [0, 0, 1, 0], [0, 0, 1, 0]],
    [[0, 0, 0, 0], [0, 0, 0, 0], [1, 1, 1, 1], [0, 0, 0, 0]],
    [[0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0]],
];

const BLOCK_T: [[[u8; 4]; 4]; 4] = [
    [[1, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
    [[0, 0, 1, 0], [0, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
    [[0, 0, 0, 0], [0, 1, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0]],
    [[1, 0, 0, 0], [1, 1, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
];

const BLOCK_L: [[[u8; 4]; 4]; 4] = [
    [[0, 1, 0, 0], [0, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
    [[0, 0, 0, 0], [1, 1, 1, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
    [[1, 1, 0, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
    [[0, 0, 1, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
];

const BLOCK_J: [[[u8; 4]; 4]; 4] = [
    [[0, 1, 0, 0], [0, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
    [[1, 0, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
    [[0, 1, 1, 0], [0, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
    [[0, 0, 0, 0], [1, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
];

const BLOCK_O: [[u8; 4]; 4] = [[0, 1, 1, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]];

const BLOCK_Z: [[[u8; 4]; 4]; 4] = [
    [[1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
    [[0, 0, 1, 0], [0, 1, 1, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
    [[0, 0, 0, 0], [1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0]],
    [[0, 1, 0, 0], [1, 1, 0, 0], [1, 0, 0, 0], [0, 0, 0, 0]],
];

const BLOCK_S: [[[u8; 4]; 4]; 4] = [
    [[0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
    [[0, 1, 0, 0], [0, 1, 1, 0], [0, 0, 1, 0], [0, 0, 0, 0]],
    [[0, 0, 0, 0], [0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0]],
    [[1, 0, 0, 0], [1, 1, 0, 0], [0, 1, 0, 0], [0, 0, 0, 0]],
];

#[derive(Clone, Copy)]
enum Type {
    I,
    S,
    Z,
    O,
    J,
    L,
    T,
    // I S Z O J L T
}

#[derive(Clone, Copy)]
struct Tetrimino {
    tty: Type,
    state: u8,
    x: i32,
    y: i32,
    base_x: u16,
    base_y: u16,
}

impl Tetrimino {
    fn new(board: &Board) -> Self {
        let mut rng = rand::thread_rng();
        let n: u8 = rng.gen();
        let pieces = [
            Type::I,
            Type::O,
            Type::T,
            Type::S,
            Type::Z,
            Type::J,
            Type::L,
        ];
        let mut t = Tetrimino {
            tty: pieces[(n % 7) as usize],
            state: 3,
            x: 10 / 2 - 2,
            y: -4,
            base_x: board.x,
            base_y: board.y,
        };
        while t.y < 0 && t.is_can_down(board) {
            t.move_down(board);
        }
        t
    }

    fn blocks_not_free(&self) -> [(u16, u16); 4] {
        let pv: Vec<(u16, u16)> = self
            .blocks()
            .iter()
            .enumerate()
            .filter(|(y, _)| (self.y + *y as i32) > 0)
            .flat_map(|(y, l)| {
                l.iter()
                    .enumerate()
                    .filter(|(_, b)| **b == 1)
                    .map(|(x, _)| ((self.x + x as i32) as u16, (self.y + y as i32) as u16))
            })
            .collect();
        pv.try_into().unwrap_or([(0, 0); 4])
    }
    fn blocks_not_free_(&self) -> [(u16, u16); 4] {
        let mut blocks: [(u16, u16); 4] = [(0, 0); 4];
        let mut index = 0;
        for (y, l) in self.blocks().iter().enumerate() {
            for (x, b) in l.iter().enumerate() {
                if *b == 1 && (self.y + y as i32) >= 0 {
                    blocks[index] = ((self.x + x as i32) as u16, (self.y + y as i32) as u16);
                    index = index + 1;
                }
            }
        }
        blocks
    }

    fn un_rotate(&mut self) {
        self.state = match self.state {
            0 => 3,
            n => n - 1,
        }
    }
    fn rotate(&mut self, board: &Board) {
        self.state = match self.state {
            3 => 0,
            n => n + 1,
        };
        if board.is_conflict(&self.blocks_not_free()) {
            self.un_rotate();
        }
    }

    fn color(&self) -> Rgb {
        match self.tty {
            Type::I => Rgb(255, 0, 0),
            Type::J => Rgb(0, 255, 0),
            Type::L => Rgb(0, 0, 255),
            Type::O => Rgb(255, 125, 0),
            Type::S => Rgb(0, 255, 125),
            Type::T => Rgb(125, 0, 255),
            Type::Z => Rgb(125, 255, 125),
        }
    }

    fn blocks(&self) -> &'static [[u8; 4]; 4] {
        match self.tty {
            Type::I => BLOCK_I.get(self.state as usize).unwrap(),
            Type::J => BLOCK_J.get(self.state as usize).unwrap(),
            Type::L => BLOCK_L.get(self.state as usize).unwrap(),
            Type::O => &BLOCK_O,
            Type::S => BLOCK_S.get(self.state as usize).unwrap(),
            Type::T => BLOCK_T.get(self.state as usize).unwrap(),
            Type::Z => BLOCK_Z.get(self.state as usize).unwrap(),
        }
    }

    fn blocks_rang_on_x(&self) -> (usize, usize) {
        let xs: Vec<usize> = self
            .blocks()
            .iter()
            .flat_map(|l| {
                l.iter()
                    .enumerate()
                    .filter(|(_, b)| b == &&1)
                    .map(|(x, _)| x)
            })
            .collect();
        (
            *xs.iter().min().unwrap_or(&4),
            *xs.iter().max().unwrap_or(&0),
        )
    }
    fn blocks_rang_on_x_(&self) -> (usize, usize) {
        let mut r: (usize, usize) = (4, 0);
        self.blocks().iter().for_each(|l| {
            for (i, b) in l.iter().enumerate() {
                if *b == 1 {
                    if i < r.0 {
                        r.0 = i;
                    }
                    if i > r.1 {
                        r.1 = i;
                    }
                }
            }
        });
        r
    }

    fn blocks_rang_on_y(&self) -> (usize, usize) {
        let ys: Vec<usize> = self
            .blocks()
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains(&1))
            .map(|(y, _)| y)
            .collect();
        (
            *ys.iter().min().unwrap_or(&4),
            *ys.iter().max().unwrap_or(&0),
        )
    }
    fn blocks_rang_on_y_(&self) -> (usize, usize) {
        let mut r: (usize, usize) = (4, 0);
        for (i, l) in self.blocks().iter().enumerate() {
            if l.contains(&1) {
                if i < r.0 {
                    r.0 = i;
                }
                if i > r.1 {
                    r.1 = i;
                }
            }
        }
        r
    }

    fn move_left(&mut self, board: &Board) {
        let (l, _) = self.blocks_rang_on_x();
        if self.x + (l as i32) > 0 {
            self.x -= 1;
            if board.is_conflict(&self.blocks_not_free()) {
                self.x += 1;
            }
        }
    }

    fn move_right(&mut self, board: &Board) {
        let (_, r) = self.blocks_rang_on_x();
        if self.x + (r as i32) < 10 - 1 {
            self.x += 1;
            if board.is_conflict(&self.blocks_not_free()) {
                self.x -= 1;
            }
        }
    }

    fn is_can_down(&mut self, board: &Board) -> bool {
        let (_, d) = self.blocks_rang_on_y();
        if self.y as i32 + (d as i32) < 20 - 1 {
            self.y += 1;
            let blocks = self.blocks_not_free();
            self.y -= 1;
            if board.is_conflict(&blocks) {
                return false;
            }
            return true;
        }
        return false;
    }

    fn move_down_to_bottom(&mut self, board: &Board) {
        while self.is_can_down(board) {
            self.y = self.y + 1;
        }
    }

    fn move_down(&mut self, board: &Board) {
        if self.is_can_down(board) {
            self.y = self.y + 1;
        }
    }

    fn draw<W: Write>(&self, w: &mut W) {
        self.blocks().iter().enumerate().for_each(|(y, l)| {
            l.iter()
                .enumerate()
                .filter(|(x, b)| **b as i32 == 1 && self.y + (y as i32) >= 0)
                .for_each(|(x, b)| {
                    write!(
                        w,
                        "{}{}  {}",
                        cursor::Goto(
                            (self.base_x as i32 + self.x * 2 + x as i32 * 2) as u16,
                            (self.base_y as i32 + self.y + y as i32) as u16
                        ),
                        color::Bg(self.color()),
                        style::Reset
                    );
                })
        });
    }
    fn draw_<W: Write>(&self, w: &mut W) {
        let blocks = self.blocks();

        for (y_b, l) in blocks.iter().enumerate() {
            for (x_b, b) in l.iter().enumerate() {
                if *b == 1 && self.y + (y_b as i32) >= 0 {
                    write!(
                        w,
                        "{}{}  {}",
                        cursor::Goto(
                            (self.base_x as i32 + self.x * 2 + x_b as i32 * 2) as u16,
                            (self.base_y as i32 + self.y + y_b as i32) as u16
                        ),
                        color::Bg(self.color()),
                        style::Reset
                    );
                }
            }
        }
    }
}

const TOP_LEFT_CORNER: &'static str = "╔";
const TOP_RIGHT_CORNER: &'static str = "╗";
const BOTTOM_LEFT_CORNER: &'static str = "╚";
const BOTTOM_RIGHT_CORNER: &'static str = "╝";
const VERTICAL_WALL: &'static str = "║";
const HORIZONTAL_WALL: &'static str = "═";

fn draw_window<W: Write>(w: &mut W, x: u16, y: u16, width: u16, height: u16) {
    write!(w, "{}{}", cursor::Goto(x, y), TOP_LEFT_CORNER);
    write!(w, "{}{}", cursor::Goto(x + width - 1, y), TOP_RIGHT_CORNER);
    write!(
        w,
        "{}{}",
        cursor::Goto(x, y + height - 1),
        BOTTOM_LEFT_CORNER
    );
    write!(
        w,
        "{}{}",
        cursor::Goto(x + width - 1, y + height - 1),
        BOTTOM_RIGHT_CORNER
    );

    for i in 1..width - 1 {
        write!(w, "{}{}", cursor::Goto(x + i as u16, y), HORIZONTAL_WALL);
        write!(
            w,
            "{}{}",
            cursor::Goto(x + i as u16, y + height - 1),
            HORIZONTAL_WALL
        );
    }

    for i in 1..height - 1 {
        write!(w, "{}{}", cursor::Goto(x, y + i as u16), VERTICAL_WALL);
        write!(
            w,
            "{}{}",
            cursor::Goto(x + width - 1, y + i as u16),
            VERTICAL_WALL
        );
    }
}
