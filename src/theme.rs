use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub ui: UiColors,
    pub syntax: SyntaxColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub border_focused: ThemeColor,
    pub title: ThemeColor,
    pub title_focused: ThemeColor,
    pub line_numbers: ThemeColor,
    pub cursor_line: ThemeColor,
    pub selection: ThemeColor,
    pub selection_fg: ThemeColor,
    pub search_match: ThemeColor,
    pub search_match_current: ThemeColor,

    // Status bar
    pub status_bar_bg: ThemeColor,
    pub status_bar_fg: ThemeColor,
    pub mode_normal_bg: ThemeColor,
    pub mode_normal_fg: ThemeColor,
    pub mode_insert_bg: ThemeColor,
    pub mode_insert_fg: ThemeColor,
    pub mode_command_bg: ThemeColor,
    pub mode_command_fg: ThemeColor,
    pub mode_filetree_bg: ThemeColor,
    pub mode_filetree_fg: ThemeColor,
    pub mode_search_bg: ThemeColor,
    pub mode_search_fg: ThemeColor,

    // File tree
    pub file_tree_dir: ThemeColor,
    pub file_tree_file: ThemeColor,
    pub file_tree_asm: ThemeColor,
    pub file_tree_exe: ThemeColor,
    pub file_tree_selected: ThemeColor,

    // Output panel
    pub output_stdout: ThemeColor,
    pub output_stderr: ThemeColor,
    pub output_error: ThemeColor,
    pub output_info: ThemeColor,

    // Tabs
    pub tab_active_bg: ThemeColor,
    pub tab_active_fg: ThemeColor,
    pub tab_inactive_bg: ThemeColor,
    pub tab_inactive_fg: ThemeColor,

    // Diagnostics
    pub diagnostic_error: ThemeColor,
    pub diagnostic_warning: ThemeColor,
    pub diagnostic_error_line: ThemeColor,
    pub diagnostic_warning_line: ThemeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: ThemeColor,    // mov, push, pop, call, ret, jmp, etc.
    pub register: ThemeColor,   // eax, ebx, ecx, edx, esi, edi, esp, ebp
    pub directive: ThemeColor,  // .data, .code, PROC, ENDP, INCLUDE
    pub number: ThemeColor,     // hex, decimal, binary
    pub string: ThemeColor,     // "quoted strings"
    pub comment: ThemeColor,    // ; comments
    pub label: ThemeColor,      // labels:
    pub operator: ThemeColor,   // +, -, *, OFFSET, PTR
    pub type_kw: ThemeColor,    // BYTE, WORD, DWORD, etc.
    pub macro_call: ThemeColor, // macro invocations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThemeColor {
    Rgb { r: u8, g: u8, b: u8 },
    Named(String),
}

