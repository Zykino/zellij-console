use std::fmt::{Display, Formatter};
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

use zellij_tile::prelude::{ui_components::*, CommandToRun, FileToOpen};
mod zellij_ui_ext;
use zellij_ui_ext::*;

use crate::action::ActionList;
use crate::{EnvironmentFrom, State};

// TODO: use the user’s theme, when available in Zellij
pub const CYAN: u8 = 51;
pub const GRAY_LIGHT: u8 = 238;
pub const GRAY_DARK: u8 = 245;
pub const WHITE: u8 = 15;
pub const BLACK: u8 = 16;
// pub const RED: u8 = 124;
// pub const GREEN: u8 = 154;
pub const ORANGE: u8 = 166;

const REQUIRED_COLOR: u8 = GRAY_DARK;
const OPTIONAL_COLOR: u8 = GRAY_LIGHT;
const UNSETTABLE_COLOR: u8 = ORANGE;

impl Display for ActionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: remove the clone? Only needed because `EncodeLengthDelimiter` need a mutable access
        let text = match self.clone() {
            Self::Unknown => {
                let text = Text::new(r#"Type a command or "help" if you need a list of commands"#)
                    .color_range(1, 19..23);
                format_text(text)
            }
            Self::Help { selection } => {
                let text = ActionList::documentation()
                    .enumerate()
                    .flat_map(|(i, variant)| -> Vec<NestedListItem> {
                        let name = variant
                            .get_serializations()
                            .first()
                            .expect("At least one serialization is garanteed");
                        let shortcut = "Shortcut variants";

                        let mut result = Vec::with_capacity(2);
                        result.push(
                            NestedListItem::new(&format!(
                                "{}:\t{}",
                                name,
                                variant.get_documentation().expect(&format!(
                                    "{variant:?} should have a line of documentation"
                                ))
                            ))
                            .color_range(1, 0..name.len()),
                        );

                        if i == selection.row as usize {
                            // TODO: Maybe only show the other ways of writing the command when selected when the line is selected
                            // TODO: When selected, "Enter" should use that commands to replace the current command
                            result.push(
                                NestedListItem::new(&format!(
                                    "{}:\t{}",
                                    shortcut,
                                    variant.get_serializations().join(", ")
                                ))
                                .indent(1)
                                .color_range(2, 0..shortcut.len()),
                            );

                            result.iter().map(|item| item.clone().selected()).collect()
                        } else {
                            result.iter().map(|item| item.to_owned()).collect()
                        }
                    })
                    .collect();

                format_nested_list(text)
            }

            Self::ClearScreen => String::from("ClearScreen"),
            Self::CloseFocus => String::from("CloseFocus"),
            Self::CloseFocusTab => String::from("CloseFocusTab "),
            Self::ClosePluginPane { id } => format!(
                "ClosePluginPane\n{} {}",
                styled_text_foreground(REQUIRED_COLOR, &bold("PATH:")),
                id.unwrap_or_default() // TODO: not default when unset…
            ),
            Self::CloseTerminalPane { id } => format!(
                "CloseTerminalPane\n{} {}",
                styled_text_foreground(REQUIRED_COLOR, &bold("PATH:")),
                id.unwrap_or_default() // TODO: not default when unset…
            ),
            Self::DecodeLengthDelimiter { buffer } => format!(
                "DecodeLengthDelimiter\n{} {:?}:{:?}",
                styled_text_foreground(REQUIRED_COLOR, &bold("PATH:")),
                zellij_tile::shim::decode_length_delimiter(buffer.as_slice()),
                buffer
            ),
            Self::Detach => String::from("Detach"),
            Self::EditScrollback => String::from("EditScrollback"),
            Self::EncodeLengthDelimiter { mut buffer } => format!(
                "EncodeLengthDelimiter\n{} {:?}:{:?}",
                styled_text_foreground(REQUIRED_COLOR, &bold("PATH:")),
                zellij_tile::shim::encode_length_delimiter(buffer.len(), &mut buffer),
                buffer
            ),

            Self::Edit(FileToOpen {
                path,
                line_number: line,
                cwd,
            }) => format!(
                "Edit\n{} {:?}\n{} {}\n{} {:?}",
                styled_text_foreground(OPTIONAL_COLOR, &bold("PATH:")),
                path,
                styled_text_foreground(OPTIONAL_COLOR, &bold("LINE:")),
                line.unwrap_or_default(),
                styled_text_foreground(UNSETTABLE_COLOR, &bold("DIRECTORY:")),
                cwd.clone().unwrap_or_default(),
            ),
            Self::NewPane { path } => format!(
                "New pane\n{} {}",
                styled_text_foreground(REQUIRED_COLOR, &bold("PATH:")),
                path
            ),
            Self::Run(CommandToRun { path, args, cwd }) => format!(
                "Run\n{} {:?}\n{} {:?}\n{} {:?}",
                styled_text_foreground(REQUIRED_COLOR, &bold("COMMAND:")),
                path,
                styled_text_foreground(OPTIONAL_COLOR, &bold("ARGUMENTS:")),
                args,
                styled_text_foreground(OPTIONAL_COLOR, &bold("DIRECTORY:")),
                cwd.clone().unwrap_or_default(),
            ),
        };

        let text = match self {
            Self::Unknown | Self::Help { .. } => text,
            _ => {
                format!(
                    "{} {}",
                    styled_text_foreground(REQUIRED_COLOR, &bold("ACTION:")),
                    text
                )
            }
        };

        write!(f, "{}", text)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.render_action_line())?;
        write!(f, "{}", self.render_controls_line())?;
        Ok(())
    }
}

