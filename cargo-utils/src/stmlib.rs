use std::{env, fs, path::PathBuf};

pub fn build_stmh7_lib(
    hal_path: &PathBuf,
    hal_srcs: Vec<&str>,
    src_path: &PathBuf,
    bin_name: String,
) {
    // modify main.c
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // per-binary dir: cc names objects <hash of source dir>-<stem>.o, so shared sources (HAL,
    // startup, modified_main.c) would otherwise overwrite each other across binaries
    let obj_dir = out_dir.join(&bin_name);
    fs::create_dir_all(&obj_dir).expect("Failed to create object dir");
    let modified_main_path = obj_dir.join("modified_main.c");
    let mut main_c_content =
        fs::read_to_string(src_path.join("cubemx/Src/main.c")).expect("Failed to read main.c");
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
        src_path.join("cubemx/Inc"),
        PathBuf::from("../third_party/freertos_config"),
    ];
    let stm32_hal_defines: [(&str, Option<&str>); 2] =
        [("USE_HAL_DRIVER", None), ("STM32H733xx", None)];

    // build C library from stm32cubemx library (this should be a shared function)
    let mut builder = cc::Build::new();
    builder.compiler("arm-none-eabi-gcc").out_dir(&obj_dir);
    for (a, b) in &stm32_hal_defines {
        // defines
        builder.define(*a, *b);
    }
    for include_path in &stm32_hal_includes {
        // include directories
        builder.include(include_path);
    }

    builder.file(modified_main_path);
    let cubemx_src_path = src_path.join("cubemx/Src");
    assert!(cubemx_src_path.is_dir());
    for entry in fs::read_dir(cubemx_src_path).expect("Failed to read directory") {
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

    // VERY IMPORTANT, MAKE SURE YOU ARE LINKING ONLY TO THE BINARIES YOU CARE ABOUT
    let objs = builder.flag("-ffunction-sections").compile_intermediates(); // Vec<PathBuf>, no archive, no cargo metadata
    for obj in &objs {
        println!("cargo:rustc-link-arg-bin={}={}", bin_name, obj.display());
        println!("cargo:rerun-if-changed={}", obj.display());
    }

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

    let wrapper_header = PathBuf::from(src_path.join("wrapper.h"))
        .into_os_string()
        .into_string()
        .unwrap();
    let mut bindings_builder = bindgen::Builder::default()
        .header(&wrapper_header)
        .use_core() // Important for #![no_std] environments
        .rust_edition(bindgen::RustEdition::Edition2024) // emit `unsafe extern` blocks
        .clang_arg("--target=thumbv7em-none-eabihf")
        .clang_arg("-mcpu=cortex-m7")
        .clang_arg("-mfloat-abi=hard")
        .clang_arg(format!("--sysroot={sysroot}"))
        .clang_arg(format!("-isystem{sysroot}/include"));
    println!("cargo::rerun-if-changed={}", &wrapper_header);
    for (name, value) in &stm32_hal_defines {
        bindings_builder = match value {
            Some(v) => bindings_builder.clang_arg(format!("-D{name}={v}")),
            None => bindings_builder.clang_arg(format!("-D{name}")),
        };
    }

    for include_path in &stm32_hal_includes {
        bindings_builder = bindings_builder.clang_arg(format!("-I{}", include_path.display()));
    }

    let bindings = bindings_builder
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to $OUT_DIR/<bin_name>_bindings.rs, include with
    // include!(concat!(env!("OUT_DIR"), "/<bin_name>_bindings.rs"))
    bindings
        .write_to_file(out_dir.join(format!("{bin_name}_bindings.rs")))
        .expect("Couldn't write bindings!");

    println!("cargo::rustc-link-arg-bin={bin_name}=-Wl,--undefined=Reset_Handler"); // nothing (linkerscript??) references the startup code, so force ld to pull it out of libstm32cube.a 
}
