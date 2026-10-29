use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};
use wikit_tui::{QueryInput, Source, plain_text, terminal_text};

/// A UTF-8 dictionary for terminals, SSH, Vim and Emacs.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Local .wikit or .mdx file; repeat to switch dictionaries with Tab.
    #[arg(short, long)]
    dictionary: Vec<PathBuf>,
    /// Wikit server base URL; repeat for multiple servers.
    #[arg(long)]
    server: Vec<String>,
    /// A word to print once. Omit to open the interactive TUI.
    word: Option<String>,
    /// Print machine-readable word/HTML pairs instead of terminal text.
    #[arg(long, requires = "word")]
    json: bool,
    /// Initial dictionary, numbered from 1.
    #[arg(long, default_value_t = 1)]
    select: usize,
    /// Output width for one-shot text.
    #[arg(long, default_value_t = 80, value_parser = clap::value_parser!(u16).range(20..=240))]
    width: u16,
}
fn main() -> Result<()> {
    let args = Args::parse();
    let mut sources = Vec::new();
    for path in &args.dictionary {
        sources.push(Source::local(path)?);
    }
    for url in &args.server {
        sources.extend(Source::remote(url)?);
    }
    anyhow::ensure!(
        !sources.is_empty(),
        "Add --dictionary PATH or --server URL. Use --help for examples."
    );
    anyhow::ensure!(
        args.select > 0 && args.select <= sources.len(),
        "--select must identify an available dictionary (1–{})",
        sources.len()
    );
    if let Some(word) = args.word {
        let results = sources[args.select - 1].lookup(&word)?;
        if args.json {
            println!("{}", serde_json::to_string(&results)?);
        } else if results.is_empty() {
            println!("No matching entry for {}.", terminal_text(&word));
        } else {
            for (word, html) in results {
                println!(
                    "{}\n\n{}",
                    terminal_text(&word),
                    terminal_text(&plain_text(&html, args.width.into())?)
                );
            }
        }
        return Ok(());
    }
    anyhow::ensure!(
        io::stdin().is_terminal() && io::stdout().is_terminal(),
        "Interactive mode needs a terminal. Supply a word for editor or pipe output."
    );
    interactive(sources, args.select - 1)
}
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}
fn interactive(sources: Vec<Source>, mut dictionary: usize) -> Result<()> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let prior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        prior(info);
    }));
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let (send, receive) = mpsc::channel::<(u64, usize, String)>();
    let (reply, results) = mpsc::channel();
    let worker_sources = sources.clone();
    std::thread::spawn(move || {
        while let Ok((version, index, word)) = receive.recv() {
            let result = worker_sources[index]
                .lookup(&word)
                .map_err(|e| e.to_string());
            if reply.send((version, result)).is_err() {
                break;
            }
        }
    });
    let mut input = QueryInput::default();
    let mut editing = true;
    let mut help = false;
    let mut entries: Vec<(String, String)> = Vec::new();
    let mut selection = ListState::default();
    let mut scroll = 0u16;
    let mut version = 0u64;
    let mut busy = false;
    let mut message = String::from("Type a word and press Enter.");
    loop {
        while let Ok((response, result)) = results.try_recv() {
            if response == version {
                busy = false;
                scroll = 0;
                match result {
                    Ok(next) => {
                        entries = next;
                        selection.select(if entries.is_empty() { None } else { Some(0) });
                        message = if entries.is_empty() {
                            "No matching entry. Try another spelling or dictionary.".into()
                        } else {
                            format!("{} entries", entries.len())
                        };
                    }
                    Err(error) => {
                        message = error;
                        entries.clear();
                        selection.select(None);
                    }
                }
            }
        }
        terminal.draw(|frame| {
            let rows=Layout::vertical([Constraint::Length(3),Constraint::Min(1),Constraint::Length(2)]).split(frame.area());
            let title=format!(" Wikit · {} · Tab to switch ",terminal_text(&sources[dictionary].name));
            frame.render_widget(Paragraph::new(terminal_text(&input.value)).block(Block::bordered().title(title)).style(Style::default().fg(if editing{Color::Green}else{Color::White})),rows[0]);
            if help {
                frame.render_widget(Paragraph::new("Search: / or i\nSubmit: Enter\nDictionary: Tab / Shift+Tab\nSelect entry: Up / Down\nScroll definition: PageUp / PageDown\nHelp: F1\nQuit: q (reading), Esc (reading), Ctrl+C\n\nOne-shot: wikit-tui -d dictionary.wikit hello\nEditor output: add --json for word/HTML pairs").block(Block::bordered().title(" Help ")).wrap(Wrap{trim:false}),rows[1]);
            } else if entries.is_empty() {
                frame.render_widget(Paragraph::new(terminal_text(if busy{"Looking up…"}else{&message})).block(Block::bordered().title(" Dictionary ")).wrap(Wrap{trim:false}),rows[1]);
            } else {
                let columns=if frame.area().width>=65 { Layout::horizontal([Constraint::Percentage(28),Constraint::Percentage(72)]).split(rows[1]) } else { Layout::horizontal([Constraint::Length(0),Constraint::Min(1)]).split(rows[1]) };
                let items=entries.iter().map(|(word,_)|ListItem::new(terminal_text(word))).collect::<Vec<_>>();
                frame.render_stateful_widget(List::new(items).block(Block::bordered().title(" Words ")).highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)).highlight_symbol("› "),columns[0],&mut selection);
                if let Some((word,html))=selection.selected().and_then(|index|entries.get(index)) {
                    let text=plain_text(html,columns[1].width.saturating_sub(4).max(10).into()).unwrap_or_else(|_|"Cannot render this definition.".into());
                    frame.render_widget(Paragraph::new(terminal_text(&text)).block(Block::bordered().title(format!(" {} ",terminal_text(word)))).scroll((scroll,0)).wrap(Wrap{trim:false}),columns[1]);
                }
            }
            frame.render_widget(Paragraph::new(format!("{}\n/ search · Tab dictionary · ↑↓ entry · PgUp/PgDn read · F1 help · Ctrl+C quit",if busy{"Looking up…"}else{&message})).style(Style::default().fg(Color::DarkGray)),rows[2]);
        })?;
        if !event::poll(Duration::from_millis(80))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('q'))
            {
                break;
            }
            match key.code {
                KeyCode::F(1) => help = !help,
                KeyCode::Tab | KeyCode::BackTab => {
                    dictionary = if key.code == KeyCode::BackTab {
                        (dictionary + sources.len() - 1) % sources.len()
                    } else {
                        (dictionary + 1) % sources.len()
                    };
                    version += 1;
                    entries.clear();
                    busy = false;
                    editing = true;
                    message = "Dictionary changed. Press Enter to look up the word.".into();
                }
                KeyCode::Enter if !input.value.trim().is_empty() => {
                    version += 1;
                    busy = true;
                    editing = false;
                    help = false;
                    send.send((version, dictionary, input.value.clone()))
                        .context("Lookup worker stopped")?;
                }
                KeyCode::Esc if help => help = false,
                KeyCode::Esc if editing => editing = false,
                KeyCode::Esc => break,
                KeyCode::Backspace if editing => input.backspace(),
                KeyCode::Char(c) if editing && !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.push(c)
                }
                KeyCode::Char('/') | KeyCode::Char('i') => editing = true,
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if let Some(index) = selection.selected() {
                        selection.select(Some(index.saturating_sub(1)));
                        scroll = 0;
                    }
                }
                KeyCode::Down => {
                    if let Some(index) = selection.selected() {
                        selection.select(Some((index + 1).min(entries.len() - 1)));
                        scroll = 0;
                    }
                }
                KeyCode::PageDown => scroll = scroll.saturating_add(10),
                KeyCode::PageUp => scroll = scroll.saturating_sub(10),
                _ => {}
            }
        }
    }
    Ok(())
}
