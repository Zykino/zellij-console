use std::path::PathBuf;

use zellij_tile::prelude::{CommandToRun, FileToOpen};

#[derive(Debug, Clone)]
pub(crate) enum ActionList {
    None,
    NewPane { path: String },
    Run(CommandToRun),
    Edit(FileToOpen),
}

impl ActionList {
    fn parse(command: String) -> Self {
        let mut split = command.split_whitespace();
        let action = split.next().unwrap_or_default().to_lowercase();
        let action = action.as_str();
        let mut action_arguments = split.map(|v| v.to_owned());

        match action {
            "new-pane" => Self::NewPane {
                path: action_arguments.collect::<Vec<String>>().join(" "),
            },
            "run" => {
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
                Self::Run(CommandToRun {
                    path: cmd.into(),
                    args,
                    cwd,
                })
            }
            "edit" => {
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

                Self::Edit(FileToOpen {
                    path: path.into(),
                    line_number,
                    cwd: None, // TODO: get the cwd
                })
            }
            _ => Self::None,
        }
    }
}

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

// Emulate a `String`
impl Action {
    pub(crate) fn push(&mut self, charactere: char) {
        self.command.push(charactere);
        // TODO: maybe don’t reparse all each time?
        // if charactere.is_whitespace() {
        self.action = ActionList::parse(self.command.clone());
        // }
    }
    pub(crate) fn pop(&mut self) -> Option<char> {
        let res = self.command.pop();
        // TODO: maybe don’t reparse all each time?
        // if res.is_some_and(|c| c.is_whitespace()) {
        self.action = ActionList::parse(self.command.clone());
        // }
        res
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
            command: String::from(""),
            action: ActionList::None,
        }
    }
}
