use crate::masm_lang::{DIRECTIVES, KEYWORDS, REGISTERS, TYPE_KEYWORDS};
use crate::theme::{SyntaxColors, ThemeColor};
use ratatui::style::Style;
use ratatui::text::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Register,
    Directive,
    Number,
    String,
    Comment,
    Label,
    Operator,
    TypeKeyword,
    MacroCall,
    Plain,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
}

impl Token {
    pub fn new(text: impl Into<String>, token_type: TokenType) -> Self {
        Self {
            text: text.into(),
            token_type,
        }
    }
}

pub struct Highlighter;

impl Highlighter {
    pub fn tokenize_line(line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut pos = 0;

        while pos < chars.len() {
            let ch = chars[pos];

            // Comment - everything from ; to end of line
            if ch == ';' {
                let comment: String = chars[pos..].iter().collect();
                tokens.push(Token::new(comment, TokenType::Comment));
                break;
            }

            // String literal
            if ch == '"' || ch == '\'' {
                let quote = ch;
                let start = pos;
                pos += 1;
                while pos < chars.len() && chars[pos] != quote {
                    if chars[pos] == '\\' && pos + 1 < chars.len() {
                        pos += 1; // Skip escaped char
                    }
                    pos += 1;
                }
                if pos < chars.len() {
                    pos += 1; // Include closing quote
                }
                let string: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(string, TokenType::String));
                continue;
            }

            // Whitespace
            if ch.is_whitespace() {
                let start = pos;
                while pos < chars.len() && chars[pos].is_whitespace() {
                    pos += 1;
                }
                let ws: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(ws, TokenType::Plain));
                continue;
            }

            // Numbers (hex, binary, decimal)
            if ch.is_ascii_digit()
                || (ch == '0'
                    && pos + 1 < chars.len()
                    && (chars[pos + 1] == 'x' || chars[pos + 1] == 'X'))
            {
                let start = pos;

                // Check for 0x prefix
                if ch == '0'
                    && pos + 1 < chars.len()
                    && (chars[pos + 1] == 'x' || chars[pos + 1] == 'X')
                {
                    pos += 2;
                    while pos < chars.len() && chars[pos].is_ascii_hexdigit() {
                        pos += 1;
                    }
                } else {
                    // Regular number, could end with h (hex), b (binary), d (decimal), o (octal)
                    while pos < chars.len() && (chars[pos].is_ascii_hexdigit() || chars[pos] == '_')
                    {
                        pos += 1;
                    }
                    // Check for suffix
                    if pos < chars.len()
                        && matches!(chars[pos].to_ascii_lowercase(), 'h' | 'b' | 'd' | 'o')
                    {
                        pos += 1;
                    }
                }
                let num: String = chars[start..pos].iter().collect();
                tokens.push(Token::new(num, TokenType::Number));
                continue;
            }

            // Identifier or keyword
            if ch.is_alphabetic() || ch == '_' || ch == '.' || ch == '@' {
                let start = pos;
                pos += 1;
                while pos < chars.len()
                    && (chars[pos].is_alphanumeric()
                        || chars[pos] == '_'
                        || chars[pos] == '?'
                        || chars[pos] == '@')
                {
                    pos += 1;
                }
                let word: String = chars[start..pos].iter().collect();
                let lower = word.to_lowercase();

                // Check if followed by colon (label)
                let is_label = pos < chars.len() && chars[pos] == ':';

                let token_type = if is_label {
                    // Consume the colon as part of the label
                    pos += 1;
                    let label: String = chars[start..pos].iter().collect();
                    tokens.push(Token::new(label, TokenType::Label));
                    continue;
                } else if KEYWORDS.contains(&lower.as_str()) {
                    TokenType::Keyword
                } else if REGISTERS.contains(&lower.as_str()) {
                    TokenType::Register
                } else if DIRECTIVES.contains(&lower.as_str()) {
                    TokenType::Directive
                } else if TYPE_KEYWORDS.contains(&lower.as_str()) {
                    TokenType::TypeKeyword
                } else {
                    TokenType::Plain
                };

                tokens.push(Token::new(word, token_type));
                continue;
            }

            // Operators and punctuation
            let op_chars = [
                '+', '-', '*', '/', ',', '[', ']', '(', ')', ':', '<', '>', '=', '&', '|', '^',
                '!', '~',
            ];
            if op_chars.contains(&ch) {
                tokens.push(Token::new(ch.to_string(), TokenType::Operator));
                pos += 1;
                continue;
            }

