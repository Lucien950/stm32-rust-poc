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
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use stm32_bindings::cube_setup;

#[unsafe(no_mangle)]
extern "C" fn main() {
    unsafe {
        cube_setup();
    }
    todo!("handoff to freertos (noreturn)");
}

#[unsafe(no_mangle)]
extern "C" fn exit() {
    todo!("handoff to freertos (noreturn)")
}
