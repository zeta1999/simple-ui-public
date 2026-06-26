pub mod evaluator;
#[cfg(not(target_arch = "wasm32"))]
pub mod ipc;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Block {
    Markdown(String),
    Plot(serde_json::Value),
    Question(serde_json::Value),
    Spreadsheet(serde_json::Value),
}

pub fn parse_markdown(input: &str) -> Document {
    let parser = Parser::new(input);
    let mut blocks = Vec::new();
    let mut current_markdown = String::new();

    let mut in_custom_block = false;
    let mut custom_block_type = String::new();
    let mut custom_block_content = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                let l = lang.as_ref();
                if l == "plot" || l == "question" || l == "spreadsheet" {
                    in_custom_block = true;
                    custom_block_type = l.to_string();

                    // Flush accumulated markdown
                    if !current_markdown.trim().is_empty() {
                        blocks.push(Block::Markdown(current_markdown.clone()));
                        current_markdown.clear();
                    }
                    continue;
                } else {
                    current_markdown.push_str(&format!("```{}\n", l));
                }
            }
            Event::End(Tag::CodeBlock(_)) => {
                if in_custom_block {
                    in_custom_block = false;
                    let parsed_json: Result<serde_json::Value, _> =
                        serde_json::from_str(&custom_block_content);
                    if let Ok(json) = parsed_json {
                        match custom_block_type.as_str() {
                            "plot" => blocks.push(Block::Plot(json)),
                            "question" => blocks.push(Block::Question(json)),
                            "spreadsheet" => blocks.push(Block::Spreadsheet(json)),
                            _ => {}
                        }
                    } else {
                        // Fallback if invalid JSON
                        blocks.push(Block::Markdown(format!(
                            "```{}\n{}\n```\n",
                            custom_block_type, custom_block_content
                        )));
                    }
                    custom_block_content.clear();
                    custom_block_type.clear();
                } else {
                    current_markdown.push_str("```\n");
                }
            }
            Event::Text(text) => {
                if in_custom_block {
                    custom_block_content.push_str(text.as_ref());
                } else {
                    current_markdown.push_str(text.as_ref());
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_custom_block {
                    custom_block_content.push('\n');
                } else {
                    current_markdown.push('\n');
                }
            }
            // For a complete implementation, all other markdown events (Heading, List, etc.)
            // should be reconstructed back to Markdown text or passed as semantic AST nodes.
            // For now, we capture raw text.
            _ => {}
        }
    }

    if !current_markdown.trim().is_empty() {
        blocks.push(Block::Markdown(current_markdown));
    }

    Document { blocks }
}
