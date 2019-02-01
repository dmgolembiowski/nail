#[allow(dead_code)]
mod util;
mod file;
mod modes;

use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Tabs, Widget, Paragraph};
use tui::Terminal;
use crate::file::File;
use crate::modes::Mode;

use crate::util::event::{Event, Events};
use crate::util::TabsState;

struct App<'a> {
    tabs: TabsState<'a>,
    files: Vec<File<'a>>,
    mode: Mode
}

fn default_mode(events: &Events, app: &mut App) -> Result<(), failure::Error>  {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Char('q') =>
                app.mode = Mode::Quit,
            Key::Char(':') =>
               app.mode = Mode::Command,
            Key::Char('i') =>
                app.mode = Mode::Insert,
            Key::Right => app.tabs.next(),
            Key::Left => app.tabs.previous(),
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn command_mode(events: &Events, app: &mut App) -> Result<(), failure::Error> {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Esc => app.mode = Mode::Default,
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn insert_mode(events: &Events, app: &mut App) -> Result<(), failure::Error> {
    match events.next()? {
        Event::Input(input) => match input {
            Key::Esc => app.mode = Mode::Default,
            _ => {}
        },
        _ => {}
    }
    Ok(())
} 

fn main() -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    
    let filenames = vec!["File 0", "File 1", "File 2","File 3"];
    let filepaths = vec!["C:/path/to/file/0.txt",
                        "C:/path/to/file/1.txt",
                        "C:/path/to/file/2.txt",
                        "C:/path/to/file/3.txt"];
    let files : Vec<File> = 
        vec![b"Test 1 blha blah blah \x00 test adsasdasdas\xFF\x12dsadsad\n".to_vec(),
             b"Test 2\n".to_vec(),
             (0u8..=0xFFu8).collect(),
             b"Test 4\n".to_vec()]
            .into_iter()
            .enumerate()
            .map(|(x, y)| File {
                name: filenames[x],
                path: filepaths[x],
                data: y.to_vec(),
                pos: 0
            })
            .collect();

    // App
    let mut app = App {
        files,
        tabs: TabsState::new(filenames),
        mode: Mode::Default
    };

    // Main loop
    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            let line_count = size.height as usize;
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                //.margin(5)
                .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(1)].as_ref())
                .split(size);

            Block::default()
                .style(Style::default().bg(
                        match app.mode {
                            Mode::Command => Color::Red,
                            _ => Color::Cyan
                        }))
                .render(&mut f, size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .titles(&app.tabs.titles)
                .select(app.tabs.index)
                .style(Style::default().fg(Color::LightBlue))
                .highlight_style(Style::default().fg(Color::Red))
                .render(&mut f, chunks[0]);
            match app.tabs.index {
                0...4 => {
                    let view = app.files[app.tabs.index].hex_view(line_count);
                    Paragraph::new(view.iter())
                    .block(
                        Block::default()
                        .title(app.files[app.tabs.index].path)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(
                            match app.mode {
                                Mode::Insert => Color::Yellow,
                                _ => Color::White
                            })))
                    .render(&mut f, chunks[1]);
                }
                _ => {}
            }
        })?;
        
        match app.mode {
            Mode::Default => default_mode(&events, &mut app)?,
            Mode::Command => command_mode(&events, &mut app)?,
            Mode::Insert => insert_mode(&events, &mut app)?,
            Mode::Quit => break,
            _ => {}
        };
    }
    Ok(())
}
