mod action;
mod ui;

use action::{Action, ActionList, ZellijAction::*};

use zellij_tile::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    action: Action,
    // file_name_search_results: Vec<String>,
    // file_contents_search_results: Vec<String>,
    loading: bool,
    loading_animation_offset: u8,
    should_open_floating: bool,
    search_filter: EnvironmentFrom,
    display_rows: usize,
    display_columns: usize,
    // displayed_search_results: (usize, Vec<String>), // usize is selected index
    last_pane: PaneManifest,
    last_tab: TabInfo, // TODO: useless?
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
            PermissionType::OpenFiles,
            PermissionType::OpenTerminalsOrPlugins,
        ]);
        subscribe(&[
            /*EventType::PaneUpdate, EventType::TabUpdate,*/ EventType::Key,
        ]);

        // TODO: This may change as I’m not convinced the `configuration`’s API is good for this
        self.action.set(format!(
            "{}",
            configuration.get("command").unwrap_or(&Default::default())
        ));
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            // Event::PaneUpdate(pane_info) => {
            //     self.last_pane = pane_info;
            //     // should_render = true;
            // }
            // Event::TabUpdate(tab_info) => {
            //     self.last_tab = tab_info.iter().find(|t| t.active).unwrap().clone();
            //     // should_render = true;
            // }
            Event::Key(key) => {
                self.handle_key(key);
                should_render = true;
            }
            Event::PermissionRequestResult(_status) => {
                // should_render = true;
            }
            _ => unimplemented!("{:?}", event),
        };

        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.change_size(rows, cols);
        print!("{}", self);
    }
}

impl State {
    pub fn handle_key(&mut self, key: Key) {
        match key {
            // Key::Down => self.move_search_selection_down(),
            // Key::Up => self.move_search_selection_up(),
            Key::Char('\n') => self.start_action(),
            // Key::BackTab => self.open_search_result_in_terminal(),
            Key::Ctrl('f') => {
                self.should_open_floating = !self.should_open_floating;
            }
            Key::Ctrl('e') => self.search_filter.progress(), // TODO: should also be a toggelable bool
            // Key::Esc | Key::Ctrl('c') => {
            //     if !self.search_term.is_empty() {
            //         self.clear_state();
            //     } else {
            //         hide_self();
            //     }
            // }
            _ => self.append_to_search_term(key),
        }
    }

    pub fn change_size(&mut self, rows: usize, cols: usize) {
        self.display_rows = rows;
        self.display_columns = cols;
    }

    fn append_to_search_term(&mut self, key: Key) {
        match key {
            Key::Char(character) => {
                self.action.push(character);
            }
            Key::Backspace => {
                self.action.pop();
                if self.action.len() == 0 {
                    // self.clear_state();
                }
            }
            _ => {}
        }
    }

    fn start_action(&mut self) {
        // Parse la ligne en séparant aux "espaces"
        let action = self.action.action().clone();
        match action {
            ActionList::Technical(_a) => {}
            ActionList::Zellij(a) => {
                let mut done = true;

                match a {
                    ClearScreen => {
                        // TODO: Clear applies on the focused pane, focus to the previous one before clearing the screen/scrollback
                        // focus_previous_pane();
                        clear_screen();
                    }
                    CloseFocus => close_focus(),
                    CloseFocusTab => close_focused_tab(),
                    ClosePluginPane { id } => match id {
                        Some(id) => close_plugin_pane(id),
                        None => done = false,
                    },
                    CloseTerminalPane { id } => match id {
                        Some(id) => close_terminal_pane(id),
                        None => done = false,
                    },
                    DecodeLengthDelimiter { buffer } => {
                        let _ = decode_length_delimiter(buffer.as_slice());
                    }
                    Detach => {
                        detach();
                    }
                    EditScrollback => {
                        // TODO: Edit scrollback applies on the focused pane, focus to the previous one before clearing the screen/scrollback
                        // focus_previous_pane();
                        edit_scrollback();
                    }
                    EncodeLengthDelimiter { mut buffer } => {
                        let _ = encode_length_delimiter(buffer.len(), &mut buffer);
                    }

                    Edit(FileToOpen {
                        path,
                        line_number,
                        cwd,
                    }) => {
                        let file = FileToOpen {
                            path: path.to_owned(),
                            line_number: line_number.to_owned(),
                            cwd: cwd.to_owned(),
                        };

                        if self.should_open_floating {
                            open_file_floating(file);
                        } else {
                            open_file(file);
                        }
                    }
                    NewPane { path } => {
                        if self.should_open_floating {
                            open_terminal_floating(path);
                        } else {
                            open_terminal(path);
                        }
                    }
                    Run(CommandToRun { path, args, cwd }) => {
                        let (path, args) = match self.search_filter {
                            // TODO: get this as parameter
                            EnvironmentFrom::ZellijSession => (path, args),
                            EnvironmentFrom::DefaultShell => {
                                let mut a = vec![
                                    "-c".to_string(),
                                    path.to_str().unwrap_or_default().to_string(),
                                ];
                                a.append(&mut args.clone());

                                ("fish".into(), a) // TODO: get user’s shell
                            }
                        };
                        let cmd = CommandToRun { path, args, cwd };

                        if self.should_open_floating {
                            open_command_pane_floating(cmd);
                        } else {
                            open_command_pane(cmd);
                        }
                    }
                }

                if done {
                    self.action.clear();
                }
            }
        };
    }

    // fn open_search_result_in_editor(&mut self) {
    //     match self.selected_search_result_entry() {
    //         Some(SearchResult::File { path, .. }) => {
    //             if self.should_open_floating {
    //                 open_file_floating(&PathBuf::from(path))
    //             } else {
    //                 open_file(&PathBuf::from(path));
    //             }
    //         }
    //         Some(SearchResult::LineInFile {
    //             path, line_number, ..
    //         }) => {
    //             if self.should_open_floating {
    //                 open_file_with_line_floating(&PathBuf::from(path), line_number);
    //             } else {
    //                 open_file_with_line(&PathBuf::from(path), line_number);
    //             }
    //         }
    //         None => eprintln!("Search results not found"),
    //     }
    // }

    // fn open_search_result_in_terminal(&mut self) {
    //     let dir_path_of_result = |path: &str| -> PathBuf {
    //         let file_path = PathBuf::from(path);
    //         let mut dir_path = file_path.components();
    //         dir_path.next_back(); // remove file name to stay with just the folder
    //         dir_path.as_path().into()
    //     };
    //     let selected_search_result_entry = self.selected_search_result_entry();
    //     if let Some(SearchResult::File { path, .. }) | Some(SearchResult::LineInFile { path, .. }) =
    //         selected_search_result_entry
    //     {
    //         let dir_path = dir_path_of_result(&path);
    //         if self.should_open_floating {
    //             open_terminal_floating(&dir_path);
    //         } else {
    //             open_terminal(&dir_path);
    //         }
    //     }
    // }
}

#[derive(Serialize, Deserialize)]
pub enum EnvironmentFrom {
    ZellijSession,
    DefaultShell,
}

impl EnvironmentFrom {
    pub fn progress(&mut self) {
        match &self {
            &EnvironmentFrom::ZellijSession => *self = EnvironmentFrom::DefaultShell,
            &EnvironmentFrom::DefaultShell => *self = EnvironmentFrom::ZellijSession,
        }
    }
}

impl Default for EnvironmentFrom {
    fn default() -> Self {
        EnvironmentFrom::ZellijSession
    }
}
