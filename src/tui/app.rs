use crate::themes::Theme;
use crate::{config::Settings, types::group_scripts};
use crate::types::{Project, Script};
use anyhow::Context;
use ratatui::widgets::ListState;

pub struct App<'a> {
    pub project: &'a Project,
    pub projects: &'a Vec<&'a Project>,
    pub theme: Theme,
    pub scripts: Vec<Script>,
    pub visible_script_indices: Vec<usize>,
    pub selected_project_state: ListState,
    pub selected_script_state: ListState,
    pub show_emoji: bool,
    pub visual_to_script_index: Vec<Option<usize>>,
}

impl<'a> App<'a> {
    pub fn new(
        project: &'a Project,
        projects: &'a Vec<&'a Project>,
        theme: Theme,
        settings: &Settings,
    ) -> anyhow::Result<Self> {
        let scripts = project.scripts()?;
        let filtered_indices: Vec<usize> = (0..scripts.len()).collect();

        let mut app = Self {
            project,
            projects,
            theme,
            scripts,
            selected_script_state: ListState::default(),
            visible_script_indices: filtered_indices,
            selected_project_state: ListState::default(),
            show_emoji: settings.show_emoji,
            visual_to_script_index: Vec::new(),
        };

        app.selected_script_state.select(Some(0));
        if !app.projects.is_empty() {
            app.selected_project_state.select(Some(0));
        }
        Ok(app)
    }

    pub fn next_script(&mut self) {
        let len = self.scripts.len();
        if len == 0 {
            return;
        }
        let i = self.selected_script_state.selected().map_or(0, |i| {
            let next = (i + 1) % len;
            // Skip dividers
            // while next != i && self.visual_to_script_index[next].is_none() {
            //     next = (next + 1) % len;
            // }
            next
        });
        self.selected_script_state.select(Some(i));
    }

    pub fn previous_script(&mut self) {
        let len = self.scripts.len();
        if len == 0 {
            return;
        }
        let i = self.selected_script_state.selected().map_or(0, |i| {
            let prev = if i == 0 {
                len - 1
            } else {
                i - 1
            };
            // Skip dividers
            // while prev != i && self.visual_to_script_index[prev].is_none() {
            //     prev = if prev == 0 {
            //         len - 1
            //     } else {
            //         prev - 1
            //     };
            // }
            prev
        });
        self.selected_script_state.select(Some(i));
    }

    pub fn get_selected_script(&self) -> Option<&Script> {
        self.selected_script_state
            .selected()
            .and_then(|i| self.visual_to_script_index.get(i))
            .and_then(|opt| opt.as_ref())
            .map(|&script_idx| &self.scripts[script_idx])
    }

    pub fn next_project(&mut self) {
        let len = self.projects.len();
        if len == 0 {
            return;
        }
        let i = match self.selected_project_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.select_project_by_index(i);
    }

    pub fn previous_project(&mut self) {
        let len = self.projects.len();
        if len == 0 {
            return;
        }
        let i = match self.selected_project_state.selected() {
            Some(i) => (i + len - 1) % len,
            None => 0,
        };
        self.select_project_by_index(i);
    }

    pub fn select_project_by_index(&mut self, i: usize) {
        self.project = &self.projects[i];
        self.selected_project_state.select(Some(i));
        self.update_scripts();
    }

    #[allow(dead_code)]
    pub fn select_project(&'a mut self, project: &'a Project) {
        let i = self
            .projects
            .iter()
            .position(|p| p.path == project.path)
            .unwrap();
        self.selected_project_state.select(Some(i));
        self.project = project;
        self.update_scripts();
    }

    fn update_scripts(&mut self) {
        self.scripts = self
            .project
            .scripts()
            .context("error getting scripts")
            .unwrap();
        self.visible_script_indices = (0..self.scripts.len()).collect();
        self.selected_script_state.select(Some(0));
    }

    pub fn group_scripts(&self) -> Vec<Vec<&Script>> {
        group_scripts(&self.scripts)
    }

    pub fn is_project_in_current_dir(&self, name: &str) -> bool {
        name == "Current Directory"
    }
}
