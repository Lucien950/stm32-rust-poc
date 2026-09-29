use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::paths::deps_path;

/// Clones a git repository and its submodules into the OUT_DIR if it doesn't already exist.
fn fetch_hal_repo(url: &str, out_dir: &PathBuf, repo_name: &str) -> PathBuf {
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
    }

    fs::canonicalize(repo_path).unwrap()
}

pub fn pull_f4_dependencies() -> PathBuf {
    const F4_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeF4.git";
    fetch_hal_repo(F4_HAL_REPO_URL, &deps_path(), "STM32CubeF4")
}

pub fn pull_h7_dependencies() -> PathBuf {
    const H7_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeH7.git";
    fetch_hal_repo(H7_HAL_REPO_URL, &deps_path(), "STM32CubeH7")

    // linker scripts INCLUDE each other relative to the linker/ directory
}
