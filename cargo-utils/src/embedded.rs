use crate::paths::linker_path;

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

static _SHARED_COMPILER_FLAGS: [&str; 1] = [""];

pub fn configure_shared_linker_flags() {
    // shared linker args
    for arg in SHARED_LINKER_ARGS {
        println!("cargo::rustc-link-arg={arg}");
    }
    println!("cargo::rustc-link-search={}", linker_path().display());
    println!("cargo::rustc-link-arg=-Wl,--fatal-warnings");
}
