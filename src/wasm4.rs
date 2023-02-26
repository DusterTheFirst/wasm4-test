//
// WASM-4: https://wasm4.org/docs

#![allow(unused)]

use core::{
    mem::transmute,
    sync::atomic::{fence, Ordering},
};

use bitflags::bitflags;
use modular_bitfield::{
    bitfield,
    specifiers::{B2, B5},
};

// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │ Platform Constants                                                        │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

pub const SCREEN_SIZE: u32 = 160;

// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │ Memory Addresses                                                          │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

mod addr {
    use super::{ButtonFlags, DrawColors, MouseFlags, Rgb, SystemFlags};

    pub const PALETTE: *mut [Rgb; 4] = 0x04 as *mut [Rgb; 4];
    pub const DRAW_COLORS: *mut DrawColors = 0x14 as *mut DrawColors;

    // 0x16 - 0x19
    pub const GAMEPAD: *const [ButtonFlags; 4] = 0x16 as *const [ButtonFlags; 4];

    pub const MOUSE_X: *const i16 = 0x1a as *const i16;
    pub const MOUSE_Y: *const i16 = 0x1c as *const i16;

    pub const MOUSE_BUTTONS: *const MouseFlags = 0x1e as *const MouseFlags;
    pub const SYSTEM_FLAGS: *mut SystemFlags = 0x1f as *mut SystemFlags;

    pub const NETPLAY: *const u8 = 0x20 as *const u8;
    pub const FRAMEBUFFER: *mut [u8; 6400] = 0xa0 as *mut [u8; 6400];
}

#[inline(always)]
pub fn palette() -> &'static mut [Rgb; 4] {
    unsafe { &mut *addr::PALETTE }
}

#[inline(always)]
pub fn draw_colors() -> &'static mut DrawColors {
    unsafe { &mut *addr::DRAW_COLORS }
}

#[inline(always)]
pub fn gamepad() -> &'static [ButtonFlags; 4] {
    unsafe { &*addr::GAMEPAD }
}

#[repr(C, packed)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    _unused: u8,
}

impl Rgb {
    #[inline(always)]
    pub fn new(red: u8, blue: u8, green: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
            _unused: 0,
        }
    }

    #[inline(always)]
    pub fn red(red: u8) -> Self {
        Self::new(red, 0, 0)
    }

    #[inline(always)]
    pub fn green(green: u8) -> Self {
        Self::new(0, green, 0)
    }

    #[inline(always)]
    pub fn blue(blue: u8) -> Self {
        Self::new(0, 0, blue)
    }
}

#[bitfield]
pub struct DrawColors(pub B2, pub B2, pub B2, pub B2);

bitflags! {
    #[repr(transparent)]
    pub struct ButtonFlags: u8 {
        const X     = 1 << 0;
        const Z     = 1 << 1;
        // Unused   = 1 << 2
        // Unused   = 1 << 3
        const LEFT  = 1 << 4;
        const RIGHT = 1 << 5;
        const UP    = 1 << 6;
        const DOWN  = 1 << 7;
    }

    #[repr(transparent)]
    pub struct MouseFlags: u8 {
        const LEFT      = 1 << 0;
        const RIGHT     = 1 << 1;
        const MIDDLE    = 1 << 2;
    }

    #[repr(transparent)]
    pub struct SystemFlags: u8 {
        const PRESERVE_FRAMEBUFFER = 1 << 0;
        const HIDE_GAMEPAD_OVERLAY = 1 << 1;
    }
}

#[bitfield]
pub struct NetPlay {
    #[skip(setters)]
    player_index: B2,

    #[skip(setters)]
    #[bits = 1]
    active: bool,

