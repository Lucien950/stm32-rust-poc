use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
mod fetch;

static SHARED_COMPILER_FLAGS: [&str; 1] = [""];

static SHARED_LINKER_ARGS: [&str; 11] = [
    "-Wl,-gc-sections,--print-memory-usage",
    // match the Rust target's hard-float ABI so gcc picks the right multilib
    "-mcpu=cortex-m7",
    "-mthumb",
    "-mfpu=fpv5-d16",
    "-mfloat-abi=hard",
    // don't pull in gcc's crt0.o (it's what references __libc_init_array)
    "-nostartfiles",
    "--specs=nano.specs",
    // libraries
    "-lc",
    "-lm",
    "-lgcc",
    // aura
    "-static",
];

fn build_stm_lib(hal_path: &PathBuf, hal_srcs: Vec<&str>) {
    // modify main.c
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let original_main_path = "src/cubemx/Src/main.c";
    let modified_main_path = out_dir.join("modified_main.c");
    let mut main_c_content = fs::read_to_string(original_main_path).expect("Failed to read main.c");
    main_c_content = main_c_content.replace("int main(void)", "void cube_setup(void)");
    main_c_content = main_c_content.replace("while (1)", "while (0)");
    fs::write(&modified_main_path, main_c_content).expect("Failed to write modified_main.c");

    let stm32_hal_includes: [PathBuf; 8] = [
        hal_path.join("Drivers/STM32H7xx_HAL_Driver/Inc"),
        hal_path.join("Middlewares/Third_Party/FreeRTOS/Source/include"),
        hal_path.join("Middlewares/Third_Party/FreeRTOS/Source/CMSIS_RTOS_V2"),
        hal_path.join("Middlewares/Third_Party/FreeRTOS/Source/portable/GCC/ARM_CM7/r0p1"),
        hal_path.join("Drivers/CMSIS/Device/ST/STM32H7xx/Include"),
        hal_path.join("Drivers/CMSIS/Include"),
        PathBuf::from("src/cubemx/Inc"),
        PathBuf::from("shared/freertos_config"),
    ];
    let stm32_hal_defines: [(&str, Option<&str>); 2] =
        [("USE_HAL_DRIVER", None), ("STM32H733xx", None)];

    // build C library from stm32cubemx library (this should be a shared function)
    let mut builder = cc::Build::new();
    builder.compiler("arm-none-eabi-gcc");
    for (a, b) in &stm32_hal_defines {
        // defines
        builder.define(*a, *b);
    }
    for include_path in &stm32_hal_includes {
        // include directories
        builder.include(include_path);
    }

    builder.file(modified_main_path);
    let path = Path::new("src/cubemx/Src");
    assert!(path.is_dir());
    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let file_path = entry.path();
        if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "c") {
            let file_name = file_path.file_name().unwrap().to_str().unwrap();
            if file_name == "main.c" {
                continue;
            }
            builder.file(file_path);
        }
    }

    let hal_src_path = hal_path.join("Drivers/STM32H7xx_HAL_Driver/Src");
    for src in hal_srcs {
        builder.file(hal_src_path.join(src));
    }

    // include the startup script
    builder.file(
        hal_path
            .join("Drivers/CMSIS/Device/ST/STM32H7xx/Source/Templates/gcc/startup_stm32h733xx.s"),
    );

    builder.compile("gte7_VC_stm32cube");
}

fn build_vc(
    h7_hal_path: &PathBuf,
    h7_app_linker_script: &PathBuf,
    _h7_boot_linker_script: &PathBuf,
) {
    // build VC
    build_stm_lib(
        h7_hal_path,
        vec![
            "stm32h7xx_ll_usb.c",
            "stm32h7xx_ll_fmc.c",
            "stm32h7xx_ll_sdmmc.c",
            "stm32h7xx_ll_delayblock.c",
        ],
    );

    const VC: &str = "VC";
    println!(
        "cargo::rustc-link-arg-bin={VC}=-Wl,-T,{}",
        h7_app_linker_script.to_string_lossy().into_owned()
    );
    println!("cargo::rustc-link-arg-bin={VC}=-Wl,--undefined=Reset_Handler"); // nothing (linkerscript??) references the startup code, so force ld to pull it out of libstm32cube.a 
    // shared linker args
    for arg in SHARED_LINKER_ARGS {
        println!("cargo::rustc-link-arg-bin={VC}={arg}");
    }
}

fn build_gte7(
    _f4_hal_path: &PathBuf,
    h7_hal_path: &PathBuf,
    _h7_app_only_linker_script: &PathBuf,
    h7_app_linker_script: &PathBuf,
    h7_boot_linker_script: &PathBuf,
) {
    build_vc(h7_hal_path, h7_app_linker_script, h7_boot_linker_script);
}

fn main() {
    // pull dependencies
    let deps_dir: PathBuf = PathBuf::from("deps");

    const H7_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeH7.git";
    let h7_hal_path: PathBuf = fetch::fetch_hal_repo(H7_HAL_REPO_URL, &deps_dir, "STM32CubeH7");
    const F4_HAL_REPO_URL: &str = "https://github.com/UBCFormulaElectric/STM32CubeF4.git";
    let f4_hal_path: PathBuf = fetch::fetch_hal_repo(F4_HAL_REPO_URL, &deps_dir, "STM32CubeF4");

    let linker_dir =
        fs::canonicalize(PathBuf::from("./linker")).expect("Linker dir should be present");
    // linker scripts INCLUDE each other relative to the linker/ directory
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    println!(
        "cargo::rustc-link-search={}",
        linker_dir.clone().into_string().unwrap()
    );

    let h7_app_only_linker_script =
        fs::canonicalize(linker_dir.join("stm32h733vgtx/stm32h733vgtx_app_only.ld"))
            .expect("H7 app only linker script should be present");
    println!(
        "cargo::rerun-if-changed={}",
        h7_app_only_linker_script.to_string_lossy().into_owned()
    );
    let h7_app_linker_script =
        fs::canonicalize(linker_dir.join("./linker/stm32h733vgtx/stm32h733vgtx_app.ld"))
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

    build_gte7(
        &f4_hal_path,
        &h7_hal_path,
        &h7_app_only_linker_script,
        &h7_app_linker_script,
        &h7_boot_linker_script,
    );
}
