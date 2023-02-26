#![no_std]

mod wasm4;

use core::{arch::wasm32, fmt::write, panic::PanicInfo, str};

use wasm4::*;

struct Formatter<'b> {
    buffer: &'b mut [u8],
    cursor: usize,
}

impl<'b> core::fmt::Write for Formatter<'b> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.buffer.len() < self.cursor + s.len() {
            return Err(core::fmt::Error);
        }

        self.buffer[self.cursor..][..s.len()].copy_from_slice(s.as_bytes());

        self.cursor += s.len();

        Ok(())
    }
}

impl<'b> Formatter<'b> {
    pub fn new(buffer: &'b mut [u8]) -> Self {
        // Ensure buffer contents are all valid utf-8
        buffer.fill(b'A');

        Self { buffer, cursor: 0 }
    }

    pub fn buffer(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.buffer[..self.cursor]) }
    }
}

#[panic_handler]
fn panicked(info: &PanicInfo) -> ! {
    trace("panic");
    if let Some(location) = info.location() {
        trace("location:");
        trace(location.file());

        let mut buffer = [0_u8; 9];
        let mut formatter = Formatter::new(&mut buffer);

        if let Ok(()) = write(
            &mut formatter,
            format_args!("{}:{}", location.line(), location.column()),
        ) {
            trace(formatter.buffer());
        } else {
            trace("NO NUMBERS");
        }
    }

    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        trace("message: ");
        trace(payload);
    }

    wasm32::unreachable()
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
fn start() {
    tone(
        ToneFrequency {
            start: 440,
            stop: 880,
        },
        ToneDuration {
            sustain: 100,
            delay: 0,
            release: 0,
            attack: 0,
        },
        ToneVolume {
            sustain: 110,
            peak: 255,
        },
        ToneFlags {
            channel: ToneChannel::Pulse1,
            mode: ToneMode::Mode3,
            pan: TonePan::Center,
        },
    )
}

#[no_mangle]
fn update() {
    *draw_colors() = DrawColors::new().with_0(2);
    text("Hello from Rust!", 10, 10);

    panic!("e");

    if gamepad()[0].contains(ButtonFlags::X) {
        *draw_colors() = DrawColors::new().with_1(1);
    }

    blit(&SMILEY, 76, 76, 8, 8, BlitFlags::ONE_BIT_PER_PIXEL);
    text("Press X to blink", 16, 90);
}