    #[skip]
    _unused: B5,
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │ Drawing Functions                                                         │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

/// Copies pixels to the framebuffer.
pub fn blit(sprite: &[u8], x: i32, y: i32, width: u32, height: u32, flags: BlitFlags) {
    extern "C" {
        fn blit(sprite: *const u8, x: i32, y: i32, width: u32, height: u32, flags: u32);
    }

    unsafe { blit(sprite.as_ptr(), x, y, width, height, flags.bits) }
}

/// Copies a subregion within a larger sprite atlas to the framebuffer.
#[allow(clippy::too_many_arguments)]
pub fn blit_sub(
    sprite: &[u8],
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    src_x: u32,
    src_y: u32,
    stride: u32,
    flags: BlitFlags,
) {
    extern "C" {
        #[link_name = "blitSub"]
        fn blit_sub(
            sprite: *const u8,
            x: i32,
            y: i32,
            width: u32,
            height: u32,
            src_x: u32,
            src_y: u32,
            stride: u32,
            flags: u32,
        );
    }

    unsafe {
        blit_sub(
            sprite.as_ptr(),
            x,
            y,
            width,
            height,
            src_x,
            src_y,
            stride,
            flags.bits,
        )
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct BlitFlags: u32 {
        const TWO_BIT_PER_PIXEL = 1;
        const ONE_BIT_PER_PIXEL = 0;

        const FLIP_X = 2;
        const FLIP_Y = 4;

        const ROTATE = 8;
    }
}

/// Draws a line between two points.
pub fn line(x1: i32, y1: i32, x2: i32, y2: i32) {
    extern "C" {
        #[link_name = "line"]
        fn extern_line(x1: i32, y1: i32, x2: i32, y2: i32);
    }

    unsafe { extern_line(x1, y1, x2, y2) }
}

/// Draws an oval (or circle).
pub fn oval(x: i32, y: i32, width: u32, height: u32) {
    extern "C" {
        #[link_name = "oval"]
        fn extern_oval(x: i32, y: i32, width: u32, height: u32);
    }

    unsafe { extern_oval(x, y, width, height) }
}

/// Draws a rectangle.
pub fn rect(x: i32, y: i32, width: u32, height: u32) {
    extern "C" {
        #[link_name = "rect"]
        fn extern_rect(x: i32, y: i32, width: u32, height: u32);
    }

    unsafe { extern_rect(x, y, width, height) }
}

/// Draws text using the built-in system font.
pub fn text<T: AsRef<[u8]>>(text: T, x: i32, y: i32) {
    extern "C" {
        #[link_name = "textUtf8"]
        fn extern_text_utf8(text: *const u8, length: usize, x: i32, y: i32);
    }

    let text_ref = text.as_ref();
    unsafe { extern_text_utf8(text_ref.as_ptr(), text_ref.len(), x, y) }
}

/// Draws a vertical line
pub fn vline(x: i32, y: i32, len: u32) {
    extern "C" {
        #[link_name = "vline"]
        fn extern_vline(x: i32, y: i32, len: u32);
    }

    unsafe { extern_vline(x, y, len) }
}

/// Draws a horizontal line
pub fn hline(x: i32, y: i32, len: u32) {
    extern "C" {
        #[link_name = "hline"]
        fn extern_hline(x: i32, y: i32, len: u32);
    }

    unsafe { extern_hline(x, y, len) }
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │ Sound Functions                                                           │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

/// Plays a sound tone.
pub fn tone(
    frequency: ToneFrequency,
    duration: ToneDuration,
    volume: ToneVolume,
    flags: ToneFlags,
) {
    extern "C" {
        #[link_name = "tone"]
        fn extern_tone(frequency: u32, duration: u32, volume: u16, flags: u8);
    }

    unsafe {
        extern_tone(
            transmute(frequency),
            transmute(duration),
            transmute(volume),
            flags.channel as u8 | flags.mode as u8 | flags.pan as u8,
        )
    }
}

#[repr(C, packed)]
pub struct ToneFrequency {
    pub start: u16,
    pub stop: u16,
}

#[repr(C, packed)]
pub struct ToneDuration {
    pub sustain: u8,
    pub delay: u8,
    pub release: u8,
    pub attack: u8,
}

#[repr(C, packed)]
pub struct ToneVolume {
    pub sustain: u8,
    pub peak: u8,
}

pub struct ToneFlags {
    pub channel: ToneChannel,
    pub mode: ToneMode,
    pub pan: TonePan,
}

#[repr(u8)]
pub enum ToneChannel {
    Pulse1 = 0b00,
    Pulse2 = 0b01,
    Triangle = 0b10,
    Noise = 0b11,
}

#[repr(u8)]
pub enum ToneMode {
    /// 1/8 duty cycle
    Mode1 = 0b00_00,
    /// 1/4 duty cycle
    Mode2 = 0b01_00,
    /// 1/2 duty cycle
    Mode3 = 0b10_00,
    /// 3/4 duty cycle
    Mode4 = 0b11_00,
}

#[repr(u8)]
pub enum TonePan {
    Center = 0b00_00_00,
    Left = 0b01_00_00,
    Right = 0b10_00_00,
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │ Storage Functions                                                         │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

extern "C" {
    /// Reads up to `size` bytes from persistent storage into the pointer `dest`.
    pub fn diskr(dest: *mut u8, size: u32) -> u32;

    /// Writes up to `size` bytes from the pointer `src` into persistent storage.
    pub fn diskw(src: *const u8, size: u32) -> u32;
}

// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │ Other Functions                                                           │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

/// Prints a message to the debug console.
pub fn trace<T: AsRef<str>>(text: T) {
    extern "C" {
        #[link_name = "traceUtf8"]
        fn extern_trace(trace: *const u8, length: usize);
    }

    let text_ref = text.as_ref();
    unsafe { extern_trace(text_ref.as_ptr(), text_ref.len()) }
}
