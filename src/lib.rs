#![no_std]

mod wasm4;

use wasm4::*;

#[panic_handler]
#[cfg(not(test))]
fn panicked(info: &core::panic::PanicInfo) -> ! {
    trace("panic");
    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        trace("message: ");
        trace(payload);
    }

    core::arch::wasm32::unreachable()
}

#[rustfmt::skip]
const SMILEY: [u8; 8] = [
    0b11000011,
    0b10000001,
    0b00100100,
    0b00100100,
    0b00000000,
    0b00100100,
    0b10011001,
    0b11000011,
];

#[no_mangle]
fn update() {
    *draw_colors() = DrawColors::new().with_0(2);
    text("Hello from Rust!", 10, 10);

    if gamepad()[0].contains(ButtonFlags::X) {
        *draw_colors() = DrawColors::new().with_0(1);
    }

    blit(&SMILEY, 76, 76, 8, 8, BlitFlags::ONE_BIT_PER_PIXEL);
    text("Press X to blink", 16, 90);
}
