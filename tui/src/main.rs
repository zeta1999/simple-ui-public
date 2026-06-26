use std::env;
use std::error::Error;
use std::fs;
use tui::run_tui;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    // Look for --file <path> or use fallback
    let mut file_path = "../graphical/public/demo.md";
    for i in 0..args.len() {
        if args[i] == "--file" && i + 1 < args.len() {
            file_path = &args[i + 1];
            break;
        }
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", file_path, e);
            eprintln!("Usage: tui --file <path-to-markdown.md>");
            std::process::exit(1);
        }
    };

    run_tui(&content)?;

    Ok(())
}
