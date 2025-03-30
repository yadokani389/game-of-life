use std::io::Write;

use crossterm::{
    cursor,
    event::{Event, KeyCode},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal,
};

const QUIT_KEY: char = 'q';
const STOP_KEY: char = 's';
const TOGGLE_KEY: char = ' ';
const UP_KEY_ALT: char = 'k';
const DOWN_KEY_ALT: char = 'j';
const LEFT_KEY_ALT: char = 'h';
const RIGHT_KEY_ALT: char = 'l';

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
                let live_neighbors = DIRECTIONS
                    .iter()
                    .filter(|&&(dx, dy)| self.is_alive_at(x as i32 + dx, y as i32 + dy))
                    .count();

                let current_cell_alive = self.is_alive_at(x as i32, y as i32);

                if current_cell_alive {
                    if live_neighbors == 2 || live_neighbors == 3 {
                        *cell = LIVING;
                    }
                } else if live_neighbors == 3 {
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

    fn is_alive_at(&self, x: i32, y: i32) -> bool {
        let nx = (x + self.width as i32) as u16 % self.width;
        let ny = (y + self.height as i32) as u16 % self.height;
        self.field
            .get(ny as usize)
            .and_then(|row| row.get(nx as usize))
            .map(|&cell| cell == LIVING)
            .unwrap_or(false)
    }

    fn handle_input(&mut self, event: Event) -> bool {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char(QUIT_KEY) => return false, // Indicate quit
                KeyCode::Char(STOP_KEY) => self.stop = !self.stop,
                KeyCode::Char(TOGGLE_KEY) => self.toggle_cell(),
                KeyCode::Up | KeyCode::Char(UP_KEY_ALT) => {
                    if 0 < self.cursor.1 {
                        self.cursor.1 -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char(DOWN_KEY_ALT) => {
                    if self.cursor.1 < self.height - 1 {
                        self.cursor.1 += 1;
                    }
                }
                KeyCode::Left | KeyCode::Char(LEFT_KEY_ALT) => {
                    if 0 < self.cursor.0 {
                        self.cursor.0 -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char(RIGHT_KEY_ALT) => {
                    if self.cursor.0 < self.width - 1 {
                        self.cursor.0 += 1;
                    }
                }
                _ => {}
            }
        }
        true // Indicate continue
    }
}

fn main() -> anyhow::Result<()> {
    let mut game = Game::try_new()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, cursor::Hide, terminal::EnterAlternateScreen,)?;
    terminal::enable_raw_mode()?;

    loop {
        game.update()?;
        if crossterm::event::poll(std::time::Duration::from_millis(50))?
            && !game.handle_input(crossterm::event::read()?)
        {
            break;
        }
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen,)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
