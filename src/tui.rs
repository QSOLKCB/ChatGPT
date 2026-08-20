use std::io;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;

#[derive(Default)]
struct AppState {
    revoked: bool,
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = event_loop(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut state = AppState::default();
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ])
                .split(frame.area());

            let title = Paragraph::new("QSOL ChatGPT | Rust authority console")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(title, chunks[0]);

            let mode = if state.revoked { "REVOKED" } else { "READY" };
            let body = format!(
                "CAPABILITY != AUTHORITY\n\nStatus: {mode}\n\nDefault authority\n  screen.capture    observe-only\n  filesystem.read   observe-only\n  shell.exec        approval required\n  filesystem.write  approval required\n  input.*           approval required\n  app.launch        approval required\n  network           denied / not brokered\n  credentials       opaque handles only\n\nSecrets are never stored in action JSON, receipts, logs, or TUI state."
            );
            let panel = Paragraph::new(body)
                .wrap(Wrap { trim: false })
                .block(Block::default().title("Authority").borders(Borders::ALL));
            frame.render_widget(panel, chunks[1]);

            let footer = Paragraph::new("q: quit    x: emergency revoke-all")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(footer, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('x') => state.revoked = true,
                _ => {}
            }
        }
    }
}
