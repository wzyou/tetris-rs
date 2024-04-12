use std::io::stdout;
use std::time::Duration;

use termion::clear;
use termion::color::Rgb;
// use termion::color::Rgb;
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
    let mut tetrimino = Tetrimino::new(15, 3);
    'main: loop {
        thread::sleep(Duration::from_millis(50));
        elapsed += Duration::from_millis(50);
        match keys.next() {
            Some(Ok(key)) => match key {
                Key::Char('q') => break,
                Key::Down => tetrimino.move_down(&board),
                Key::Left => tetrimino.move_left(&board),
                Key::Right => tetrimino.move_right(&board),
                Key::Up => tetrimino.rotate(&board),
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
                board.add_block(Block::On(color::Rgb(255, 0, 0)), *x, *y);
            }
            tetrimino = Tetrimino::new(15, 3);
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
        match self {
            Block::Free => true,
            _ => false,
        }
    }

    fn draw<W: Write>(&self, w: &mut W, x: u16, y: u16) {
        let _ = match self {
            Block::Free => write!(w, "{}{}  ", cursor::Goto(x, y), style::Reset),
            Block::On(_) => write!(w, "{}{}  ", cursor::Goto(x, y), color::Bg(color::Blue)),
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
        for (y_l, l) in self.lines.iter().enumerate() {
            l.draw(w, self.x, self.y + (y_l as u16))
        }
    }

    fn add_block(&mut self, b: Block, x: u16, y: u16) {
        self.lines[y as usize].add(b, x);
    }

    fn is_free_on_xy(&self, x: u16, y: u16) -> bool {
        self.lines[y as usize].is_free_on_x(x)
    }

    fn erase(&mut self) -> usize {
        for i_l in 0..self.lines.len() {
            if self.lines[i_l].is_all_free() || self.lines[i_l].is_should_erase() {
                if i_l < self.lines.len() - 1 {
                    self.lines[i_l] = self.lines[i_l + 1];
                } else {
                    self.lines[i_l] = Line::new();
                }
            }
        }
        0
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
    y: u16,
    base_x: u16,
    base_y: u16,
}

impl Tetrimino {
    fn new(base_x: u16, base_y: u16) -> Self {
        Tetrimino {
            tty: Type::I,
            state: 3,
            x: 10 / 2 - 2,
            y: 0,
            base_x,
            base_y,
        }
    }

    fn blocks_not_free(&self) -> [(u16, u16); 4] {
        let mut blocks: [(u16, u16); 4] = [(0, 0); 4];
        let mut index = 0;
        for (y, l) in self.blocks().iter().enumerate() {
            for (x, b) in l.iter().enumerate() {
                if *b == 1 {
                    blocks[index] = ((self.x + x as i32) as u16, self.y + y as u16);
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
        let mut r: (usize, usize) = (4, 0);
        for l in self.blocks() {
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
        }
        r
    }

    fn blocks_rang_on_y(&self) -> (usize, usize) {
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
        if self.y + (d as u16) < 20 - 1 {
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

    fn move_down(&mut self, board: &Board) {
        if self.is_can_down(board) {
            self.y = self.y + 1;
        }
    }

    fn draw<W: Write>(&self, w: &mut W) {
        let blocks = self.blocks();
        let color = self.color();

        for (y_b, l) in blocks.iter().enumerate() {
            for (x_b, b) in l.iter().enumerate() {
                if *b == 1 {
                    write!(
                        w,
                        "{}{}  {}",
                        cursor::Goto(
                            (self.base_x as i32 + self.x * 2 + x_b as i32 * 2) as u16,
                            (self.base_y + self.y + y_b as u16) as u16
                        ),
                        color::Bg(color::Red),
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
