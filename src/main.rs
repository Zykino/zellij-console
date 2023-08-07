mod ui;

use zellij_tile::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Default)]
struct State {
    action: String,
    // file_name_search_results: Vec<String>,
    // file_contents_search_results: Vec<String>,
    loading: bool,
    loading_animation_offset: u8,
    should_open_floating: bool,
    search_filter: EnvironmentFrom,
    display_rows: usize,
    display_columns: usize,
    displayed_search_results: (usize, Vec<String>), // usize is selected index

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
        let (cmd, args) = match self.search_filter {
            EnvironmentFrom::ZellijSession => ("env", Vec::<&str>::new()),
            EnvironmentFrom::DefaultShell => ("fish", vec!["-c", "env"]),
            EnvironmentFrom::LastPane => todo!(),
        };

        match self.action.as_str() {
            "run" => {
                if self.should_open_floating {
                    open_command_pane_floating(cmd, args);
                } else {
                    open_command_pane(cmd, args);
                }
            }
            _ => (),
        }
    }
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
