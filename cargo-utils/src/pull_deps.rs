use std::{fs, path::PathBuf};

use crate::fetch;

#[derive(Debug)]
pub struct DepsInfo {
    pub h7_hal_path: PathBuf,
    pub f4_hal_path: PathBuf,
    pub linker_dir: PathBuf,
    pub h7_app_only_linker_script: PathBuf,
    pub h7_app_linker_script: PathBuf,
    pub h7_boot_linker_script: PathBuf,
}

pub fn pull_dependencies() -> DepsInfo {
    // pull dependencies
    let deps_dir: PathBuf = PathBuf::from("../../deps");

    const H7_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeH7.git";
    let h7_hal_path: PathBuf = fetch::fetch_hal_repo(H7_HAL_REPO_URL, &deps_dir, "STM32CubeH7");
    const F4_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeF4.git";
    let f4_hal_path: PathBuf = fetch::fetch_hal_repo(F4_HAL_REPO_URL, &deps_dir, "STM32CubeF4");

    let linker_dir =
        fs::canonicalize(PathBuf::from("../../linker")).expect("Linker dir should be present");
    // linker scripts INCLUDE each other relative to the linker/ directory
    println!("cargo::rustc-link-search={}", linker_dir.display());
    println!("cargo::rustc-link-arg=-Wl,--fatal-warnings");

    let h7_app_only_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_app_only.ld"))
            .expect("H7 app only linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        h7_app_only_linker_script.to_string_lossy().into_owned()
    );
    let h7_app_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_app.ld"))
            .expect("H7 app linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        h7_app_linker_script.to_string_lossy().into_owned()
    );
    let h7_boot_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_boot.ld"))
            .expect("H7 boot linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        h7_boot_linker_script.to_string_lossy().into_owned()
    );
    DepsInfo {
        h7_hal_path,
        f4_hal_path,
        linker_dir,
        h7_app_only_linker_script,
        h7_app_linker_script,
        h7_boot_linker_script,
    }
}
