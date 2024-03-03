use std::{collections::binary_heap::Iter, iter::FilterMap, path::PathBuf};
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};
use strum_macros::{EnumIter, EnumMessage, EnumProperty};

use zellij_tile::prelude::{CommandToRun, FileToOpen};

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Selection {
    pub(crate) row: usize,
    max: usize,
}

#[derive(Debug, Default, Clone, EnumIter, EnumMessage, EnumProperty)]
pub(crate) enum ActionList {
    /*
        Technical actions
    */
    /// No action have been recognized
    #[default]
    #[strum(props(Hidden = "True"))] // FIXME: Find a better property name and value
    Unknown,
    /// Show the list of commands
    #[strum(serialize = "Help", serialize = "?")]
    Help { selection: Selection },

    /*
        Zellij actions (sorted alphabetically)
    */
    /// Clear the last focused pane’s scroll buffer
    #[strum(
        serialize = "ClearScreen",
        serialize = "Clear-Screen",
        serialize = "Clear_Screen",
        serialize = "Clear",
        serialize = "cl"
    )]
    ClearScreen,
    /// Close the focused pane
    #[strum(
        serialize = "Close",
        serialize = "Exit",
        serialize = "CloseFocus",
        serialize = "Close-Focus",
        serialize = "Close_Focus",
        serialize = "ClosePane",
        serialize = "Close-Pane",
        serialize = "Close_Pane",
        serialize = "ExitFocus",
        serialize = "Exit-Focus",
        serialize = "Exit_Focus",
        serialize = "ExitPane",
        serialize = "Exit-Pane",
        serialize = "Exit_Pane"
    )]
    CloseFocus,
    /// Close the focused tab
    #[strum(
        serialize = "CloseFocusTab",
        serialize = "Close-Focus-Tab",
        serialize = "Close_Focus_Tab",
        serialize = "CloseTab",
        serialize = "Close-Tab",
        serialize = "Close_Tab"
    )]
    CloseFocusTab,
    /// Close the specified plugin pane
    #[strum(
        serialize = "ClosePluginPane",
        serialize = "Close-Plugin-Pane",
        serialize = "Close_Plugin_Pane",
        serialize = "ClosePlugin",
        serialize = "Close-Plugin",
        serialize = "Close_Plugin"
    )]
    ClosePluginPane { id: Option<u32> },
    /// Close the specified terminal pane
    #[strum(
        serialize = "CloseTerminalPane",
        serialize = "Close-Terminal-Pane",
        serialize = "Close_Terminal_Pane",
        serialize = "CloseTerminal",
        serialize = "Close-Terminal",
        serialize = "Close_Terminal"
    )]
    CloseTerminalPane { id: Option<u32> },
    /// Detach from the current session
    Detach,
    /// Edit a pane scrollback
    EditScrollback,

    /// Edit a file in a new edit pane
    Edit(FileToOpen),
    /// Open a new pane in the current tab
    #[strum(
        serialize = "NewPane",
        serialize = "New-Pane",
        serialize = "New_Pane",
        serialize = "np"
    )]
    NewPane { path: String },
    /// Run a command in a new edit pane
    Run(CommandToRun),
}

fn deserialize_action(action: &String, variant: impl EnumMessage) -> bool {
    variant
        .get_serializations()
        .iter()
        .map(|a| a.to_lowercase())
        .collect::<Vec<_>>()
        .contains(action)
}

