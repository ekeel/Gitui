# GitUI - A Git TUI in Rust

A Terminal User Interface (TUI) for Git written in Rust, providing an intuitive way to interact with your Git repositories.

## Features

- **Multiple Views**:
  - **Files View** (default): Shows working directory status with live diff preview
  - **History View**: Displays commit history with author, date, and messages
  - **Branches View**: Lists all local branches with current branch highlighted

- **File Operations**:
  - Stage individual files or all changes
  - View diffs for modified files
  - Commit staged changes with custom messages

- **Branch Management**:
  - View all local branches
  - Checkout/switch branches
  - Current branch indicator

- **Remote Operations**:
  - Pull from remote
  - Push to remote
  - Sync (pull + push)

## Installation

### Install from crates.io

```bash
cargo install gituie
```

### Manual Installation

1. Clone this repository:

   ```bash
   git clone <repository-url>
   cd RustTest
   ```

2. Build from source:

   ```bash
   cargo build --release
   ```

3. The binary will be available at `target/release/gtek`

4. (Optional) Add to PATH or copy to a directory in your PATH:
   ```bash
   cp target/release/gtek /usr/local/bin/
   ```

## Usage

Run from any directory within a Git repository:

```bash
gtek
```

Or specify a repository path:

```bash
gtek /path/to/repo
```

If running from source:

```bash
cargo run
```

## Keyboard Shortcuts

### Global

- `1` - Switch to Files view
- `2` - Switch to History view
- `3` - Switch to Branches view
- `↑/↓` or `k/j` - Navigate up/down
- `r` - Refresh current view
- `q` - Quit application

### Files View

- `s` - Stage selected file
- `a` - Stage all files
- `d` - Discard changes selected file
- 'D' - Discard changes all files
- `c` - Open commit dialog
- `p` - Pull from remote
- `P` - Push to remote
- `S` - Sync (pull + push)
- `Enter` - Refresh diff for selected file

### Branches View

- `Enter` or `o` - Checkout selected branch

### Commit Dialog

- Type to enter commit message
- `Enter` - Commit with message
- `Esc` - Cancel commit
- `Backspace` - Delete character

## Project Structure

```
src/
├── main.rs          - Application entry point and main loop
├── app.rs           - Application state and data structures
├── git.rs           - Git operations wrapper (using git2-rs)
├── input.rs         - Keyboard input handling
├── ui.rs            - Main UI rendering and layout
├── ui_files.rs      - Files view rendering
├── ui_history.rs    - History view rendering
└── ui_branches.rs   - Branches view rendering
```

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation
- `git2` - libgit2 bindings for Git operations
- `anyhow` - Error handling
- `chrono` - Date/time formatting

## UI Layout

The application displays:

- **Header**: Shows application name, current branch, and view switcher
- **Main Content**: Dynamic content based on selected view
- **Footer**: Context-sensitive help and status messages

### Files View Layout

```
┌─────────────────────────────────────────────────┐
│ Files (40%)        │ Diff Preview (60%)        │
│ M  modified.rs     │ - old line                │
│ A  new.rs          │ + new line                │
│ ?? untracked.txt   │ ...                       │
└─────────────────────────────────────────────────┘
```

## Status Indicators

### File Status

- `A ` - Added (staged)
- `M ` - Modified (staged)
- `D ` - Deleted (staged)
- ` M` - Modified (unstaged)
- ` D` - Deleted (unstaged)
- `??` - Untracked

### Diff Colors

- Green: Added lines (+)
- Red: Removed lines (-)
- Cyan: Hunk headers (@@)
- White: Context lines

## Notes

- The application requires a Git repository to function
- Remote operations (push/pull/sync) assume an "origin" remote exists
- The application uses libgit2 for all Git operations
- Merge conflicts and complex Git operations are not yet supported

## Future Enhancements

Potential features for future versions:

- Unstage files
- Amend commits
- Stash operations
- Remote management
- Merge conflict resolution
- Search functionality
- Commit message templates
- Custom key bindings

## License

This project is available under the MIT license.
