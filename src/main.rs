#![cfg_attr(not(test), no_std)]
#![no_main]

// use cortex_m_rt::entry;
// use panic_halt as _;
// use stm32h7xx_hal::{pac, prelude::*};
// #[entry]

use core::panic::PanicInfo;

// Your custom panic handler for a bare-metal target
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // 1. Log to your hardware interface here (e.g., UART, serial console)

    // 2. Loop infinitely or trigger a reset so the program never returns
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn main() {
    todo!("handoff to freertos (noreturn)")
}

#[unsafe(no_mangle)]
extern "C" fn exit() {
    todo!("handoff to freertos (noreturn)")
}
