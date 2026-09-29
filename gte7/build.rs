use std::path::PathBuf;

use cargo_utils::embedded::configure_shared_linker_flags;
use cargo_utils::paths::{LinkerPaths, h7_linker_paths};
use cargo_utils::pull_deps::pull_h7_dependencies;
use cargo_utils::stmlib::build_stmh7_lib;

fn main() {
    let d = pull_h7_dependencies();
    // println!("{:?}", d);

    build_stmh7_lib(
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
            "stm32h7xx_hal_iwdg.c",
        ],
        &PathBuf::from("./vc/src"),
        "gte7_VC_app".to_string(),
    );

    build_stmh7_lib(
        &d,
        vec![
            "stm32h7xx_hal.c",
            "stm32h7xx_hal_dma_ex.c",
            "stm32h7xx_hal_dma.c",
            "stm32h7xx_hal_fdcan.c",
            "stm32h7xx_hal_gpio.c",
            "stm32h7xx_hal_cortex.c",
            "stm32h7xx_hal_pwr_ex.c",
            "stm32h7xx_hal_rcc.c",
            "stm32h7xx_hal_rcc_ex.c",
            "stm32h7xx_hal_tim.c",
            "stm32h7xx_hal_tim_ex.c",
        ],
        &PathBuf::from("./vc/boot"),
        "gte7_VC_boot".to_string(),
    );

    let linker_paths: LinkerPaths = h7_linker_paths();

    println!(
        "cargo::rustc-link-arg-bin=gte7_VC_app=-Wl,-T,{}",
        &linker_paths.app_linker_script.display()
    );
    println!(
        "cargo::rustc-link-arg-bin=gte7_VC_boot=-Wl,-T,{}",
        &linker_paths.boot_linker_script.display()
    );

    configure_shared_linker_flags();
}
