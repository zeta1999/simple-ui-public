pub mod widgets;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use markdown_engine::{parse_markdown, Block as MdBlock, Document};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};
use std::{error::Error, io};

use widgets::candlestick::{Candle, CandlestickChart};
use widgets::editor::CodeEditor;
use widgets::pty::EmbeddedTerminal;

struct App<'a> {
    pub titles: Vec<String>,
    pub index: usize,
    pub document: Document,
    pub editor: CodeEditor<'a>,
    pub pty: EmbeddedTerminal,
    pub candles: Vec<Candle>,
}

impl<'a> App<'a> {
    fn new(document: Document) -> App<'a> {
        let editor = CodeEditor::new("fn main() {\n    println!(\"Hello World\");\n}\n");
        let pty = EmbeddedTerminal::new();
        // Spawning a sample shell for the embedded pty
        // pty.spawn("bash"); // (Placeholder for actual spawn logic if implemented)

        let candles = vec![
            Candle {
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
            },
            Candle {
                open: 105.0,
                high: 120.0,
                low: 100.0,
                close: 115.0,
            },
            Candle {
                open: 115.0,
                high: 115.0,
                low: 95.0,
                close: 98.0,
            },
            Candle {
                open: 98.0,
                high: 130.0,
                low: 90.0,
                close: 125.0,
            },
        ];

        App {
            titles: vec!["Dashboard".into(), "Financials".into(), "Terminal".into()],
            index: 0,
            document,
            editor,
            pty,
            candles,
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub fn run_tui(md_content: &str) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse the markdown dynamically into the generic AST
    let ast = parse_markdown(md_content);

    // Create app state using the generated AST
    let app = App::new(ast);

    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<'a, B: Backend>(terminal: &mut Terminal<B>, mut app: App<'a>) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Split into Header and Body
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            // Render Tabs Header
            let titles: Vec<ratatui::text::Line> = app
                .titles
                .iter()
                .cloned()
                .map(ratatui::text::Line::from)
                .collect();
            let tabs = Tabs::new(titles)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Extended Markdown GUI"),
                )
                .select(app.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(Style::default().fg(Color::Yellow));
            f.render_widget(tabs, chunks[0]);

            // Render Body depending on Tab
            match app.index {
                0 => render_dynamic_ast(f, chunks[1], &app.document, &app.editor),
                1 => render_financials(f, chunks[1], &app.candles),
                2 => render_terminal(f, chunks[1], &app.pty),
                _ => unreachable!(),
            }
        })?;

        if let CEvent::Key(key) = event::read()? {
            // Forward events to editor if on dashboard tab
            if app.index == 0 {
                app.editor.handle_event(CEvent::Key(key));
            }

            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Right => app.next(),
                KeyCode::Left => app.previous(),
                _ => {}
            }
        }
    }
}

fn render_dynamic_ast(
    f: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    doc: &Document,
    editor: &CodeEditor,
) {
    // A simplified layout engine mapping generic AST blocks to TUI widgets
    let splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let mut markdown_blocks = Vec::new();
    let mut interactive_blocks = Vec::new();

    for block in &doc.blocks {
        match block {
            MdBlock::Markdown(text) => markdown_blocks.push(text.clone()),
            MdBlock::Question(q) => {
                let text = format!(
                    "❓ Question: {}\nOptions: {:?}",
                    q["question"], q["options"]
                );
                interactive_blocks.push(text);
            }
            MdBlock::Plot(p) => {
                let text = format!("📈 Plot (Type: {}): {}", p["type"], p["title"]);
                interactive_blocks.push(text);
            }
            MdBlock::Spreadsheet(s) => {
                let text = format!(
                    "📊 Spreadsheet ({} rows)",
                    s["data"].as_array().map(|a| a.len()).unwrap_or(0)
                );
                interactive_blocks.push(text);
            }
        }
    }

    let p1 = Paragraph::new(markdown_blocks.join("\n")).block(
        Block::default()
            .title("Parsed Markdown Text")
            .borders(Borders::ALL),
    );
    f.render_widget(p1, splits[0]);

    // Replaced the interactive blocks generic paragraph with the actual Code Editor for demo
    f.render_widget(editor.widget(), splits[1]);
}

fn render_financials(f: &mut ratatui::Frame, area: ratatui::layout::Rect, candles: &[Candle]) {
    let chart = CandlestickChart::new(candles).block(
        Block::default()
            .title("Candlestick Chart")
            .borders(Borders::ALL),
    );

    f.render_widget(chart, area);
}

fn render_terminal(f: &mut ratatui::Frame, area: ratatui::layout::Rect, pty: &EmbeddedTerminal) {
    // Update terminal size based on current layout area
    pty.resize(area.height, area.width);

    f.render_widget(pty, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_app_state_navigation() {
        let doc = Document { blocks: vec![] };
        let mut app = App::new(doc);
        assert_eq!(app.index, 0); // Starts at Dashboard

        app.next();
        assert_eq!(app.index, 1); // Moves to Financials

        app.previous();
        assert_eq!(app.index, 0); // Back to Dashboard
    }

    #[test]
    fn test_render_dynamic_ast() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let doc = Document {
            blocks: vec![
                MdBlock::Markdown("Hello Ast".to_string()),
                MdBlock::Plot(serde_json::json!({"type": "bar", "title": "Test Plot"})),
            ],
        };

        let editor = CodeEditor::new("");

        terminal
            .draw(|f| {
                render_dynamic_ast(f, f.size(), &doc, &editor);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        let content = format!("{:?}", buffer);
        assert!(content.contains("Hello Ast"));
    }

    #[test]
    fn test_render_financials() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let candles = vec![Candle {
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
        }];

        terminal
            .draw(|f| {
                render_financials(f, f.size(), &candles);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = format!("{:?}", buffer);
        assert!(content.contains("Candlestick Chart"));
    }
}
