use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    History,
    Files,
    Branches,
}

#[derive(Debug)]
pub struct App {
    pub current_view: View,
    pub repo_path: PathBuf,
    pub should_quit: bool,
    pub history_state: HistoryState,
    pub files_state: FilesState,
    pub branches_state: BranchesState,
    pub status_message: Option<String>,
    pub show_commit_dialog: bool,
    pub commit_message: String,
    pub show_branch_dialog: bool,
    pub branch_creation: BranchCreation,
}

#[derive(Debug)]
pub struct HistoryState {
    pub selected: usize,
    pub commits: Vec<CommitInfo>,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug)]
pub struct FilesState {
    pub selected: usize,
    pub files: Vec<FileStatus>,
    pub current_diff: Option<String>,
    pub diff_scroll: usize,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Debug)]
pub struct BranchesState {
    pub selected: usize,
    pub branches: Vec<BranchInfo>,
    pub current_branch: String,
}

#[derive(Debug)]
pub struct BranchCreation {
    pub new_branch_name: String,
    pub base_branch_selected: usize,
    pub selecting_base: bool,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
}

impl App {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            current_view: View::Files,
            repo_path,
            should_quit: false,
            show_branch_dialog: false,
            branch_creation: BranchCreation {
                new_branch_name: String::new(),
                base_branch_selected: 0,
                selecting_base: false,
            },
            history_state: HistoryState {
                selected: 0,
                commits: Vec::new(),
            },
            files_state: FilesState {
                selected: 0,
                files: Vec::new(),
                current_diff: None,
                diff_scroll: 0,
            },
            branches_state: BranchesState {
                selected: 0,
                branches: Vec::new(),
                current_branch: String::new(),
            },
            status_message: None,
            show_commit_dialog: false,
            commit_message: String::new(),
        }
    }

    pub fn switch_view(&mut self, view: View) {
        self.current_view = view;
    }

    pub fn next_item(&mut self) {
        match self.current_view {
            View::History => {
                if !self.history_state.commits.is_empty() {
                    self.history_state.selected =
                        (self.history_state.selected + 1).min(self.history_state.commits.len() - 1);
                }
            }
            View::Files => {
                if !self.files_state.files.is_empty() {
                    self.files_state.selected =
                        (self.files_state.selected + 1).min(self.files_state.files.len() - 1);
                }
            }
            View::Branches => {
                if !self.branches_state.branches.is_empty() {
                    self.branches_state.selected = (self.branches_state.selected + 1)
                        .min(self.branches_state.branches.len() - 1);
                }
            }
        }
    }

    pub fn previous_item(&mut self) {
        match self.current_view {
            View::History => {
                if self.history_state.selected > 0 {
                    self.history_state.selected -= 1;
                }
            }
            View::Files => {
                if self.files_state.selected > 0 {
                    self.files_state.selected -= 1;
                }
            }
            View::Branches => {
                if self.branches_state.selected > 0 {
                    self.branches_state.selected -= 1;
                }
            }
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn scroll_diff_up(&mut self) {
        if self.files_state.diff_scroll > 0 {
            self.files_state.diff_scroll -= 1;
        }
    }

    pub fn scroll_diff_down(&mut self) {
        self.files_state.diff_scroll += 1;
    }

    pub fn reset_diff_scroll(&mut self) {
        self.files_state.diff_scroll = 0;
    }
}
