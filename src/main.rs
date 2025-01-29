use std::{io, time::{Duration, Instant}};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    is_mine: bool,
    state: CellState,
    neighbor_mines: u8,
}

struct Game {
    width: usize,
    height: usize,
    mines: usize,
    cells: Vec<Vec<Cell>>,
    cursor_x: usize,
    cursor_y: usize,
    game_over: bool,
    victory: bool,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
    flags: usize,
}

impl Game {
    fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut game = Game {
            width,
            height,
            mines,
            cells: vec![vec![Cell { is_mine: false, state: CellState::Hidden, neighbor_mines: 0 }; width]; height],
            cursor_x: 0,
            cursor_y: 0,
            game_over: false,
            victory: false,
            start_time: None,
            end_time: None,
            flags: 0,
        };
        game.place_mines();
        game.calculate_neighbors();
        game
    }

    fn get_elapsed_time(&self) -> u64 {
        match (self.start_time, self.end_time) {
            (None, _) => 0,
            (Some(start), None) => start.elapsed().as_secs(),
            (Some(start), Some(end)) => end.duration_since(start).as_secs(),
        }
    }

    fn place_mines(&mut self) {
        let mut rng = rand::rng();
        let mut placed = 0;
        while placed < self.mines {
            let x = rng.random_range(0..self.width);
            let y = rng.random_range(0..self.height);
            if !self.cells[y][x].is_mine {
                self.cells[y][x].is_mine = true;
                placed += 1;
            }
        }
    }

    fn reveal(&mut self, x: usize, y: usize) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
        if self.cells[y][x].state != CellState::Hidden {
            return;
        }

        if self.cells[y][x].is_mine {
            self.game_over = true;
            self.end_time = Some(Instant::now());
            return;
        }

        self.cells[y][x].state = CellState::Revealed;
        self.check_victory();
    }

    fn check_victory(&mut self) {
        let mut revealed_count = 0;
        for row in &self.cells {
            for cell in row {
                if cell.state == CellState::Revealed && !cell.is_mine {
                    revealed_count += 1;
                }
            }
        }
        let total_safe = self.width * self.height - self.mines;
        self.victory = revealed_count == total_safe;
        if self.victory {
            self.end_time = Some(Instant::now());
        }
    }
}

struct Leaderboard {
    records: [Option<u64>; 3],
}

impl Leaderboard {
    fn new() -> Self {
        Self { records: [None, None, None] }
    }

    fn update(&mut self, difficulty: usize, time: u64) {
        if let Some(best_time) = self.records[difficulty] {
            if time < best_time {
                self.records[difficulty] = Some(time);
            }
        } else {
            self.records[difficulty] = Some(time);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let difficulties = [(8, 8, 10), (16, 16, 40), (24, 20, 99)];
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut leaderboard = Leaderboard::new();
    let mut difficulty = 0;
    let mut game = Game::new(difficulties[difficulty].0, difficulties[difficulty].1, difficulties[difficulty].2);

    loop {
        terminal.draw(|f| {
            let elapsed_time_text = format!("时间: {} 秒", game.get_elapsed_time());
            let elapsed_paragraph = Paragraph::new(Span::styled(
                elapsed_time_text, Style::default().fg(Color::Cyan)
            ));
            f.render_widget(elapsed_paragraph, f.size());
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('r') => {
                    game = Game::new(difficulties[difficulty].0, difficulties[difficulty].1, difficulties[difficulty].2);
                }
                KeyCode::Char(' ') => {
                    game.reveal(game.cursor_x, game.cursor_y);
                    if game.victory {
                        leaderboard.update(difficulty, game.get_elapsed_time());
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
    Ok(())
}
