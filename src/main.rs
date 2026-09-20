use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::panic;
use std::time::Duration;

mod config;
mod converter;
mod domain;
mod downloader;
mod error;
pub mod i18n;
mod scraper;
mod ui;

use config::Config;
use ui::{App, EventHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Handle CLI arguments (e.g. neo-mangal sources install)
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "sources" {
        let sub = args.get(2).map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "install" => {
                println!("󰋩 Installing official manga scrapers into ~/.config/neo-mangal/sources/...");
                match scraper::SourceManager::install_sources_from_repo(scraper::manager::DEFAULT_REPO) {
                    Ok(sources) => {
                        println!("✓ Successfully installed {} scrapers:", sources.len());
                        for s in sources {
                            println!("  • {}", s);
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Error installing scrapers: {}", e);
                    }
                }
                return Ok(());
            }
            "list" => {
                let sources = scraper::SourceManager::list_sources();
                if sources.is_empty() {
                    println!("No scrapers found in ~/.config/neo-mangal/sources/.");
                    println!("Run 'neo-mangal sources install' to download official scrapers.");
                } else {
                    println!("󰋩 Installed manga scrapers ({}) in ~/.config/neo-mangal/sources/:", sources.len());
                    for s in sources {
                        println!("  • {}", s);
                    }
                }
                return Ok(());
            }
            "reset" => {
                let dir = scraper::SourceManager::sources_dir();
                println!("󰋩 Resetting manga scrapers in {:?}...", dir);
                match scraper::SourceManager::reset_sources() {
                    Ok(sources) => {
                        println!("✓ Successfully reset scrapers. Active sources ({}):", sources.len());
                        for s in sources {
                            println!("  • {}", s);
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Error resetting scrapers: {}", e);
                    }
                }
                return Ok(());
            }
            "path" | "dir" => {
                let dir = scraper::SourceManager::sources_dir();
                println!("{}", dir.display());
                return Ok(());
            }
            _ => {
                println!("Usage: neo-mangal sources [install|list|reset|path]");
                return Ok(());
            }
        }
    }

    // 1. Setup panic hook to always restore terminal on unexpected panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            DisableBracketedPaste
        );
        original_hook(panic_info);
    }));

    // 2. Initialize terminal with bracketed paste mode (crucial for Drag & Drop)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Load configuration (with auto-migration from mangal.toml if available)
    let config = Config::load().unwrap_or_default();

    // 4. Initialize event stream and application state
    let mut event_handler = EventHandler::new(Duration::from_millis(50));
    let mut app = App::new(config, event_handler.sender());

    // 5. Main TUI render & event loop
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if let Some(event) = event_handler.next().await {
            app.handle_event(event);
        }

        if app.should_quit {
            break;
        }
    }

    // 6. Clean terminal restore and immediate exit
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    );
    let _ = terminal.show_cursor();

    std::process::exit(0);
}
