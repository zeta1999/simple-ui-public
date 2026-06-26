use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Read the demo.md file
    let md_content = fs::read_to_string("examples/full_tui/demo.md")
        .unwrap_or_else(|_| "Failed to load demo.md".to_string());

    // Run the TUI application with the loaded markdown content
    tui::run_tui(&md_content)?;

    Ok(())
}
