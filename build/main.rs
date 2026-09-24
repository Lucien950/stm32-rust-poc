use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
mod fetch;

fn main() {
    let hal_repo_url = "https://github.com/UBCFormulaElectric/STM32CubeH7.git";
    let out_dir = PathBuf::from("deps");
    let hal_path = fetch::fetch_hal_repo(hal_repo_url, &out_dir, "STM32CubeH7");

    // modify main.c
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let original_main_path = "src/cubemx/Src/main.c";
    let modified_main_path = out_dir.join("modified_main.c");
    let mut main_c_content = fs::read_to_string(original_main_path).expect("Failed to read main.c");
    main_c_content = main_c_content.replace("int main(void)", "void cube_setup(void)");
    main_c_content = main_c_content.replace("while (1)", "while (0)");
    fs::write(&modified_main_path, main_c_content).expect("Failed to write modified_main.c");

    // include paths
    // let hal_src_path = hal_path.join("Drivers/STM32H7xx_HAL_Driver/Src");
    #[allow(non_snake_case)]
    let STM32HAL_INCLUDES = [
        hal_path.join("Drivers/STM32H7xx_HAL_Driver/Inc"),
        hal_path.join("Middlewares/Third_Party/FreeRTOS/Source/include"),
        hal_path.join("Middlewares/Third_Party/FreeRTOS/Source/CMSIS_RTOS_V2"),
        hal_path.join("Middlewares/Third_Party/FreeRTOS/Source/portable/GCC/ARM_CM7/r0p1"),
        hal_path.join("Drivers/CMSIS/Device/ST/STM32H7xx/Include"),
        hal_path.join("Drivers/CMSIS/Include"),
        PathBuf::from("src/cubemx/Inc"),
        PathBuf::from("shared/freertos_config"),
    ];
    #[allow(non_snake_case)]
    let STM32HAL_DEFINES = [("USE_HAL_DRIVER", None), ("STM32H733xx", None)];

    // build C library from stm32cubemx library (this should be a shared function)
    let mut builder = cc::Build::new();
    for (a, b) in &STM32HAL_DEFINES {
        builder.define(*a, b.clone());
    }
    for include_path in &STM32HAL_INCLUDES {
        builder.include(include_path);
    }
    builder.file(modified_main_path);
    let mut add_c_files_from_dir = |dir: &str, exclude_file: Option<&str>| {
        let path = Path::new(dir);
        if path.is_dir() {
            for entry in fs::read_dir(path).expect("Failed to read directory") {
                let entry = entry.expect("Failed to read entry");
                let file_path = entry.path();

                // Check if it's a file and has a .c extension
                if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "c") {
                    let file_name = file_path.file_name().unwrap().to_str().unwrap();

                    // Skip the excluded file (e.g., the original main.c)
                    if let Some(exclude) = exclude_file {
                        if file_name == exclude {
                            continue;
                        }
                    }

                    // Add the file to the build
                    builder.file(file_path);
                }
            }
        }
    };
    add_c_files_from_dir("src/cubemx/Src", Some("main.c"));
    builder.compiler("arm-none-eabi-gcc");
    builder.compile("stm32cube");

    // bindgen uses libclang, which doesn't know where the ARM toolchain's newlib headers
    // (math.h, stdint.h, ...) live, so borrow the sysroot from arm-none-eabi-gcc
    let sysroot_output = std::process::Command::new("arm-none-eabi-gcc")
        .arg("-print-sysroot")
        .output()
        .expect("Failed to run arm-none-eabi-gcc -print-sysroot");
    let sysroot = String::from_utf8(sysroot_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let mut bindings_builder = bindgen::Builder::default()
        .header("src/wrapper.h")
        .use_core() // Important for #![no_std] environments
        .clang_arg("--target=thumbv7em-none-eabihf")
        .clang_arg("-mcpu=cortex-m7")
        .clang_arg("-mfloat-abi=hard")
        .clang_arg(format!("--sysroot={sysroot}"))
        .clang_arg(format!("-isystem{sysroot}/include"));
    for (name, value) in &STM32HAL_DEFINES {
        bindings_builder = match value {
            Some(v) => bindings_builder.clang_arg(format!("-D{name}={v}")),
            None => bindings_builder.clang_arg(format!("-D{name}")),
        };
    }
    for include_path in &STM32HAL_INCLUDES {
        bindings_builder = bindings_builder.clang_arg(format!("-I{}", include_path.display()));
    }

    let bindings = bindings_builder
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