impl ThemeColor {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb { r, g, b }
    }

    pub fn to_color(&self) -> Color {
        match self {
            ThemeColor::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
            ThemeColor::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                _ => {
                    // Try parsing hex color #RRGGBB
                    if name.starts_with('#') && name.len() == 7 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            u8::from_str_radix(&name[1..3], 16),
                            u8::from_str_radix(&name[3..5], 16),
                            u8::from_str_radix(&name[5..7], 16),
                        ) {
                            return Color::Rgb(r, g, b);
                        }
                    }
                    Color::White
                }
            },
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: String::from("dark"),
            ui: UiColors {
                background: ThemeColor::rgb(30, 30, 30),
                foreground: ThemeColor::rgb(212, 212, 212),
                border: ThemeColor::rgb(60, 60, 60),
                border_focused: ThemeColor::rgb(100, 149, 237),
                title: ThemeColor::rgb(100, 100, 100),
                title_focused: ThemeColor::rgb(100, 149, 237),
                line_numbers: ThemeColor::rgb(90, 90, 90),
                cursor_line: ThemeColor::rgb(40, 40, 40),
                selection: ThemeColor::rgb(70, 70, 120),
                selection_fg: ThemeColor::rgb(255, 255, 255),
                search_match: ThemeColor::rgb(100, 80, 0),
                search_match_current: ThemeColor::rgb(150, 120, 0),

                status_bar_bg: ThemeColor::rgb(25, 25, 25),
                status_bar_fg: ThemeColor::rgb(150, 150, 150),
                mode_normal_bg: ThemeColor::rgb(86, 156, 214),
                mode_normal_fg: ThemeColor::rgb(30, 30, 30),
                mode_insert_bg: ThemeColor::rgb(78, 201, 176),
                mode_insert_fg: ThemeColor::rgb(30, 30, 30),
                mode_command_bg: ThemeColor::rgb(220, 220, 170),
                mode_command_fg: ThemeColor::rgb(30, 30, 30),
                mode_filetree_bg: ThemeColor::rgb(180, 130, 200),
                mode_filetree_fg: ThemeColor::rgb(30, 30, 30),
                mode_search_bg: ThemeColor::rgb(214, 157, 86),
                mode_search_fg: ThemeColor::rgb(30, 30, 30),

                file_tree_dir: ThemeColor::rgb(86, 156, 214),
                file_tree_file: ThemeColor::rgb(180, 180, 180),
                file_tree_asm: ThemeColor::rgb(220, 220, 170),
                file_tree_exe: ThemeColor::rgb(78, 201, 176),
                file_tree_selected: ThemeColor::rgb(50, 50, 80),

                output_stdout: ThemeColor::rgb(212, 212, 212),
                output_stderr: ThemeColor::rgb(244, 135, 113),
                output_error: ThemeColor::rgb(244, 71, 71),
                output_info: ThemeColor::rgb(86, 156, 214),

                tab_active_bg: ThemeColor::rgb(45, 45, 45),
                tab_active_fg: ThemeColor::rgb(212, 212, 212),
                tab_inactive_bg: ThemeColor::rgb(30, 30, 30),
                tab_inactive_fg: ThemeColor::rgb(128, 128, 128),

                diagnostic_error: ThemeColor::rgb(244, 71, 71),
                diagnostic_warning: ThemeColor::rgb(229, 192, 123),
                diagnostic_error_line: ThemeColor::rgb(50, 30, 30),
                diagnostic_warning_line: ThemeColor::rgb(50, 45, 30),
            },
            syntax: SyntaxColors {
                keyword: ThemeColor::rgb(86, 156, 214),     // Blue
                register: ThemeColor::rgb(156, 220, 254),   // Light blue
                directive: ThemeColor::rgb(197, 134, 192),  // Purple
                number: ThemeColor::rgb(181, 206, 168),     // Light green
                string: ThemeColor::rgb(206, 145, 120),     // Orange/brown
                comment: ThemeColor::rgb(106, 153, 85),     // Green
                label: ThemeColor::rgb(220, 220, 170),      // Yellow
                operator: ThemeColor::rgb(212, 212, 212),   // White
                type_kw: ThemeColor::rgb(78, 201, 176),     // Teal
                macro_call: ThemeColor::rgb(220, 220, 170), // Yellow
            },
        }
    }

    pub fn light() -> Self {
        Self {
            name: String::from("light"),
            ui: UiColors {
                background: ThemeColor::rgb(255, 255, 255),
                foreground: ThemeColor::rgb(30, 30, 30),
                border: ThemeColor::rgb(200, 200, 200),
                border_focused: ThemeColor::rgb(0, 122, 204),
                title: ThemeColor::rgb(120, 120, 120),
                title_focused: ThemeColor::rgb(0, 122, 204),
                line_numbers: ThemeColor::rgb(150, 150, 150),
                cursor_line: ThemeColor::rgb(240, 240, 240),
                selection: ThemeColor::rgb(173, 214, 255),
                selection_fg: ThemeColor::rgb(0, 0, 0),
                search_match: ThemeColor::rgb(255, 235, 150),
                search_match_current: ThemeColor::rgb(255, 215, 0),

                status_bar_bg: ThemeColor::rgb(240, 240, 240),
                status_bar_fg: ThemeColor::rgb(80, 80, 80),
                mode_normal_bg: ThemeColor::rgb(0, 122, 204),
                mode_normal_fg: ThemeColor::rgb(255, 255, 255),
                mode_insert_bg: ThemeColor::rgb(22, 163, 74),
                mode_insert_fg: ThemeColor::rgb(255, 255, 255),
                mode_command_bg: ThemeColor::rgb(180, 140, 0),
                mode_command_fg: ThemeColor::rgb(255, 255, 255),
                mode_filetree_bg: ThemeColor::rgb(147, 51, 234),
                mode_filetree_fg: ThemeColor::rgb(255, 255, 255),
                mode_search_bg: ThemeColor::rgb(234, 88, 12),
                mode_search_fg: ThemeColor::rgb(255, 255, 255),

                file_tree_dir: ThemeColor::rgb(0, 122, 204),
                file_tree_file: ThemeColor::rgb(60, 60, 60),
                file_tree_asm: ThemeColor::rgb(180, 140, 0),
                file_tree_exe: ThemeColor::rgb(22, 163, 74),
                file_tree_selected: ThemeColor::rgb(220, 235, 252),

                output_stdout: ThemeColor::rgb(30, 30, 30),
                output_stderr: ThemeColor::rgb(220, 38, 38),
                output_error: ThemeColor::rgb(185, 28, 28),
                output_info: ThemeColor::rgb(0, 122, 204),

                tab_active_bg: ThemeColor::rgb(255, 255, 255),
                tab_active_fg: ThemeColor::rgb(30, 30, 30),
                tab_inactive_bg: ThemeColor::rgb(240, 240, 240),
                tab_inactive_fg: ThemeColor::rgb(128, 128, 128),

                diagnostic_error: ThemeColor::rgb(220, 38, 38),
                diagnostic_warning: ThemeColor::rgb(180, 140, 0),
                diagnostic_error_line: ThemeColor::rgb(254, 226, 226),
                diagnostic_warning_line: ThemeColor::rgb(254, 249, 195),
            },
            syntax: SyntaxColors {
                keyword: ThemeColor::rgb(0, 0, 255),      // Blue
                register: ThemeColor::rgb(0, 128, 128),   // Teal
                directive: ThemeColor::rgb(175, 0, 219),  // Purple
                number: ThemeColor::rgb(9, 134, 88),      // Green
                string: ThemeColor::rgb(163, 21, 21),     // Red/brown
                comment: ThemeColor::rgb(0, 128, 0),      // Green
                label: ThemeColor::rgb(121, 94, 38),      // Brown
                operator: ThemeColor::rgb(30, 30, 30),    // Black
                type_kw: ThemeColor::rgb(38, 127, 153),   // Teal
                macro_call: ThemeColor::rgb(121, 94, 38), // Brown
            },
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: String::from("dracula"),
            ui: UiColors {
                background: ThemeColor::rgb(40, 42, 54),
                foreground: ThemeColor::rgb(248, 248, 242),
                border: ThemeColor::rgb(68, 71, 90),
                border_focused: ThemeColor::rgb(189, 147, 249),
                title: ThemeColor::rgb(98, 114, 164),
                title_focused: ThemeColor::rgb(189, 147, 249),
                line_numbers: ThemeColor::rgb(98, 114, 164),
                cursor_line: ThemeColor::rgb(68, 71, 90),
                selection: ThemeColor::rgb(68, 71, 90),
                selection_fg: ThemeColor::rgb(248, 248, 242),
                search_match: ThemeColor::rgb(241, 250, 140),
                search_match_current: ThemeColor::rgb(255, 184, 108),

                status_bar_bg: ThemeColor::rgb(33, 34, 44),
                status_bar_fg: ThemeColor::rgb(248, 248, 242),
                mode_normal_bg: ThemeColor::rgb(189, 147, 249),
                mode_normal_fg: ThemeColor::rgb(40, 42, 54),
                mode_insert_bg: ThemeColor::rgb(80, 250, 123),
                mode_insert_fg: ThemeColor::rgb(40, 42, 54),
                mode_command_bg: ThemeColor::rgb(241, 250, 140),
                mode_command_fg: ThemeColor::rgb(40, 42, 54),
                mode_filetree_bg: ThemeColor::rgb(255, 121, 198),
                mode_filetree_fg: ThemeColor::rgb(40, 42, 54),
                mode_search_bg: ThemeColor::rgb(255, 184, 108),
                mode_search_fg: ThemeColor::rgb(40, 42, 54),

                file_tree_dir: ThemeColor::rgb(189, 147, 249),
                file_tree_file: ThemeColor::rgb(248, 248, 242),
                file_tree_asm: ThemeColor::rgb(241, 250, 140),
                file_tree_exe: ThemeColor::rgb(80, 250, 123),
                file_tree_selected: ThemeColor::rgb(68, 71, 90),

                output_stdout: ThemeColor::rgb(248, 248, 242),
                output_stderr: ThemeColor::rgb(255, 85, 85),
                output_error: ThemeColor::rgb(255, 85, 85),
                output_info: ThemeColor::rgb(139, 233, 253),

                tab_active_bg: ThemeColor::rgb(68, 71, 90),
                tab_active_fg: ThemeColor::rgb(248, 248, 242),
                tab_inactive_bg: ThemeColor::rgb(40, 42, 54),
                tab_inactive_fg: ThemeColor::rgb(98, 114, 164),

                diagnostic_error: ThemeColor::rgb(255, 85, 85),
                diagnostic_warning: ThemeColor::rgb(241, 250, 140),
                diagnostic_error_line: ThemeColor::rgb(60, 42, 54),
                diagnostic_warning_line: ThemeColor::rgb(55, 55, 54),
            },
            syntax: SyntaxColors {
                keyword: ThemeColor::rgb(255, 121, 198),   // Pink
                register: ThemeColor::rgb(139, 233, 253),  // Cyan
                directive: ThemeColor::rgb(189, 147, 249), // Purple
                number: ThemeColor::rgb(189, 147, 249),    // Purple
                string: ThemeColor::rgb(241, 250, 140),    // Yellow
                comment: ThemeColor::rgb(98, 114, 164),    // Comment gray
                label: ThemeColor::rgb(80, 250, 123),      // Green
                operator: ThemeColor::rgb(248, 248, 242),  // White
                type_kw: ThemeColor::rgb(139, 233, 253),   // Cyan
                macro_call: ThemeColor::rgb(80, 250, 123), // Green
            },
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: String::from("gruvbox"),
            ui: UiColors {
                background: ThemeColor::rgb(40, 40, 40),
                foreground: ThemeColor::rgb(235, 219, 178),
                border: ThemeColor::rgb(80, 73, 69),
                border_focused: ThemeColor::rgb(215, 153, 33),
                title: ThemeColor::rgb(146, 131, 116),
                title_focused: ThemeColor::rgb(215, 153, 33),
                line_numbers: ThemeColor::rgb(124, 111, 100),
                cursor_line: ThemeColor::rgb(60, 56, 54),
                selection: ThemeColor::rgb(80, 73, 69),
                selection_fg: ThemeColor::rgb(235, 219, 178),
                search_match: ThemeColor::rgb(215, 153, 33),
                search_match_current: ThemeColor::rgb(250, 189, 47),

                status_bar_bg: ThemeColor::rgb(50, 48, 47),
                status_bar_fg: ThemeColor::rgb(168, 153, 132),
                mode_normal_bg: ThemeColor::rgb(131, 165, 152),
                mode_normal_fg: ThemeColor::rgb(40, 40, 40),
                mode_insert_bg: ThemeColor::rgb(184, 187, 38),
                mode_insert_fg: ThemeColor::rgb(40, 40, 40),
                mode_command_bg: ThemeColor::rgb(250, 189, 47),
                mode_command_fg: ThemeColor::rgb(40, 40, 40),
                mode_filetree_bg: ThemeColor::rgb(211, 134, 155),
                mode_filetree_fg: ThemeColor::rgb(40, 40, 40),
                mode_search_bg: ThemeColor::rgb(254, 128, 25),
                mode_search_fg: ThemeColor::rgb(40, 40, 40),

                file_tree_dir: ThemeColor::rgb(131, 165, 152),
                file_tree_file: ThemeColor::rgb(235, 219, 178),
                file_tree_asm: ThemeColor::rgb(250, 189, 47),
                file_tree_exe: ThemeColor::rgb(184, 187, 38),
                file_tree_selected: ThemeColor::rgb(80, 73, 69),

                output_stdout: ThemeColor::rgb(235, 219, 178),
                output_stderr: ThemeColor::rgb(251, 73, 52),
                output_error: ThemeColor::rgb(204, 36, 29),
                output_info: ThemeColor::rgb(131, 165, 152),

                tab_active_bg: ThemeColor::rgb(60, 56, 54),
                tab_active_fg: ThemeColor::rgb(235, 219, 178),
                tab_inactive_bg: ThemeColor::rgb(40, 40, 40),
                tab_inactive_fg: ThemeColor::rgb(146, 131, 116),

                diagnostic_error: ThemeColor::rgb(251, 73, 52),
                diagnostic_warning: ThemeColor::rgb(250, 189, 47),
                diagnostic_error_line: ThemeColor::rgb(60, 40, 40),
                diagnostic_warning_line: ThemeColor::rgb(55, 50, 40),
            },
            syntax: SyntaxColors {
                keyword: ThemeColor::rgb(251, 73, 52),      // Red
                register: ThemeColor::rgb(131, 165, 152),   // Aqua
                directive: ThemeColor::rgb(211, 134, 155),  // Purple
                number: ThemeColor::rgb(211, 134, 155),     // Purple
                string: ThemeColor::rgb(184, 187, 38),      // Green
                comment: ThemeColor::rgb(146, 131, 116),    // Gray
                label: ThemeColor::rgb(250, 189, 47),       // Yellow
                operator: ThemeColor::rgb(235, 219, 178),   // Fg
                type_kw: ThemeColor::rgb(254, 128, 25),     // Orange
                macro_call: ThemeColor::rgb(131, 165, 152), // Aqua
            },
        }
    }

    pub fn nord() -> Self {
        Self {
            name: String::from("nord"),
            ui: UiColors {
                background: ThemeColor::rgb(46, 52, 64),
                foreground: ThemeColor::rgb(236, 239, 244),
                border: ThemeColor::rgb(67, 76, 94),
                border_focused: ThemeColor::rgb(136, 192, 208),
                title: ThemeColor::rgb(76, 86, 106),
                title_focused: ThemeColor::rgb(136, 192, 208),
                line_numbers: ThemeColor::rgb(76, 86, 106),
                cursor_line: ThemeColor::rgb(59, 66, 82),
                selection: ThemeColor::rgb(67, 76, 94),
                selection_fg: ThemeColor::rgb(236, 239, 244),
                search_match: ThemeColor::rgb(235, 203, 139),
                search_match_current: ThemeColor::rgb(208, 135, 112),

                status_bar_bg: ThemeColor::rgb(59, 66, 82),
                status_bar_fg: ThemeColor::rgb(229, 233, 240),
                mode_normal_bg: ThemeColor::rgb(136, 192, 208),
                mode_normal_fg: ThemeColor::rgb(46, 52, 64),
                mode_insert_bg: ThemeColor::rgb(163, 190, 140),
                mode_insert_fg: ThemeColor::rgb(46, 52, 64),
                mode_command_bg: ThemeColor::rgb(235, 203, 139),
                mode_command_fg: ThemeColor::rgb(46, 52, 64),
                mode_filetree_bg: ThemeColor::rgb(180, 142, 173),
                mode_filetree_fg: ThemeColor::rgb(46, 52, 64),
                mode_search_bg: ThemeColor::rgb(208, 135, 112),
                mode_search_fg: ThemeColor::rgb(46, 52, 64),

                file_tree_dir: ThemeColor::rgb(129, 161, 193),
                file_tree_file: ThemeColor::rgb(236, 239, 244),
                file_tree_asm: ThemeColor::rgb(235, 203, 139),
                file_tree_exe: ThemeColor::rgb(163, 190, 140),
                file_tree_selected: ThemeColor::rgb(67, 76, 94),

                output_stdout: ThemeColor::rgb(236, 239, 244),
                output_stderr: ThemeColor::rgb(191, 97, 106),
                output_error: ThemeColor::rgb(191, 97, 106),
                output_info: ThemeColor::rgb(136, 192, 208),

                tab_active_bg: ThemeColor::rgb(67, 76, 94),
                tab_active_fg: ThemeColor::rgb(236, 239, 244),
                tab_inactive_bg: ThemeColor::rgb(46, 52, 64),
                tab_inactive_fg: ThemeColor::rgb(76, 86, 106),

                diagnostic_error: ThemeColor::rgb(191, 97, 106),
                diagnostic_warning: ThemeColor::rgb(235, 203, 139),
                diagnostic_error_line: ThemeColor::rgb(56, 52, 64),
                diagnostic_warning_line: ThemeColor::rgb(56, 56, 64),
            },
            syntax: SyntaxColors {
                keyword: ThemeColor::rgb(129, 161, 193),    // Blue
                register: ThemeColor::rgb(136, 192, 208),   // Cyan
                directive: ThemeColor::rgb(180, 142, 173),  // Purple
                number: ThemeColor::rgb(180, 142, 173),     // Purple
                string: ThemeColor::rgb(163, 190, 140),     // Green
                comment: ThemeColor::rgb(76, 86, 106),      // Gray
                label: ThemeColor::rgb(235, 203, 139),      // Yellow
                operator: ThemeColor::rgb(236, 239, 244),   // White
                type_kw: ThemeColor::rgb(208, 135, 112),    // Orange
                macro_call: ThemeColor::rgb(136, 192, 208), // Cyan
            },
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "dracula" => Self::dracula(),
            "gruvbox" => Self::gruvbox(),
            "nord" => Self::nord(),
            _ => Self::dark(),
        }
    }

    pub fn available_themes() -> Vec<&'static str> {
        vec!["dark", "light", "dracula", "gruvbox", "nord"]
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::gruvbox()
    }
}
