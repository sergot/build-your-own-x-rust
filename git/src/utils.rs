use std::path::PathBuf;

pub(crate) fn find_git_dir() -> Option<PathBuf> {
    let cur_dir = std::env::current_dir().ok()?;

    for path in cur_dir.ancestors() {
        let git_dir = path.join(".git");
        if git_dir.is_dir() {
            return Some(git_dir);
        }
    }

    None
}
