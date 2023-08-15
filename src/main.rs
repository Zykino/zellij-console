mod action;
mod ui;

use action::Action;

use zellij_tile::prelude::*;

use serde::{Deserialize, Serialize};

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
    fn load(&mut self) {
        subscribe(&[EventType::PaneUpdate, EventType::TabUpdate, EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;

        match event {
            Event::PaneUpdate(pane_info) => {
                self.last_pane = pane_info;
                // should_render = true;
            }
            Event::TabUpdate(tab_info) => {
                self.last_tab = tab_info.iter().find(|t| t.active).unwrap().clone();
                // should_render = true;
            }
            Event::Key(key) => {
                self.handle_key(key);
                should_render = true;
            }
            _ => unimplemented!(),
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
            Key::Ctrl('r') => self.search_filter.progress(),
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
        match self.action.as_str() {
            "run" => {
                let (cmd, args) = match self.search_filter {
                    // TODO: get this as parameter
                    EnvironmentFrom::ZellijSession => ("env", vec![]),
                    EnvironmentFrom::DefaultShell => ("fish", vec!["-c", "env"]),
                    EnvironmentFrom::LastPane => todo!(),
                };

                if self.should_open_floating {
                    open_command_pane_floating(cmd, args);
                } else {
                    open_command_pane(cmd, args);
                }
            }
            "edit" => {
                let file = "Cargo.toml"; // TODO: get this as parameter
                if self.should_open_floating {
                    open_file_floating(file);
                } else {
                    open_file(file);
                }
            }
            "new-pane" => {
                let path = "."; // TODO: get this as parameter
                if self.should_open_floating {
                    open_terminal_floating(path);
                } else {
                    open_terminal(path);
                }
            }
            _ => (),
        }
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
    LastPane,
}

impl EnvironmentFrom {
    pub fn progress(&mut self) {
        match &self {
            &EnvironmentFrom::ZellijSession => *self = EnvironmentFrom::DefaultShell,
            &EnvironmentFrom::DefaultShell => *self = EnvironmentFrom::LastPane,
            &EnvironmentFrom::LastPane => *self = EnvironmentFrom::ZellijSession,
        }
    }
}

impl Default for EnvironmentFrom {
    fn default() -> Self {
        EnvironmentFrom::ZellijSession
    }
}
