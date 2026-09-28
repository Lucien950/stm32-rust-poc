use std::path::PathBuf;

use crate::{fetch, paths::deps_path};

pub fn pull_f4_dependencies() -> PathBuf {
    const F4_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeF4.git";
    fetch::fetch_hal_repo(F4_HAL_REPO_URL, &deps_path(), "STM32CubeF4")
}

pub fn pull_h7_dependencies() -> PathBuf {
    const H7_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeH7.git";
    fetch::fetch_hal_repo(H7_HAL_REPO_URL, &deps_path(), "STM32CubeH7")

    // linker scripts INCLUDE each other relative to the linker/ directory
}