            // Anything else - just add as plain text
            tokens.push(Token::new(ch.to_string(), TokenType::Plain));
            pos += 1;
        }

        tokens
    }

    pub fn highlight_line<'a>(line: &str, syntax_colors: &SyntaxColors) -> Vec<Span<'a>> {
        let tokens = Self::tokenize_line(line);

        tokens
            .into_iter()
            .map(|token| {
                let color = match token.token_type {
                    TokenType::Keyword => &syntax_colors.keyword,
                    TokenType::Register => &syntax_colors.register,
                    TokenType::Directive => &syntax_colors.directive,
                    TokenType::Number => &syntax_colors.number,
                    TokenType::String => &syntax_colors.string,
                    TokenType::Comment => &syntax_colors.comment,
                    TokenType::Label => &syntax_colors.label,
                    TokenType::Operator => &syntax_colors.operator,
                    TokenType::TypeKeyword => &syntax_colors.type_kw,
                    TokenType::MacroCall => &syntax_colors.macro_call,
                    TokenType::Plain => &syntax_colors.operator, // Use operator color for plain text (usually foreground)
                };
                Span::styled(token.text, Style::default().fg(color.to_color()))
            })
            .collect()
    }

    /// Highlight line with search matches
    pub fn highlight_line_with_search<'a>(
        line: &str,
        syntax_colors: &SyntaxColors,
        search_query: Option<&str>,
        search_match_color: &ThemeColor,
        current_match_positions: &[(usize, usize)], // (line, col) of current matches
        line_index: usize,
        current_match_color: &ThemeColor,
    ) -> Vec<Span<'a>> {
        let base_spans = Self::highlight_line(line, syntax_colors);

        let query = match search_query {
            Some(q) if !q.is_empty() => q,
            _ => return base_spans,
        };

        // Find all matches in this line
        let matches: Vec<(usize, usize)> = line
            .to_lowercase()
            .match_indices(&query.to_lowercase())
            .map(|(start, _)| (start, start + query.len()))
            .collect();

        if matches.is_empty() {
            return base_spans;
        }

        // Rebuild spans with search highlighting
        let mut result = Vec::new();
        let mut char_pos = 0;

        for span in base_spans {
            let span_text = span.content.to_string();
            let span_start = char_pos;
            let span_end = char_pos + span_text.len();
            let base_style = span.style;

            let mut current_pos = 0;
            let span_chars: Vec<char> = span_text.chars().collect();

            for &(match_start, match_end) in &matches {
                // Check if match overlaps with this span
                if match_end <= span_start || match_start >= span_end {
                    continue;
                }

                // Calculate overlap
                let overlap_start = match_start.saturating_sub(span_start);
                let overlap_end = (match_end - span_start).min(span_text.len());

                // Add text before match
                if overlap_start > current_pos {
                    let before: String = span_chars[current_pos..overlap_start].iter().collect();
                    result.push(Span::styled(before, base_style));
                }

                // Add matched text with highlight
                if overlap_end > overlap_start {
                    let matched: String = span_chars[overlap_start..overlap_end].iter().collect();
                    let is_current = current_match_positions
                        .iter()
                        .any(|&(l, c)| l == line_index && c == match_start);

                    let highlight_color = if is_current {
                        current_match_color
                    } else {
                        search_match_color
                    };

                    result.push(Span::styled(
                        matched,
                        Style::default()
                            .bg(highlight_color.to_color())
                            .fg(base_style.fg.unwrap_or(ratatui::style::Color::Black)),
                    ));
                    current_pos = overlap_end;
                }
            }

            // Add remaining text
            if current_pos < span_text.len() {
                let remaining: String = span_chars[current_pos..].iter().collect();
                result.push(Span::styled(remaining, base_style));
            }

            char_pos = span_end;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_comment() {
        let tokens = Highlighter::tokenize_line("; this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Comment);
    }

    #[test]
    fn test_tokenize_instruction() {
        let tokens = Highlighter::tokenize_line("mov eax, 10h");
        assert!(tokens
            .iter()
            .any(|t| t.token_type == TokenType::Keyword && t.text == "mov"));
        assert!(tokens
            .iter()
            .any(|t| t.token_type == TokenType::Register && t.text == "eax"));
        assert!(tokens
            .iter()
            .any(|t| t.token_type == TokenType::Number && t.text == "10h"));
    }

    #[test]
    fn test_tokenize_label() {
        let tokens = Highlighter::tokenize_line("main:");
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Label));
    }

    #[test]
    fn test_tokenize_string() {
        let tokens = Highlighter::tokenize_line("msg BYTE \"Hello\", 0");
        assert!(tokens
            .iter()
            .any(|t| t.token_type == TokenType::String && t.text == "\"Hello\""));
    }
}
