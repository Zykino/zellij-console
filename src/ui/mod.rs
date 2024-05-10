use std::fmt::{Display, Formatter};
use strum::EnumMessage;

use zellij_tile::prelude::{ui_components::*, CommandToRun, FileToOpen, Palette};

use crate::action::ActionList;
use crate::{EnvironmentFrom, State};

pub const WHITE: u8 = 15;
pub const BLACK: u8 = 16;

const REQUIRED_COLOR: usize = 2;
const OPTIONAL_COLOR: usize = 3;
const UNSETTABLE_COLOR: usize = 4;

impl Display for ActionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Unknown => {
                let text = Text::new(r#"Type a command or "help" if you need a list of commands"#)
                    .color_range(1, 19..23);
                serialize_text(&text)
            }
            Self::Help { selection } => {
                let text: Vec<_> = ActionList::documentation()
                    .enumerate()
                    .flat_map(|(i, variant)| -> Vec<NestedListItem> {
                        let name = variant
                            .get_serializations()
                            .first()
                            .expect("At least one serialization is garanteed");
                        let shortcut = "Shortcut variants";

                        let mut result = Vec::with_capacity(2);
                        result.push(
                            NestedListItem::new(format!(
                                "{}:\t{}",
                                name,
                                variant.get_documentation().unwrap_or_else(|| panic!(
                                    "{variant:?} should have a line of documentation"
                                ))
                            ))
                            .color_range(1, 0..name.len()),
                        );

                        if i == selection.row {
                            // TODO: Maybe only show the other ways of writing the command when selected when the line is selected
                            // TODO: When selected, "Enter" should use that commands to replace the current command
                            result.push(
                                NestedListItem::new(format!(
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

                serialize_nested_list(&text)
            }

            Self::ClearScreen => String::from("ClearScreen"),
            Self::CloseFocus => String::from("CloseFocus"),
            Self::CloseFocusTab => String::from("CloseFocusTab "),
            Self::ClosePluginPane { id } => format!(
                "ClosePluginPane\n{} {}",
                serialize_text(&Text::new("PATH:").color_range(REQUIRED_COLOR, 0..4)),
                id.unwrap_or_default() // TODO: not default when unset…
            ),
            Self::CloseTerminalPane { id } => format!(
                "CloseTerminalPane\n{} {}",
                serialize_text(&Text::new("PATH:").color_range(REQUIRED_COLOR, 0..4)),
                id.unwrap_or_default() // TODO: not default when unset…
            ),
            Self::Detach => String::from("Detach"),
            Self::EditScrollback => String::from("EditScrollback"),
            Self::Edit(FileToOpen {
                path,
                line_number: line,
                cwd,
            }) => format!(
                "Edit\n{} {:?}\n{} {}\n{} {:?}",
                serialize_text(&Text::new("PATH:").color_range(REQUIRED_COLOR, 0..4)),
                path,
                serialize_text(&Text::new("LINE:").color_range(REQUIRED_COLOR, 0..4)),
                line.unwrap_or_default(),
                serialize_text(&Text::new("DIRECTORY:").color_range(UNSETTABLE_COLOR, 0..4)),
                cwd.clone().unwrap_or_default(),
            ),
            Self::NewPane { path } => format!(
                "New pane\n{} {}",
                serialize_text(&Text::new("PATH:").color_range(REQUIRED_COLOR, 0..4)),
                path
            ),
            Self::Run(CommandToRun { path, args, cwd }) => format!(
                "Run\n{} {:?}\n{} {:?}\n{} {:?}",
                serialize_text(&Text::new("COMMAND:").color_range(REQUIRED_COLOR, 0..7)),
                path,
                serialize_text(&Text::new("ARGUMENTS:").color_range(OPTIONAL_COLOR, 0..9)),
                args,
                serialize_text(&Text::new("DIRECTORY:").color_range(OPTIONAL_COLOR, 0..9)),
                cwd.clone().unwrap_or_default(),
            ),
        };

        let text = match self {
            Self::Unknown | Self::Help { .. } => text,
            _ => {
                format!(
                    "{} {}",
                    serialize_text(&Text::new("ACTION:").color_range(1, 0..6)),
                    text
                )
            }
        };

        write!(f, "{}", text)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        // Use the user’s theme
        let theme = self.mode_info.style.colors;
        write!(f, "{}", self.render_action_line(theme))?;
        // TODO: Only print the control line when its options are usefull… or remove it entirely to integrate the options in the command actions
        write!(f, "{}", self.render_controls_line(theme))?;
        Ok(())
    }
}

impl State {
    pub fn render_action_line(&self, _theme: Palette) -> String {
        // TODO: I don’t think this is a good setup: the 2 methods do not coexist yet, so… I only keep it for reference while waiting for the new theme spec
        // let c = match theme.cyan {
        //     zellij_tile::prelude::PaletteColor::Rgb(_) => 1,
        //     zellij_tile::prelude::PaletteColor::EightBit(c) => {
        //         print!("EightBit: {}{:?}{:?}", c, theme.source, theme.theme_hue);
        //         c as usize
        //     }
        // };
        format!(
            "{} {}{}\n{}\n",
            serialize_text(&Text::new("PROMPT:").color_range(1, 0..6)),
            self.action.as_str(),
            styled_text_background(WHITE, " "), // "Cursor" representation
            self.action.action(),
        )
    }

    pub fn render_controls_line(&self, _theme: Palette) -> String {
        // let has_results = true; // !self.displayed_search_results.1.is_empty();
        let tiled_floating_control =
            self.new_floating_control("Ctrl + f", self.should_open_floating);
        let names_contents_control = self.new_filter_control("Ctrl + e", &self.search_filter);

        serialize_ribbon_line_with_coordinates(
            [tiled_floating_control, names_contents_control],
            0,
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
