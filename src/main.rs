#![allow(dead_code)]

mod app;
mod autocomplete;
mod build;
mod config;
mod diagnostics;
mod docs;
mod input;
mod masm_lang;
mod project;
mod syntax;
mod theme;
mod ui;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "masmide")]
#[command(author, version, about = "TUI IDE for MASM development on Linux", long_about = None)]
struct Args {
    /// File or directory to open
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Create a new project with the given name
    #[arg(short, long)]
    new: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(project_name) = args.new {
        project::create_new_project(&project_name)?;
        println!("Created new project: {}", project_name);
        println!("Run: cd {} && masmide", project_name);
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.path)?;
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Update editor visible height for proper scroll handling after resize
        let visible_height = terminal.size()?.height.saturating_sub(5) as usize;
        app.update_editor_visible_height(visible_height);

        // Check autosave
        app.check_autosave();

        if let Some(action) = input::handle_event(app)? {
            match action {
                input::Action::Quit => break,
                input::Action::Build => app.build()?,
                input::Action::Run => app.run()?,
                input::Action::BuildAndRun => {
                    app.build()?;
                    if app.build_succeeded() {
                        app.run()?;
                    }
                }
                input::Action::Save => {
                    app.save_current_file()?;
                    app.last_save_time = std::time::Instant::now();
                }
                input::Action::None => {}
            }
        }
    }
    Ok(())
}