impl ActionList {
    fn parse(command: String) -> Self {
        let mut split = command.split_whitespace();
        let action = split.next().unwrap_or_default().to_lowercase();
        let mut action_arguments = split.map(|v| v.to_owned());

        match action.as_str() {
            // Zellij actions
            _ if deserialize_action(&action, ActionList::ClearScreen) => ActionList::ClearScreen,
            _ if deserialize_action(&action, ActionList::CloseFocus) => ActionList::CloseFocus,
            _ if deserialize_action(&action, ActionList::CloseFocusTab) => {
                ActionList::CloseFocusTab
            }
            _ if deserialize_action(
                &action,
                ActionList::ClosePluginPane {
                    id: Default::default(), // TODO: Am I forced to defines every variable. Can’t I just `Default::default()`?                },
                },
            ) =>
            {
                let id = action_arguments
                    .last()
                    .map(|s| s.parse::<u32>().ok())
                    .unwrap_or_default();

                ActionList::ClosePluginPane { id }
            }
            _ if deserialize_action(
                &action,
                ActionList::CloseTerminalPane {
                    id: Default::default(), // TODO: Am I forced to defines every variable. Can’t I just `Default::default()`?                },
                },
            ) =>
            {
                let id = action_arguments
                    .last()
                    .map(|s| s.parse::<u32>().ok())
                    .unwrap_or_default();

                ActionList::CloseTerminalPane { id }
            }
            _ if deserialize_action(&action, ActionList::Detach) => ActionList::Detach,
            _ if deserialize_action(&action, ActionList::EditScrollback) => {
                ActionList::EditScrollback
            }

            _ if deserialize_action(&action, ActionList::Edit(Default::default())) => {
                let last = action_arguments.next_back();
                let line_number = last
                    .clone()
                    .unwrap_or(String::new())
                    .parse::<usize>()
                    .map_or(None, |v| Some(v));
                let mut path = action_arguments.collect::<Vec<String>>().join(" ");
                if line_number == None {
                    if !path.is_empty() {
                        path.push(' ');
                    }
                    path.push_str(&last.unwrap_or_default())
                }

                ActionList::Edit(FileToOpen {
                    path: path.into(),
                    line_number,
                    cwd: None, // TODO: get the cwd
                })
            }
            _ if deserialize_action(
                &action,
                ActionList::NewPane {
                    path: Default::default(),
                },
            ) =>
            {
                ActionList::NewPane {
                    path: action_arguments.collect::<Vec<String>>().join(" "),
                }
            }
            _ if deserialize_action(&action, ActionList::Run(Default::default())) => {
                let mut cmd = String::new();
                let mut args: Vec<String> = Default::default();
                let mut cwd = Default::default();

                let mut val = action_arguments.next();
                while val.is_some() {
                    // Safety: checked in the while loop
                    let v = unsafe { val.unwrap_unchecked() };

                    match v.as_str() {
                        "---cwd" => cwd = action_arguments.next().map(PathBuf::from),
                        _ => {
                            if cmd.is_empty() {
                                cmd = v;
                            } else {
                                args.push(v);
                            }
                        }
                    };
                    val = action_arguments.next();
                }

                ActionList::Run(CommandToRun {
                    path: cmd.into(),
                    args,
                    cwd,
                })
            }

            // Technicals
            _ if deserialize_action(
                &action,
                ActionList::Help {
                    selection: Default::default(),
                },
            ) =>
            {
                let max = ActionList::documentation().count();
                ActionList::Help {
                    selection: Selection {
                        max,
                        ..Default::default()
                    },
                }
            }

            _ => ActionList::Unknown,
        }
    }

    pub(crate) fn documentation() -> impl Iterator<Item = ActionList> {
        ActionList::iter().filter(|v| v.get_str("Hidden").is_none())
    }
}

// TODO: use self referential struct? So we do not store the data twice since `action` is computed from `command`
#[derive(Debug, Clone)]
pub(crate) struct Action {
    command: String,
    action: ActionList,
}

impl Action {
    pub(crate) fn action(&self) -> &ActionList {
        &self.action
    }
}

impl Action {
    fn parse_action(&mut self) {
        // TODO: maybe don’t reparse all each time?
        // if charactere.is_whitespace() {
        self.action = ActionList::parse(self.command.clone());
        // }
    }

    pub(crate) fn set(&mut self, command: &str) {
        self.command = command.to_string();
        self.parse_action();
    }

    // Emulate a `String`
    pub(crate) fn push(&mut self, charactere: char) {
        self.command.push(charactere);
        self.parse_action();
    }
    pub(crate) fn pop(&mut self) -> Option<char> {
        let res = self.command.pop();
        self.parse_action();
        res
    }
    pub(crate) fn clear(&mut self) {
        self.command.clear();
        self.parse_action();
    }
    pub(crate) fn len(&self) -> usize {
        self.command.len()
    }
    pub(crate) fn as_str(&self) -> &str {
        &self.command
    }

    pub(crate) fn selection_up(&mut self) {
        self.action = match self.action {
            ActionList::Help { mut selection } => {
                if selection.row != 0 {
                    selection.row = selection.row - 1;
                } else {
                    selection.row = selection.max - 1
                }
                ActionList::Help { selection }
            }
            _ => self.action.clone(),
        }
    }

    pub(crate) fn selection_down(&mut self) {
        self.action = match self.action {
            ActionList::Help { mut selection } => {
                selection.row = (selection.row + 1) % selection.max;
                ActionList::Help { selection }
            }
            _ => self.action.clone(),
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Action {
            command: String::new(),
            action: ActionList::Unknown,
        }
    }
}
