mod controls_line;
mod loading_animation;
mod selection_controls_area;

use std::fmt::{Display, Formatter};
use strum::{EnumMessage, IntoEnumIterator};

use zellij_tile::prelude::{CommandToRun, FileToOpen};

use crate::action::{ActionList, TechnicalAction, ZellijAction};
use crate::ui::controls_line::{Control, ControlsLine};
use crate::ui::selection_controls_area::SelectionControlsArea;
use crate::State;

pub const CYAN: u8 = 51;
pub const GRAY_LIGHT: u8 = 238;
pub const GRAY_DARK: u8 = 245;
pub const WHITE: u8 = 15;
pub const BLACK: u8 = 16;
pub const RED: u8 = 124;
pub const GREEN: u8 = 154;
pub const ORANGE: u8 = 166;

const REQUIRED_COLOR: u8 = GRAY_DARK;
const OPTIONAL_COLOR: u8 = GRAY_LIGHT;
const UNSETTABLE_COLOR: u8 = ORANGE;

impl Display for TechnicalAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::None => {
                String::from(r#"Type a command or "help" if you need a list of commands"#)
            }
            Self::Help => {
                let mut msg = String::new();
                // let mut msg = format!(
                //     "Help\n{} ",
                //     styled_text_foreground(REQUIRED_COLOR, &bold("TODO:")),
                //     // toto.get_documentation()
                //     //
                // );
                ZellijAction::iter().for_each(|variant| {
                    msg.push_str(&format!(
                        "{}:\t{}\n\t{} {}\n",
                        variant.get_serializations().first().unwrap(), // Safe since at least one serialization is garanteed
                        variant
                            .get_documentation()
                            .expect(&format!("{variant:?} should have a line of documentation")),
                        // TODO: Maybe only show the other ways of writing the command when selected when the line is selected
                        // TODO: When selected, "Enter" should use that commands to replace the current command
                        styled_text_foreground(OPTIONAL_COLOR, &bold("Shortcuts:")),
                        variant.get_serializations().join(", ")
                    ));
                });
                msg
            }
        };

        write!(f, "{}", text)
    }
}

impl Display for ZellijAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self.clone() {
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

        // TODO: Should this be in the ActionList impl? The user may not need to know about this distinction of commands
        //       At the same time, maybe we need to add an equivalent in the TechnicalAction impl?
        let text = format!(
            "{} {}",
            styled_text_foreground(REQUIRED_COLOR, &bold("ACTION:")),
            text
        );

        write!(f, "{}", text)
    }
}

impl Display for ActionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Technical(t) => t.to_string(),
            Self::Zellij(z) => z.to_string(),
        };

        write!(f, "{}", text)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.render_action_line())?;
        // write!(f, "{}", self.render_search_results())?;
        // write!(f, "{}", self.render_selection_control_area())?;
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
            styled_text_background(WHITE, " "),
            self.action.action(),
        )
    }

    pub fn render_search_results(&self) -> String {
        let mut space_for_results = self.display_rows.saturating_sub(3); // title and both controls lines
        let mut to_render = String::new();
        // for (i, search_result) in self.displayed_search_results.1.iter().enumerate() {
        //     let result_height = search_result.rendered_height();
        //     if space_for_results < result_height {
        //         break;
        //     }
        //     space_for_results -= result_height;
        //     let index_of_selected_result = self.displayed_search_results.0;
        //     let is_selected = i == index_of_selected_result;
        //     let is_below_search_result = i > index_of_selected_result;
        //     let rendered_result =
        //         search_result.render(self.display_columns, is_selected, is_below_search_result);
        //     to_render.push_str(&format!("{}", rendered_result));
        //     to_render.push('\n')
        // }
        to_render
    }

    pub fn render_selection_control_area(&self) -> String {
        // let rows_for_results = self.rows_for_results();
        // if !self.displayed_search_results.1.is_empty() {
        //     format!(
        //         "{}\n",
        //         SelectionControlsArea::new(rows_for_results, self.display_columns)
        //             .render(self.number_of_lines_in_displayed_search_results())
        //     )
        // } else {
        //     format!(
        //         "{}\n",
        //         SelectionControlsArea::new(rows_for_results, self.display_columns)
        //             .render_empty_lines()
        //     )
        // }

        String::new()
    }

    pub fn render_controls_line(&self) -> String {
        let has_results = true; // !self.displayed_search_results.1.is_empty();
        let tiled_floating_control =
            Control::new_floating_control("Ctrl f", self.should_open_floating);
        let names_contents_control = Control::new_filter_control("Ctrl e", &self.search_filter);
        if self.loading {
            ControlsLine::new(
                vec![tiled_floating_control, names_contents_control],
                Some(vec!["Scanning folder", "Scanning", "S"]),
            )
            .with_animation_offset(self.loading_animation_offset)
            .render(self.display_columns, has_results)
        } else {
            ControlsLine::new(vec![tiled_floating_control, names_contents_control], None)
                .render(self.display_columns, has_results)
        }
    }
}

pub fn bold(text: &str) -> String {
    format!("\u{1b}[1m{}\u{1b}[m", text)
}

pub fn underline(text: &str) -> String {
    format!("\u{1b}[4m{}\u{1b}[m", text)
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

pub fn color_line_to_end(background_color: u8) -> String {
    format!("\u{1b}[48;5;{}m\u{1b}[0K", background_color)
}

pub fn arrow(foreground: u8, background: u8) -> String {
    format!("\u{1b}[38;5;{}m\u{1b}[48;5;{}m", foreground, background)
}

pub fn dot(foreground: u8, background: u8) -> String {
    format!("\u{1b}[38;5;{};48;5;{}m•", foreground, background)
}
