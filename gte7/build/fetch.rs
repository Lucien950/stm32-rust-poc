use std::path::PathBuf;
use std::process::Command;

/// Clones a git repository and its submodules into the OUT_DIR if it doesn't already exist.
pub fn fetch_hal_repo(url: &str, out_dir: &PathBuf, repo_name: &str) -> PathBuf {
    let repo_path = out_dir.join(repo_name);

    // Check if the repository is already cached
    if !repo_path.join(".git").exists() {
        println!(
            "cargo:warning=Downloading {} and its submodules...",
            repo_name
        );

        let status = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1") // Shallow clone of the main repo
            .arg("--recurse-submodules") // Fetch all submodules
            .arg("--shallow-submodules") // Shallow clone for submodules too
            .arg(url)
            .arg(&repo_path)
            .status()
            .expect("Failed to execute git command. Is git installed and in PATH?");

        if !status.success() {
            panic!("Failed to clone repository: {}", url);
        }
    } else {
        println!("cargo:warning=Using cached {}", repo_name);

        // Safety check: ensure submodules are initialized and updated
        // just in case a previous clone was interrupted or didn't fetch them.
        let _ = Command::new("git")
            .current_dir(&repo_path)
            .arg("submodule")
            .arg("update")
            .arg("--init")
            .arg("--recursive")
            .status();
    }

    repo_path
}
