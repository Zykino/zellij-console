use std::fmt::{Debug, Display, Formatter};
use strum::{EnumMessage, EnumProperty};

use zellij_tile::prelude::{ui_components::*, CommandToRun, FileToOpen, Palette};

use crate::action::{ActionList, Interface, Selection};
use crate::{EnvironmentFrom, State};

const WHITE: u8 = 15;
const BLACK: u8 = 16;

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
            Self::Unavailable {
                action,
                calling_interface,
            } => {
                // TODO: implement Display for interface? --> This would help get its `len()`
                let command = action
                    .get_serializations()
                    .first()
                    .expect("At least one serialization is guaranteed");
                let available_interface = action.get_usable_interface().expect("Command {action:?} stored in `ActionList::Unavailable`, but cannot be used on any interface");

                let indice_after_command = 5 + command.len();
                let indice_before_current_interface = 59 + indice_after_command;
                let indice_after_current_interface = indice_before_current_interface + 4; // 4 because interfaces different from "all" are 4 char long ("Pane" & "Pipe")
                let indice_before_available_interfaces = indice_after_current_interface + 38;
                let indice_after_available_interfaces = indice_before_available_interfaces + 4; // 4 because interfaces different from "all" are 4 char long ("Pane" & "Pipe")

                let mut texts = Vec::with_capacity(2);
                texts.push(Text::new(format!(
                    "The `{command}` command is not available through the current interface (`{calling_interface:?}`). It should makes sens only on the `{available_interface}` interface.",
                ))
                    .color_range(0,..)
                    .color_range(1, 5..indice_after_command)
                    .color_range(1, indice_before_current_interface..indice_after_current_interface)
                    .color_range(1, indice_before_available_interfaces..indice_after_available_interfaces)
                );

                if let Interface::Pipe = calling_interface {
                    texts.push(Text::new("Hint: You can force execution of the command by adding the argument `--args=force_available`.\r\n\tWARNING: This may execute the command more than once if you have multiple users connected to this session.")
                    .color_range(2, ..4)
                    .color_range(1, 69..91)
                    .color_range(0, 96..103)
                )
                }

                texts.iter().fold(String::new(), |accu, item| {
                    accu + "\n" + &serialize_text(item)
                })
            }
            Self::HelpAll { selection }
            | Self::HelpPane { selection }
            | Self::HelpPipe { selection } => {
                let docs: Box<dyn Iterator<Item = ActionList>> = match self {
                    Self::HelpAll { .. } => Box::new(ActionList::filter_any()),
                    Self::HelpPane { .. } => Box::new(ActionList::filter_pane()),
                    Self::HelpPipe { .. } => Box::new(ActionList::filter_pipe()),
                    _ => panic!("Should be one of the help modes"),
                };
                let text: Vec<_> = docs
                    .enumerate()
                    .flat_map(|(i, variant)| -> Vec<NestedListItem> {
                        let name = variant
                            .get_serializations()
                            .first()
                            .expect("At least one serialization is garanteed");
                        let shortcut_msg = "Shortcut variations";
                        let interface_msg = "Interface restriction";

                        let mut result = Vec::with_capacity(3);
                        result.push(
                            NestedListItem::new(format!(
                                "{}:\t{}",
                                name,
                                variant
                                    .get_documentation()
                                    .expect("{variant:?} should have a line of documentation")
                            ))
                            .color_range(1, 0..name.len()),
                        );

                        // TODO: || Interface::Pipe { // --> Expand all when asking for Help from the CLI.
                        let (show, select) = match selection {
                            Selection::One { row, max: _ } => (&i == row, &i == row ),
                            Selection::Expand => (true, false),
                        };

                        if show {
                            result.push(
                                NestedListItem::new(format!(
                                    "{}:\t{}",
                                    shortcut_msg,
                                    variant.get_serializations().join(", ")
                                ))
                                    .indent(1)
                                    .color_range(2, 0..shortcut_msg.len()),
                            );
                            
                            if !variant.usable_in_all() {
                                result.push(
                                    NestedListItem::new(format!(
                                        "{}:\t{}",
                                        interface_msg,
                                        variant.get_usable_interface().expect("Interface restriction should be set for command we show help on")
                                    ))
                                        .indent(1)
                                        .color_range(0, 0..interface_msg.len()),
                                );
                            }
                        }

                        if select {
                            result.iter().map(|item| item.clone().selected()).collect()
                        } else {
                            result.iter().map(|item| item.to_owned()).collect()
                        }
                    })
                    .collect();

                serialize_nested_list(&text)
            }

            // Self::ClearScreen => String::from("ClearScreen"),
            // Self::CloseFocus => String::from("CloseFocus"),
            // Self::CloseFocusTab => String::from("CloseFocusTab "),
            // Self::ClosePluginPane { id } => format!(
            //     "ClosePluginPane\n{} {}",
            //     serialize_text(&Text::new("PATH:").color_range(REQUIRED_COLOR, 0..4)),
            //     id.unwrap_or_default() // TODO: not default when unset…
            // ),
            // Self::CloseTerminalPane { id } => format!(
            //     "CloseTerminalPane\n{} {}",
            //     serialize_text(&Text::new("PATH:").color_range(REQUIRED_COLOR, 0..4)),
            //     id.unwrap_or_default() // TODO: not default when unset…
            // ),
            Self::DetachEveryone => String::from("DetachEveryone"),
            Self::DetachMe => String::from("DetachMe"),
            Self::DetachOthers => String::from("DetachOthers"),
            // Self::EditScrollback => String::from("EditScrollback"),
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
            Self::Unknown
            | Self::Unavailable { .. }
            | Self::HelpAll { .. }
            | Self::HelpPane { .. }
            | Self::HelpPipe { .. } => text,
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
        write!(f, "{}", self.render_action_line())?;
        // TODO: Only print the control line when its options are usefull… or remove it entirely to integrate the options in the command actions
        write!(f, "{}", self.render_controls_line())?;
        Ok(())
    }
}

impl State {
    pub fn render_action_line(&self) -> String {
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

    pub fn render_controls_line(&self) -> String {
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
