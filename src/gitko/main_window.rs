use crate::git;
use crate::git::{parse_file_state, FileState};
use crate::ascii_table::*;
use crate::render::{Renderer, KeyHandlers, Component, ScreenSize, Window, Position};
use crate::gitko::log_window::LogWindow;
use crate::gitko::diff_window::DiffWindow;
use crate::gitko::branch_window::BranchWindow;
use crate::gitko::command_window::CommandWindow;
use crate::gitko::prompt_window::PromptWindow;

pub struct MainWindow { }

impl MainWindow {
    fn diff_file(&mut self, window: &mut Window) -> bool {
        let line = window.get_cursor_line();
        let file_state = parse_file_state(&line);

        if !matches!(file_state, FileState::Unknown) {
            let path = line[3..].trim();

            Renderer::new(
                DiffWindow::new(path),
                ScreenSize { lines: window.height(), cols: window.width() },
                Position::default()
            ).render();
        }

        true
    }

    fn open_branch_window(&mut self, window: &mut Window) -> bool {
        Renderer::new(
            BranchWindow{},
            ScreenSize::max(),
            Position::default()
        ).render();

        self.on_start(window);

        true
    }

    fn open_log_window(&mut self, window: &mut Window) -> bool {
        Renderer::new(
            LogWindow{},
            ScreenSize::max(),
            Position::default()
        ).render();

        self.on_start(window);

        true
    }

    fn open_command_window(&mut self, window: &mut Window) -> bool {
        Renderer::new(
            CommandWindow{},
            ScreenSize { lines: 2, cols: window.width() },
            Position { x: 0, y: window.height() - 2 }
        ).render();

        self.on_start(window);

        true
    }

    fn git_checkout_file(&mut self, window: &mut Window) -> bool {
        let line = window.get_cursor_line();
        let file_state = parse_file_state(&line);

        if matches!(file_state, FileState::Modified) {
            let file = line[3..].trim();
            Renderer::new(
                PromptWindow::new(&format!("Are you sure you want to checkout file '{}'? y/n", file),
                                  || { git::checkout_file(file); },
                                  || {}),
                ScreenSize { lines: 1, cols: 0 },
                Position { x: 0, y: window.height() - 1 }
            ).render();
        }

        self.on_start(window);

        true
    }

    fn git_add_file(&mut self, window: &mut Window) -> bool {
        // TODO: add parse git status that returns file state
        // and file path?
        let line = window.get_cursor_line();
        let file_state = parse_file_state(&line);

        if !matches!(file_state, FileState::Staged) {
            git::add_file(line[3..].trim());
        }

        self.on_start(window);

        true
    }

    fn git_unstage_file(&mut self, window: &mut Window) -> bool {
        let line = window.get_cursor_line();
        let file_state = parse_file_state(&line);

        if matches!(file_state, FileState::Staged) {
            git::unstage_file(line[3..].trim());
        }

        self.on_start(window);

        true
    }
}

impl Component<MainWindow> for MainWindow {
    fn on_start(&mut self, window: &mut Window) {
         let git_status: Vec<String> = git::status();

        // TODO: lists folders instead of all files in the newly
        // added folder
        let mut added: Vec<String> = git_status
            .iter()
            .filter(|c| c.starts_with("??"))
            .cloned()
            .collect();

        let mut deleted: Vec<String> = git_status
            .iter()
            .filter(|c| c.starts_with(" D"))
            .cloned()
            .collect();

        let mut unstaged: Vec<String> = git_status
            .iter()
            .filter(|c| c.starts_with(" M") || c.starts_with("MM"))
            .cloned()
            .collect();

        let mut staged: Vec<String> = git_status
            .iter()
            .filter(|c| c.starts_with('M') || c.starts_with('A') || c.starts_with('D'))
            .cloned()
            .collect();

        let sections_count = 4;
        let lines_between = 5;

        let used_lines = added.len()
            + deleted.len()
            + unstaged.len()
            + staged.len()
            + sections_count
            + lines_between;

        let remaining_lines = window.height() - (used_lines as i32) * 2;
        let recent_commits_count = (remaining_lines - 1) as u32;

        let mut recent_commits: Vec<String> = git::log(Some(recent_commits_count));

        let mut status: Vec<String> = vec![];

        if staged.is_empty() && unstaged.is_empty() && added.is_empty() && deleted.is_empty() {
            status.push("No changes found".to_string());
        }

        if !added.is_empty() {
            status.push("Untracked files:".to_string());
            status.append(&mut added);

            status.push("".to_string());
        }

        if !deleted.is_empty() {
            status.push("Deleted files:".to_string());
            status.append(&mut deleted);

            status.push("".to_string());
        }

        if !unstaged.is_empty() {
            status.push("Modified files:".to_string());
            status.append(&mut unstaged);

            status.push("".to_string());
        }

        if !staged.is_empty() {
            status.push("Staged files:".to_string());
            status.append(&mut staged);
        }

        if !recent_commits.is_empty() {
            status.append(&mut vec!["".to_string(); lines_between]);
            status.push("Recent commits:".to_string());
            status.append(&mut recent_commits);
        }

        if status.is_empty() {
            status.push("No changes found.".to_string());
        }

        window.data = status.clone();
    }

    fn register_handlers(&self, handlers: &mut KeyHandlers<MainWindow>) {
        handlers.insert(KEY_LF, MainWindow::diff_file);
        handlers.insert(KEY_B_LOWER, MainWindow::open_branch_window);
        handlers.insert(KEY_C_LOWER, MainWindow::git_checkout_file);
        handlers.insert(KEY_L_LOWER, MainWindow::open_log_window);
        handlers.insert(KEY_T_LOWER, MainWindow::git_add_file);
        handlers.insert(KEY_U_LOWER, MainWindow::git_unstage_file);
        handlers.insert(KEY_COLON, MainWindow::open_command_window);
    }
}
