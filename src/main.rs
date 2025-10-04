use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, Terminal};
use typewell_tui::app;

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // run app
    let res = app::run(&mut terminal);

    // restore terminal
    disable_raw_mode().ok();
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture).ok();
    if let Err(e) = res {
        eprintln!("error: {e:?}");
    }
    Ok(())
}
