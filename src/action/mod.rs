use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActionList {
    None,
    NewPane {
        path: String,
    },
    Run {
        cmd: String,
        args: Vec<String>,
        cwd: Option<String>, // TODO: `Option<PathBuf>`?
        env: BTreeMap<String, Option<String>>,
    },
    Edit {
        path: String,
        line: Option<usize>,
    },
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
                let mut env: BTreeMap<String, Option<String>> = Default::default();

                let mut are_args = true;

                let mut val = action_arguments.next();
                while val.is_some() {
                    // Safety: checked in the while loop
                    let v = unsafe { val.unwrap_unchecked() };

                    match v.as_str() {
                        "---cwd" => cwd = action_arguments.next(),
                        "---env" => are_args = false,
                        _ => {
                            if cmd.is_empty() {
                                cmd = v;
                            } else if are_args {
                                args.push(v);
                            } else {
                                env.insert(v, action_arguments.next());
                            }
                        }
                    };
                    val = action_arguments.next();
                }
                Self::Run {
                    cmd,
                    args,
                    cwd,
                    env,
                }
            }
            "edit" => {
                let last = action_arguments.next_back();
                let line = last
                    .clone()
                    .unwrap_or(String::new())
                    .parse::<usize>()
                    .map_or(None, |v| Some(v));
                let mut path = action_arguments.collect::<Vec<String>>().join(" ");
                if line == None {
                    if !path.is_empty() {
                        path.push(' ');
                    }
                    path.push_str(&last.unwrap_or_default())
                }
                Self::Edit { path, line }
            }
            _ => Self::None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
