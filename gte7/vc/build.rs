use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use cargo_utils::paths::{LinkerPaths, h7_linker_paths, linker_path};
use cargo_utils::pull_deps::pull_h7_dependencies;

static _SHARED_COMPILER_FLAGS: [&str; 1] = [""];

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

fn build_stm_lib(hal_path: &PathBuf, hal_srcs: Vec<&str>, src_path: &PathBuf) {
    // modify main.c
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let modified_main_path = out_dir.join("modified_main.c");
    let mut main_c_content =
        fs::read_to_string("./src/cubemx/Src/main.c").expect("Failed to read main.c");
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
        PathBuf::from("../../third_party/freertos_config"),
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

    builder.compile("gte7_VC_stm32cube");

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

    let wrapper_header = PathBuf::from("./src/wrapper.h")
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

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from("./src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo::rustc-link-arg=-Wl,--undefined=Reset_Handler"); // nothing (linkerscript??) references the startup code, so force ld to pull it out of libstm32cube.a 
}

fn main() {
    let d = pull_h7_dependencies();
    // println!("{:?}", d);

    build_stm_lib(
        &d,
        vec![
            "stm32h7xx_ll_usb.c",
            "stm32h7xx_ll_fmc.c",
            "stm32h7xx_ll_sdmmc.c",
            "stm32h7xx_ll_delayblock.c",
            "stm32h7xx_hal_adc_ex.c",
            "stm32h7xx_hal_adc.c",
            "stm32h7xx_hal.c",
            "stm32h7xx_hal_rcc.c",
            "stm32h7xx_hal_rcc_ex.c",
            "stm32h7xx_hal_pwr_ex.c",
            "stm32h7xx_hal_cortex.c",
            "stm32h7xx_hal_fmac.c",
            "stm32h7xx_hal_cordic.c",
            "stm32h7xx_hal_dma_ex.c",
            "stm32h7xx_hal_dma.c",
            "stm32h7xx_hal_exti.c",
            "stm32h7xx_hal_fdcan.c",
            "stm32h7xx_hal_gpio.c",
            "stm32h7xx_hal_i2c.c",
            "stm32h7xx_hal_i2c_ex.c",
            "stm32h7xx_hal_pcd.c",
            "stm32h7xx_hal_pcd_ex.c",
            "stm32h7xx_hal_spi.c",
            "stm32h7xx_hal_tim.c",
            "stm32h7xx_hal_tim_ex.c",
            "stm32h7xx_hal_uart.c",
            "stm32h7xx_hal_uart_ex.c",
            "stm32h7xx_ll_usb.c",
            "stm32h7xx_hal_spi.c",
            "stm32h7xx_hal_tim_ex.c",
            "stm32h7xx_hal_gpio.c",
            "stm32h7xx_hal_iwdg.c",
        ],
    );

    let linker_paths: LinkerPaths = h7_linker_paths();

    println!(
        "cargo::rustc-link-arg=-Wl,-T,{}",
        &linker_paths.app_linker_script.display()
    );
    // shared linker args
    for arg in SHARED_LINKER_ARGS {
        println!("cargo::rustc-link-arg={arg}");
    }
    println!("cargo::rustc-link-search={}", linker_path().display());
    println!("cargo::rustc-link-arg=-Wl,--fatal-warnings");
}
