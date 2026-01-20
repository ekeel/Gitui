use anyhow::{Context, Result};
use git2::{Branch, BranchType, Commit, Delta, DiffOptions, Repository, Status, StatusOptions};
use std::path::Path;

use crate::app::{BranchInfo, CommitInfo, FileStatus};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(Self { repo })
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    }

    pub fn get_branches(&self) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();
        let current_branch = self.get_current_branch().unwrap_or_default();

        for branch_result in self.repo.branches(Some(BranchType::Local))? {
            if let Ok((branch, _)) = branch_result {
                if let Some(name) = branch.name()? {
                    branches.push(BranchInfo {
                        name: name.to_string(),
                        is_current: name == current_branch,
                    });
                }
            }
        }

        Ok(branches)
    }

    pub fn get_commits(&self, limit: usize) -> Result<Vec<CommitInfo>> {
        let mut commits = Vec::new();
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        for oid in revwalk.take(limit) {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            let author = commit.author();
            let date = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();

            commits.push(CommitInfo {
                id: format!("{:.7}", oid),
                author: author.name().unwrap_or("Unknown").to_string(),
                date,
                message: commit
                    .message()
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string(),
            });
        }

        Ok(commits)
    }

    pub fn get_status(&self) -> Result<Vec<FileStatus>> {
        let mut files = Vec::new();
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);

        let statuses = self.repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = match entry.status() {
                s if s.contains(Status::INDEX_NEW) => "A ",
                s if s.contains(Status::INDEX_MODIFIED) => "M ",
                s if s.contains(Status::INDEX_DELETED) => "D ",
                s if s.contains(Status::WT_NEW) => "??",
                s if s.contains(Status::WT_MODIFIED) => " M",
                s if s.contains(Status::WT_DELETED) => " D",
                _ => "  ",
            };

            files.push(FileStatus {
                path,
                status: status.to_string(),
            });
        }

        Ok(files)
    }

    pub fn get_diff_for_file(&self, path: &str) -> Result<String> {
        let mut diff_text = String::new();

        // Check if file is untracked
        let file_path = Path::new(path);
        if file_path.exists() {
            let mut opts = StatusOptions::new();
            opts.pathspec(path);
            let statuses = self.repo.statuses(Some(&mut opts))?;

            if let Some(entry) = statuses.get(0) {
                if entry.status().contains(Status::WT_NEW) {
                    // For untracked files, show the content as all new lines
                    if let Ok(content) = std::fs::read_to_string(file_path) {
                        diff_text.push_str(&format!("New file: {}\n", path));
                        diff_text.push_str("--- /dev/null\n");
                        diff_text.push_str(&format!("+++ {}\n", path));
                        for line in content.lines() {
                            diff_text.push('+');
                            diff_text.push_str(line);
                            diff_text.push('\n');
                        }
                        return Ok(diff_text);
                    }
                }
            }
        }

        // Get the diff for the working directory changes
        let mut opts = DiffOptions::new();
        opts.pathspec(path);
        opts.include_untracked(true);

        let diff = self.repo.diff_index_to_workdir(None, Some(&mut opts))?;

        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let origin = line.origin();
            let content = std::str::from_utf8(line.content()).unwrap_or("");

            match origin {
                '+' | '-' | ' ' => {
                    diff_text.push(origin);
                    diff_text.push_str(content);
                }
                _ => {
                    diff_text.push_str(content);
                }
            }
            true
        })?;

        if diff_text.is_empty() {
            // Try staged changes
            if let Ok(head) = self.repo.head() {
                if let Ok(tree) = head.peel_to_tree() {
                    let diff = self
                        .repo
                        .diff_tree_to_index(Some(&tree), None, Some(&mut opts))?;

                    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                        let origin = line.origin();
                        let content = std::str::from_utf8(line.content()).unwrap_or("");

                        match origin {
                            '+' | '-' | ' ' => {
                                diff_text.push(origin);
                                diff_text.push_str(content);
                            }
                            _ => {
                                diff_text.push_str(content);
                            }
                        }
                        true
                    })?;
                }
            }
        }

        if diff_text.is_empty() {
            diff_text = format!("No changes to display for: {}", path);
        }

        Ok(diff_text)
    }

    pub fn stage_file(&self, path: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(Path::new(path))?;
        index.write()?;
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        let oid = index.write_tree()?;
        let signature = self.repo.signature()?;
        let tree = self.repo.find_tree(oid)?;

        let parent_commit = self.repo.head()?.peel_to_commit()?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }

    pub fn create_branch(&self, branch_name: &str, base_branch: &str) -> Result<()> {
        let base_commit = self.repo.revparse_single(base_branch)?.peel_to_commit()?;
        self.repo.branch(branch_name, &base_commit, false)?;
        Ok(())
    }

    pub fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        let (object, reference) = self.repo.revparse_ext(branch_name)?;

        self.repo.checkout_tree(&object, None)?;

        match reference {
            Some(gref) => self.repo.set_head(gref.name().unwrap())?,
            None => self.repo.set_head_detached(object.id())?,
        }

        Ok(())
    }

    pub fn pull(&self) -> Result<()> {
        // Simplified pull - fetch and fast-forward merge
        let mut remote = self.repo.find_remote("origin")?;
        let config = self.repo.config()?;

        // Set up authentication callbacks
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |url, username_from_url, allowed_types| {
            // For HTTPS, try credential helper
            if url.starts_with("https://") {
                if let Ok(cred) = git2::Cred::credential_helper(&config, url, username_from_url) {
                    return Ok(cred);
                }
            }
            
            // For SSH, try SSH agent
            if url.starts_with("git@") || url.starts_with("ssh://") {
                if let Ok(cred) = git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                    return Ok(cred);
                }
            }

            // Try username/password if allowed
            if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                if let Ok(cred) = git2::Cred::credential_helper(&config, url, username_from_url) {
                    return Ok(cred);
                }
            }

            // Try SSH key if allowed
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                if let Ok(cred) = git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                    return Ok(cred);
                }
            }

            // Try default
            if allowed_types.contains(git2::CredentialType::DEFAULT) {
                if let Ok(cred) = git2::Cred::default() {
                    return Ok(cred);
                }
            }

            Err(git2::Error::from_str("No valid credentials found"))
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&["HEAD"], Some(&mut fetch_options), None)?;

        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;

        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;

        if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.get_current_branch()?);
            let mut reference = self.repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            self.repo.set_head(&refname)?;
            self.repo
                .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        }

        Ok(())
    }

    pub fn push(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let branch = self.get_current_branch()?;
        let refspec = format!("refs/heads/{}", branch);

        // Set up authentication callbacks
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            // For HTTPS, use git credential fill
            if url.starts_with("https://") && allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                // Call git credential fill
                use std::io::Write;
                use std::process::{Command, Stdio};
                
                let mut child = Command::new("git")
                    .arg("credential")
                    .arg("fill")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .map_err(|e| git2::Error::from_str(&format!("Failed to spawn git credential: {}", e)))?;
                
                // Write the credential request
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = writeln!(stdin, "protocol=https");
                    let _ = writeln!(stdin, "host=github.com");
                    if let Some(username) = username_from_url {
                        let _ = writeln!(stdin, "username={}", username);
                    }
                    let _ = writeln!(stdin);
                }
                
                let output = child.wait_with_output()
                    .map_err(|e| git2::Error::from_str(&format!("Failed to get git credential output: {}", e)))?;
                
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut username = String::new();
                    let mut password = String::new();
                    
                    for line in stdout.lines() {
                        if let Some(user) = line.strip_prefix("username=") {
                            username = user.to_string();
                        } else if let Some(pass) = line.strip_prefix("password=") {
                            password = pass.to_string();
                        }
                    }
                    
                    if !username.is_empty() && !password.is_empty() {
                        return git2::Cred::userpass_plaintext(&username, &password);
                    }
                }
            }
            
            // For SSH
            if url.starts_with("git@") || url.starts_with("ssh://") || allowed_types.contains(git2::CredentialType::SSH_KEY) {
                if let Ok(cred) = git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                    return Ok(cred);
                }
            }

            // Try default
            if allowed_types.contains(git2::CredentialType::DEFAULT) {
                return git2::Cred::default();
            }

            Err(git2::Error::from_str("No valid credentials found"))
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        remote.push(&[&refspec], Some(&mut push_options))?;
        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        self.pull()?;
        self.push()?;
        Ok(())
    }
}
