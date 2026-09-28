use std::{fs, path::PathBuf};

pub fn deps_path() -> PathBuf {
    PathBuf::from("../../deps")
}

pub fn linker_path() -> PathBuf {
    fs::canonicalize(PathBuf::from("../../linker"))
        .expect("Linker dir should be present in Git Repo")
}

pub struct LinkerPaths {
    pub app_only_linker_script: PathBuf,
    pub app_linker_script: PathBuf,
    pub boot_linker_script: PathBuf,
}

pub fn f4_linker_paths() -> LinkerPaths {
    let linker_dir = linker_path();
    let f4_app_only_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_app_only.ld"))
            .expect("H7 app only linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        f4_app_only_linker_script.to_string_lossy().into_owned()
    );
    let f4_app_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_app.ld"))
            .expect("H7 app linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        f4_app_linker_script.to_string_lossy().into_owned()
    );
    let f4_boot_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_boot.ld"))
            .expect("H7 boot linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        f4_boot_linker_script.to_string_lossy().into_owned()
    );
    LinkerPaths {
        app_only_linker_script: f4_app_only_linker_script,
        app_linker_script: f4_app_linker_script,
        boot_linker_script: f4_boot_linker_script,
    }
}

pub fn h7_linker_paths() -> LinkerPaths {
    let linker_dir = linker_path();
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

    LinkerPaths {
        app_only_linker_script: h7_app_only_linker_script,
        app_linker_script: h7_app_linker_script,
        boot_linker_script: h7_boot_linker_script,
    }
}
