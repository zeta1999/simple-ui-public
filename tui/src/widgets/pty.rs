use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use std::sync::{Arc, Mutex};
use vt100::Parser;

/// A rudimentary PTY wrapper that embeds a vt100 terminal screen into Ratatui.
pub struct EmbeddedTerminal {
    parser: Arc<Mutex<Parser>>,
}

impl Default for EmbeddedTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddedTerminal {
    pub fn new() -> Self {
        let parser = Arc::new(Mutex::new(Parser::new(24, 80, 0)));
        Self { parser }
    }

    pub fn write_output(&self, data: &[u8]) {
        if let Ok(mut parser) = self.parser.lock() {
            parser.process(data);
        }
    }

    pub fn resize(&self, rows: u16, cols: u16) {
        if let Ok(mut parser) = self.parser.lock() {
            parser.set_size(rows, cols);
        }
    }
}

impl Widget for &EmbeddedTerminal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Ok(parser) = self.parser.lock() {
            let screen = parser.screen();
            let mut lines = Vec::new();

            let size = screen.size();
            for row in 0..size.0 {
                let mut spans = Vec::new();
                for col in 0..size.1 {
                    if let Some(cell) = screen.cell(row, col) {
                        let mut style = Style::default();
                        // Map vt100 colors to Ratatui colors (simplified)
                        let _fg = cell.fgcolor();
                        // Simple presence check (vt100::Color is an enum, usually not Default if set)
                        style = style.fg(Color::Gray);

                        // cell.contents() gives a string. If empty, add space
                        let content = cell.contents();
                        let text = if content.is_empty() {
                            " "
                        } else {
                            content.as_str()
                        };
                        spans.push(Span::styled(text.to_string(), style));
                    }
                }
                lines.push(Line::from(spans));
            }

            Paragraph::new(lines).render(area, buf);
        }
    }
}
