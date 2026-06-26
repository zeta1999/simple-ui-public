use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::widgets::Widget;
use tui_textarea::TextArea;

/// A simple multi-line text editor widget.
///
/// ⚠️ **Plaintext only.** The buffer is held in ordinary `String`s with no
/// zeroization or locked memory, so editing leaves copies on the heap. Do **not**
/// use this widget to edit secret material that must be wiped from memory; it is
/// intended for non-sensitive notes and text.
pub struct CodeEditor<'a> {
    textarea: TextArea<'a>,
}

impl<'a> CodeEditor<'a> {
    /// Builds an editor seeded with `content`. The content round-trips exactly
    /// through [`text`](Self::text): lines are split on `\n` without fabricating
    /// a trailing newline or rewriting line endings.
    pub fn new(content: &str) -> Self {
        let lines: Vec<String> = content.split('\n').map(String::from).collect();
        Self {
            textarea: TextArea::new(lines),
        }
    }

    /// Forwards a terminal event to the editor. Returns `false` if the event was
    /// an `Esc` (caller should leave the editor) and `true` if the editor
    /// consumed it.
    pub fn handle_event(&mut self, event: Event) -> bool {
        if let Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) = event
        {
            return false; // Leave editor
        }
        self.textarea.input(event)
    }

    /// The current buffer as individual lines (no line terminators).
    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    /// The current buffer as a single string, lines joined by `\n`.
    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn widget(&'a self) -> impl Widget + 'a {
        self.textarea.widget()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trips_without_fabricated_newline() {
        // No trailing newline in -> none out (the old impl appended one).
        let e = CodeEditor::new("alpha\nbeta");
        assert_eq!(e.lines(), &["alpha".to_string(), "beta".to_string()]);
        assert_eq!(e.text(), "alpha\nbeta");
    }

    #[test]
    fn text_preserves_trailing_newline_and_crlf() {
        let e = CodeEditor::new("a\r\nb\n");
        // CRLF is preserved (the \r stays in the line); the trailing \n yields a
        // final empty line, so join restores the original exactly.
        assert_eq!(e.text(), "a\r\nb\n");
    }

    #[test]
    fn empty_input_is_a_single_empty_line() {
        let e = CodeEditor::new("");
        assert_eq!(e.lines(), &["".to_string()]);
        assert_eq!(e.text(), "");
    }
}
