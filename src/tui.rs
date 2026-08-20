use std::io;

use crossterm::cursor::Show;
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

struct TerminalModeGuard;

impl TerminalModeGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let guard = Self;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        Ok(guard)
    }
}

impl Drop for TerminalModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, Show);
    }
}

pub fn run() -> io::Result<()> {
    let _terminal_mode = TerminalModeGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    event_loop(&mut terminal)
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
                "CAPABILITY != AUTHORITY\n\nStatus: {mode}\n\nDesktop observation\n  screen.capture    one-shot / portal mediated\n  screen.observe    approval required / bounded ScreenCast\n  raw frames        local-only; no OpenAI forwarding yet\n\nDefault authority\n  filesystem.read   observe-only\n  shell.exec        approval required\n  filesystem.write  approval required\n  input.*           approval required\n  app.launch        approval required\n\nOBS structured broker\n  obs.* reads       loopback-only read\n  scene/record      approval required\n  stream.stop       approval required\n  stream.start      DENIED pending stronger approval\n\nGeneral network     denied / not brokered\nCredentials         opaque handles only\n\nSecrets and raw screen frames are never stored in receipts, logs, or TUI state."
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
