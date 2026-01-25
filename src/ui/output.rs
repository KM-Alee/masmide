use crate::theme::Theme;
use ratatui::{
    prelude::*,
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Stdout,
    Stderr,
    Error,
    Info,
    Success,
    Divider,
}

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub text: String,
    pub output_type: OutputType,
}

pub struct OutputState {
    pub lines: Vec<OutputLine>,
    pub scroll_offset: usize,
    pub visible_height: usize,
    pub auto_scroll: bool,
}

impl OutputState {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll_offset: 0,
            visible_height: 10, // Default, will be updated on render
            auto_scroll: true,
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    pub fn append_stdout(&mut self, text: &str) {
        for line in text.lines() {
            // Keep empty lines for program output formatting
            self.lines.push(OutputLine {
                text: line.to_string(),
                output_type: OutputType::Stdout,
            });
        }
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn append_stderr(&mut self, text: &str) {
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let output_type = if line.to_lowercase().contains("error") {
                OutputType::Error
            } else if line.to_lowercase().contains("warning") {
                OutputType::Stderr
            } else {
                OutputType::Stderr
            };
            self.lines.push(OutputLine {
                text: line.to_string(),
                output_type,
            });
        }
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn append_error(&mut self, text: &str) {
        self.lines.push(OutputLine {
            text: text.to_string(),
            output_type: OutputType::Error,
        });
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn append_info(&mut self, text: &str) {
        self.lines.push(OutputLine {
            text: text.to_string(),
            output_type: OutputType::Info,
        });
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn append_success(&mut self, text: &str) {
        self.lines.push(OutputLine {
            text: text.to_string(),
            output_type: OutputType::Success,
        });
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn append_divider(&mut self) {
        self.lines.push(OutputLine {
            text: String::new(),
            output_type: OutputType::Divider,
        });
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        let content_height = self.visible_height.saturating_sub(2);
        if self.lines.len() > content_height {
            self.scroll_offset = self.lines.len() - content_height;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        self.auto_scroll = false;
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let content_height = self.visible_height.saturating_sub(2);
        let max_scroll = self.lines.len().saturating_sub(content_height);
        self.scroll_offset = (self.scroll_offset + lines).min(max_scroll);
        // Re-enable auto-scroll if we're at the bottom
        if self.scroll_offset >= max_scroll {
            self.auto_scroll = true;
        }
    }

    pub fn page_up(&mut self) {
        let page_size = self.visible_height.saturating_sub(4).max(1);
        self.scroll_up(page_size);
    }

    pub fn page_down(&mut self) {
        let page_size = self.visible_height.saturating_sub(4).max(1);
        self.scroll_down(page_size);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn update_visible_height(&mut self, height: usize) {
        self.visible_height = height;
        // Re-adjust scroll if needed after resize
        let content_height = height.saturating_sub(2);
        let max_scroll = self.lines.len().saturating_sub(content_height);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &mut OutputState, focused: bool, theme: &Theme) {
    let border_style = if focused {
        Style::default().fg(theme.ui.border_focused.to_color())
    } else {
        Style::default().fg(theme.ui.border.to_color())
    };

    let title = if focused { " Output " } else { " Output " };
    let title_style = if focused {
        Style::default()
            .fg(theme.ui.title_focused.to_color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.ui.title.to_color())
    };

    let block = Block::default()
        .title(Span::styled(title, title_style))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(border_style)
        .style(Style::default().bg(theme.ui.background.to_color()));

    let inner = block.inner(area);
    let visible_height = inner.height as usize;
    let _inner_width = inner.width as usize;

    // Update state with current visible height for proper scrolling
    state.update_visible_height(visible_height);

    // Show placeholder if empty
    if state.is_empty() {
        let placeholder = Paragraph::new(Line::from(vec![
            Span::styled(
                " Press ",
                Style::default().fg(theme.ui.line_numbers.to_color()),
            ),
            Span::styled(
                "F5",
                Style::default()
                    .fg(theme.ui.title_focused.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to build+run  ",
                Style::default().fg(theme.ui.line_numbers.to_color()),
            ),
            Span::styled(
                "F6",
                Style::default()
                    .fg(theme.ui.title_focused.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " build  ",
                Style::default().fg(theme.ui.line_numbers.to_color()),
            ),
            Span::styled(
                "F7",
                Style::default()
                    .fg(theme.ui.title_focused.to_color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " run",
                Style::default().fg(theme.ui.line_numbers.to_color()),
            ),
        ]))
        .block(block);
        frame.render_widget(placeholder, area);
        return;
    }

    // Build output lines with padding
    let mut text: Vec<Line> = Vec::new();

    // Add top padding (empty line)
    text.push(Line::from(""));

    // Calculate how many content lines we can show (reserve 2 for top/bottom padding)
    let content_height = visible_height.saturating_sub(2);

    // Add content lines
    for line in state
        .lines
        .iter()
        .skip(state.scroll_offset)
        .take(content_height)
    {
        let styled_line = match line.output_type {
            OutputType::Divider => {
                // Empty line as visual separator
                Line::from("")
            }
            OutputType::Success => Line::from(vec![
                Span::styled(
                    "  ✓ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &line.text,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            OutputType::Error => Line::from(vec![
                Span::styled(
                    "  ✗ ",
                    Style::default()
                        .fg(theme.ui.diagnostic_error.to_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &line.text,
                    Style::default().fg(theme.ui.diagnostic_error.to_color()),
                ),
            ]),
            OutputType::Stderr => Line::from(vec![
                Span::styled(
                    "  ⚠ ",
                    Style::default().fg(theme.ui.diagnostic_warning.to_color()),
                ),
                Span::styled(
                    &line.text,
                    Style::default().fg(theme.ui.diagnostic_warning.to_color()),
                ),
            ]),
            OutputType::Info => Line::from(vec![
                Span::styled("  → ", Style::default().fg(theme.ui.output_info.to_color())),
                Span::styled(
                    &line.text,
                    Style::default().fg(theme.ui.output_info.to_color()),
                ),
            ]),
            OutputType::Stdout => {
                // Regular program output - clean, indented
                Line::from(Span::styled(
                    format!("    {}", line.text),
                    Style::default().fg(theme.ui.foreground.to_color()),
                ))
            }
        };
        text.push(styled_line);
    }

    // Add bottom padding (empty line)
    text.push(Line::from(""));

    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}