impl State {
    pub fn render_action_line(&self) -> String {
        format!(
            "{} {}{}\n{}\n",
            styled_text_foreground(CYAN, &bold("PROMPT:")),
            self.action.as_str(),
            styled_text_background(WHITE, " "), // "Cursor" representation
            self.action.action(),
        )
    }

    pub fn render_controls_line(&self) -> String {
        // let has_results = true; // !self.displayed_search_results.1.is_empty();
        let tiled_floating_control =
            self.new_floating_control("Ctrl + f", self.should_open_floating);
        let names_contents_control = self.new_filter_control("Ctrl + e", &self.search_filter);

        format_ribbon_line(
            &[tiled_floating_control, names_contents_control],
            self.display.rows,
            None,
            None,
        )
    }

    fn new_floating_control(&self, key: &'static str, should_open_floating: bool) -> Text {
        if should_open_floating {
            Text::new(format!("<{}> OPEN FLOATING", key)).color_range(0, 1..=key.len())
        } else {
            Text::new(format!("<{}> OPEN TILED", key)).color_range(0, 1..=key.len())
        }
    }

    fn new_filter_control(&self, key: &'static str, search_filter: &EnvironmentFrom) -> Text {
        match search_filter {
            EnvironmentFrom::ZellijSession => {
                Text::new(format!("<{}> ZELLIJ’S ENVIRONMENT", key)).color_range(0, 1..=key.len())
            }
            EnvironmentFrom::DefaultShell => {
                Text::new(format!("<{}> SHELL’S ENVIRONMENT", key)).color_range(0, 1..=key.len())
            }
        }
    }
}

pub fn bold(text: &str) -> String {
    format!("\u{1b}[1m{}\u{1b}[m", text)
}

pub fn styled_text(foreground_color: u8, background_color: u8, text: &str) -> String {
    format!(
        "\u{1b}[38;5;{};48;5;{}m{}\u{1b}[m",
        foreground_color, background_color, text
    )
}

pub fn styled_text_foreground(foreground_color: u8, text: &str) -> String {
    format!("\u{1b}[38;5;{}m{}\u{1b}[m", foreground_color, text)
}

pub fn styled_text_background(background_color: u8, text: &str) -> String {
    format!("\u{1b}[48;5;{}m{}\u{1b}[m", background_color, text)
}
