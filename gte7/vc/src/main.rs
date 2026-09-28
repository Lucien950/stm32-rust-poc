#![cfg_attr(not(test), no_std)]
#![no_main]
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod stm32_bindings {
    include!("bindings.rs");
}

use stm32_bindings::cube_setup;

#[unsafe(no_mangle)]
extern "C" fn main() {
    unsafe {
        cube_setup();
    }
    todo!("handoff to freertos (noreturn)");
}

// newlib's __libc_init_array calls _init, normally provided by crti.o which -nostartfiles drops
#[unsafe(no_mangle)]
extern "C" fn _init() {}
