use std::io::Write;

use crossterm::{
    cursor,
    event::{Event, KeyCode},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal,
};

const LIVING: char = '■';
const DEAD: char = '□';

const DIRECTIONS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

struct Game {
    field: Vec<Vec<char>>,
    width: u16,
    height: u16,
    stop: bool,
    cursor: (u16, u16),
}

impl Game {
    fn try_new() -> anyhow::Result<Game> {
        let (width, height) = terminal::size()?;
        Ok(Game {
            field: vec![vec![DEAD; width as usize]; height as usize],
            width,
            height,
            stop: true,
            cursor: (0, 0),
        })
    }

    fn print_field(&self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout();
        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            Print(
                "Press 'q' to quit, 's' to stop, 'space' to toggle cell, arrow keys to move cursor"
            )
        )?;
        let (width, height) = terminal::size()?;
        for y in 1..height.min(self.height) {
            for x in 0..width.min(self.width) {
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    SetForegroundColor(if self.cursor == (x, y - 1) {
                        Color::Cyan
                    } else {
                        Color::Reset
                    }),
                    Print(self.field[y as usize - 1][x as usize])
                )?;
            }
        }

        stdout.flush()?;

        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        if self.stop {
            self.print_field()?;
            return Ok(());
        }

        let mut new_field = vec![vec![DEAD; self.width.into()]; self.height.into()];

        for (y, row) in new_field.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let mut count = 0;
                for &(dx, dy) in DIRECTIONS.iter() {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (x as i32 + dx + self.width as i32) as u16 % self.width;
                    let ny = (y as i32 + dy + self.height as i32) as u16 % self.height;

                    if self
                        .field
                        .get(ny as usize)
                        .and_then(|row| row.get(nx as usize))
                        .map(|cell| *cell == LIVING)
                        .unwrap_or(false)
                    {
                        count += 1;
                    }
                }
                if self
                    .field
                    .get(y)
                    .and_then(|row| row.get(x))
                    .map(|cell| *cell == LIVING)
                    .unwrap_or(false)
                {
                    if count == 2 || count == 3 {
                        *cell = LIVING;
                    }
                } else if count == 3 {
                    *cell = LIVING;
                }
            }
        }
        self.field = new_field;

        self.print_field()?;

        {
            let (width, height) = terminal::size()?;
            self.width = width;
            self.height = height;
            self.cursor = (
                self.cursor.0.min(self.width - 1),
                self.cursor.1.min(self.height - 1),
            );
        }

        Ok(())
    }

    fn toggle_cell(&mut self) {
        let (x, y) = self.cursor;
        self.field[y as usize][x as usize] = if self.field[y as usize][x as usize] == LIVING {
            DEAD
        } else {
            LIVING
        };
    }
}

fn main() -> anyhow::Result<()> {
    let mut game = Game::try_new()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, cursor::Hide, terminal::EnterAlternateScreen,)?;
    terminal::enable_raw_mode()?;

    loop {
        game.update()?;
        if let Event::Key(event) = crossterm::event::read()? {
            match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('s') => game.stop = !game.stop,
                KeyCode::Char(' ') => game.toggle_cell(),
                KeyCode::Up | KeyCode::Char('k') => {
                    if 0 < game.cursor.1 {
                        game.cursor.1 -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if game.cursor.1 < game.height - 1 {
                        game.cursor.1 += 1;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if 0 < game.cursor.0 {
                        game.cursor.0 -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if game.cursor.0 < game.width - 1 {
                        game.cursor.0 += 1;
                    }
                }
                _ => {}
            }
        }
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen,)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
