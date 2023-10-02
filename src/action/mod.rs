use std::path::PathBuf;
use strum::{EnumMessage, IntoEnumIterator};
use strum_macros::{EnumIter, EnumMessage};

use zellij_tile::prelude::{CommandToRun, FileToOpen};

#[derive(Debug, Clone, EnumMessage)]
pub(crate) enum TechnicalAction {
    /// No commanad is recognized
    None,
    /// Show the list of commands
    #[strum(serialize = "Help", serialize = "?")]
    Help,
}

#[derive(Debug, Default, Clone, EnumMessage, EnumIter)]
pub(crate) enum ZellijAction {
    // We need a default value… but only want to easily create any enum type. So by default we will use the first one, alphabetically.
    #[default]

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
    CloseFocus,
    /// Close the focused tab
    #[strum(
        serialize = "CloseFocusTab",
        serialize = "Close-Focus-Tab",
        serialize = "Close_Focus_Tab"
    )]
    CloseFocusTab,

    /// Detach from the current session
    Detach,
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

#[derive(Debug, Clone, EnumMessage)]
pub(crate) enum ActionList {
    Technical(TechnicalAction),
    Zellij(ZellijAction),
}

impl ActionList {
    fn parse(command: String) -> Self {
        let mut split = command.split_whitespace();
        let action = split.next().unwrap_or_default().to_lowercase();
        // let action = action.as_str();
        let mut action_arguments = split.map(|v| v.to_owned());

        match action.as_str() {
            // ZellijAction
            _ if ZellijAction::ClearScreen
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
                Self::Zellij(ZellijAction::ClearScreen)
            }
            _ if ZellijAction::CloseFocus
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
                Self::Zellij(ZellijAction::CloseFocus)
            }
            _ if ZellijAction::CloseFocusTab
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
                Self::Zellij(ZellijAction::CloseFocusTab)
            }

            _ if ZellijAction::Detach
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
                Self::Zellij(ZellijAction::Detach)
            }
            _ if ZellijAction::Edit(Default::default())
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
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

                Self::Zellij(ZellijAction::Edit(FileToOpen {
                    path: path.into(),
                    line_number,
                    cwd: None, // TODO: get the cwd
                }))
            }
            _ if ZellijAction::NewPane {
                path: Default::default(), // TODO: Am I forced to defines every variable. Can’t I just `Default::default()`?
            }
            .get_serializations()
            .iter()
            .map(|a| a.to_lowercase())
            .collect::<Vec<_>>()
            .contains(&action) =>
            {
                Self::Zellij(ZellijAction::NewPane {
                    path: action_arguments.collect::<Vec<String>>().join(" "),
                })
            }
            _ if ZellijAction::Run(Default::default())
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
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
                Self::Zellij(ZellijAction::Run(CommandToRun {
                    path: cmd.into(),
                    args,
                    cwd,
                }))
            }

            // Technicals
            _ if TechnicalAction::Help
                .get_serializations()
                .iter()
                .map(|a| a.to_lowercase())
                .collect::<Vec<_>>()
                .contains(&action) =>
            {
                Self::Technical(TechnicalAction::Help)
            }
            _ => Self::Technical(TechnicalAction::None),
        }
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

    pub(crate) fn set(&mut self, command: String) {
        self.command = command;
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
}

impl Default for Action {
    fn default() -> Self {
        Action {
            command: String::new(),
            action: ActionList::Technical(TechnicalAction::None),
        }
    }
}
