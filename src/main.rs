use std::{io, time::{Duration, Instant}};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind},
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
    Hidden, // 未被翻开
    Revealed, // 已翻开
    Flagged, // 已标记
}

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    is_mine: bool, // 是否是雷
    state: CellState, // 状态
    neighbor_mines: u8, // 周围雷的数量
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
            cells: vec![vec![Cell {
                is_mine: false,
                state: CellState::Hidden,
                neighbor_mines: 0,
            }; width]; height],
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

    fn calculate_neighbors(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.cells[y][x].is_mine {
                    let mut count = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dy == 0 && dx == 0 {
                                continue;
                            }
                            let ny = y as i32 + dy;
                            let nx = x as i32 + dx;
                            if ny >= 0
                                && ny < self.height as i32
                                && nx >= 0
                                && nx < self.width as i32
                            {
                                if self.cells[ny as usize][nx as usize].is_mine {
                                    count += 1;
                                }
                            }
                        }
                    }
                    self.cells[y][x].neighbor_mines = count;
                }
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
            // 记录游戏结束时间
            if self.end_time.is_none() {
                self.end_time = self.start_time;
            }
            return;
        }

        self.cells[y][x].state = CellState::Revealed;

        if self.cells[y][x].neighbor_mines == 0 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let ny = y as i32 + dy;
                    let nx = x as i32 + dx;
                    if ny >= 0
                        && ny < self.height as i32
                        && nx >= 0
                        && nx < self.width as i32
                    {
                        self.reveal(nx as usize, ny as usize);
                    }
                }
            }
        }

        self.check_victory();
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.cells[y][x].state == CellState::Hidden && self.flags < self.mines {
            self.cells[y][x].state = CellState::Flagged;
            self.flags += 1;
        } else if self.cells[y][x].state == CellState::Flagged {
            self.cells[y][x].state = CellState::Hidden;
            self.flags -= 1;
        }
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
        if self.victory && self.end_time.is_none() {
            self.end_time = Some(Instant::now());
       }
    }

    fn get_elapsed_time(&self) -> u64 {
        match (self.start_time, self.end_time) {
            //未开始
            (None, _) => 0,
            //进行中
            (Some(start),None) => start.elapsed().as_secs(),
            //已结束
            (Some(start), Some(end)) => end.duration_since(start).as_secs(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let difficulties = [(8, 8, 10), (16, 16, 40), (24, 20, 99)];

    println!("选择难度:");
    println!("1. 初级 (8x8, 10 雷)");
    println!("2. 中级 (16x16, 40 雷)");
    println!("3. 高级 (24x20, 99 雷)");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let mut difficulty = input.trim().parse::<usize>().unwrap_or(1) - 1;
    if difficulty > 2 {
        difficulty = 2;
    }

    let (width, height, mines) = difficulties[difficulty];
    let mut game = Game::new(width, height, mines);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title(Span::styled(" 扫雷 ", Style::default().fg(Color::Yellow)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(block.inner(size));

            f.render_widget(block, size);

            // 绘制游戏区域
            let cell_width = 3;
            let cell_height = 1;
            let start_x = layout[1].x + (layout[1].width - (cell_width * width as u16)) as u16 / 2;
            let start_y = layout[1].y + (layout[1].height - (cell_height * height as u16)) as u16 / 2;

            for y in 0..game.height {
                for x in 0..game.width {
                    let cell = &game.cells[y][x];
                    let symbol = match cell.state {
                        CellState::Hidden => "■",
                        CellState::Flagged => "⚑",
                        CellState::Revealed => {
                            if cell.is_mine {
                                "💣"
                            } else if cell.neighbor_mines > 0 {
                                match cell.neighbor_mines {
                                    1 => "1",
                                    2 => "2",
                                    3 => "3",
                                    4 => "4",
                                    5 => "5",
                                    6 => "6",
                                    7 => "7",
                                    8 => "8",
                                    _ => " ",
                                }
                            } else {
                                " "
                            }
                        }
                    };

                    let style = if x == game.cursor_x && y == game.cursor_y && !game.game_over {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

                    let _span = Span::styled(
                        format!("{} ", symbol),
                        style.fg(match cell.state {
                            CellState::Revealed if cell.is_mine => Color::Red,
                            CellState::Flagged => Color::Red,
                            CellState::Revealed => match cell.neighbor_mines {
                                1 => Color::Blue,
                                2 => Color::Green,
                                3 => Color::Red,
                                _ => Color::White,
                            },
                            _ => Color::White,
                        }),
                    );

                    let color = match symbol {
                        "2" => Color::Green,
                        "3" => Color::Red,
                        _ => Color::White,
                    };
                    
                    let paragraph = Paragraph::new(Span::styled(
                        format!("{} ", symbol),
                        style.fg(color) // 将 color 变量传递给 fg 方法
                    )); 
                    
                    f.render_widget(
                        paragraph,  // 现在渲染的是Paragraph而不是Span
                        Rect {
                            x: start_x + x as u16 * cell_width,
                            y: start_y + y as u16 * cell_height,
                            width: cell_width,
                            height: cell_height,
                        },
                    );
                }
            }

            // 绘制状态信息
            let status_text = format!(
                "时间: {}秒 | 剩余旗帜: {} | 难度: {}",
                game.get_elapsed_time(),
                game.mines - game.flags,
                match difficulty {
                    0 => "初级",
                    1 => "中级",
                    _ => "高级",
                }
            );
            let status_paragraph = Paragraph::new(Spans::from(vec![Span::styled(
                status_text,
                Style::default().fg(Color::Cyan),
            )]));
            f.render_widget(status_paragraph, layout[0]);

            // 绘制游戏结束信息
            if game.game_over || game.victory {
                let message = if game.victory {
                    Spans::from(vec![Span::styled(
                        " 你赢了！按R重玩，Q退出 ",
                        Style::default().fg(Color::Green),
                    )])
                } else {
                    Spans::from(vec![Span::styled(
                        " 你输了！按R重玩，Q退出 ",
                        Style::default().fg(Color::Red),
                    )])
                };
                let message_paragraph = Paragraph::new(message).alignment(Alignment::Center);
                f.render_widget(
                    message_paragraph,
                    Rect {
                        x: layout[1].x + (layout[1].width - 20) / 2,
                        y: layout[1].y + (layout[1].height - 1) / 2,
                        width: 20,
                        height: 1,
                    },
                );
            }
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,    
                        ..
                    } => break,

                    KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if game.game_over || game.victory => {
                        game = Game::new(width, height, mines);
                        game.start_time = Some(Instant::now());
                    }

                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if !game.game_over && !game.victory => {
                        if game.cursor_y > 0 {
                            game.cursor_y -= 1;
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if !game.game_over && !game.victory => {
                        if game.cursor_y < game.height - 1 {
                            game.cursor_y += 1;
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if !game.game_over && !game.victory => {
                        if game.cursor_x > 0 {
                            game.cursor_x -= 1;
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if !game.game_over && !game.victory => {
                        if game.cursor_x < game.width - 1 {
                            game.cursor_x += 1;
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Char(' '),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if !game.game_over && !game.victory => {
                        if game.start_time.is_none() {
                            game.start_time = Some(Instant::now());
                        }
                        game.reveal(game.cursor_x, game.cursor_y);
                    }

                    KeyEvent {
                        code: KeyCode::Char('f'),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        ..
                    } if !game.game_over && !game.victory => {
                        game.toggle_flag(game.cursor_x, game.cursor_y);
                    }

                    _ => {}
                }
            }
        }

        if game.game_over || game.victory {
            if let Some(start_time) = game.start_time {
                if start_time.elapsed().as_secs() > 999 {
                    game.start_time = Some(Instant::now() - Duration::from_secs(999));
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        Show
    )?;
    Ok(())
}