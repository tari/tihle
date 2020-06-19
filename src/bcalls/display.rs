#![allow(non_snake_case)]

use super::test_flag;
use crate::display::ScrollDirection;
use crate::include::tios;
use crate::{Emulator, Flags, Z80};

pub fn HomeUp(emu: &mut Emulator) -> usize {
    emu.mem[tios::curCol] = 0;
    emu.mem[tios::curRow] = 0;
    16
}

pub fn ClrLCDFull(emu: &mut Emulator) -> usize {
    emu.display.clear();
    60000 // Slower than ionFastCopy
}

pub fn GrBufCpy(emu: &mut Emulator) -> usize {
    emu.display
        .blit_fullscreen(&emu.mem[tios::plotSScreen..tios::plotSScreen + 768]);
    60000 //  Slower than ionFastCopy
}

pub fn PutC(emu: &mut Emulator, cpu: &Z80) -> usize {
    put_char_scrolling(emu, cpu, cpu.regs().get_a());
    PUTC_TIME
}

pub fn PutMap(emu: &mut Emulator, cpu: &Z80) -> usize {
    put_char(
        emu,
        cpu.regs().get_a(),
        emu.mem[tios::curCol],
        emu.mem[tios::curRow],
    );
    PUTC_TIME
}

/// Approximate cycle count to write a character to the screen.
const PUTC_TIME: usize = 500;

/// Write the given character to the screen and update cursor, scrolling if necessary.
fn put_char_scrolling(emu: &mut Emulator, cpu: &Z80, c: u8) {
    let mut y = emu.mem[tios::curRow];
    let mut x = emu.mem[tios::curCol];

    put_char(emu, c, x, y);
    x += 1;
    if x > 15 {
        let should_scroll = test_flag(emu, cpu, tios::appFlags, tios::appAutoScroll);

        // Going offscreen without scrolling clamps to the bottom right corner
        // and stops displaying chars.
        if !should_scroll {
            emu.mem[tios::curCol] = 15;
            return;
        }

        x = 0;
        y += 1;
        if y >= 7 {
            y = 6;
            emu.display.scroll(ScrollDirection::Up, 8);
        }
    }

    emu.mem[tios::curCol] = x;
    emu.mem[tios::curRow] = y;
}

fn put_char(emu: &mut Emulator, c: u8, col: u8, row: u8) {
    assert!(
        col < 16 && row < 8,
        "Screen coordinates ({}, {}) are out of bounds",
        col,
        row
    );
    let char_index = c as usize * 7;
    debug!(
        "Display char {:02x} @ {}: data = {:?}",
        c,
        char_index,
        &LARGE_FONT[char_index..char_index + 7]
    );

    emu.display
        .blit_8bit_over(col * 6, row * 8, &LARGE_FONT[char_index..char_index + 7], 6);
}

static LARGE_FONT: &[u8] = include_bytes!("lgfont.bin");

pub fn DispHL(emu: &mut Emulator, core: &mut Z80) -> usize {
    let s = format!("{:5}", core.regs().hl);
    emu.mem[tios::OP1..tios::OP1 + 5].copy_from_slice(s.as_bytes());

    let row = emu.mem[tios::curRow];
    let start_col = std::cmp::min(emu.mem[tios::curCol], 15);
    for (c, col) in s.chars().zip(start_col..16) {
        put_char(emu, c as u8, col, row);
    }

    PUTC_TIME * 5 + 200
}

/// Display a small font character, returning the character width in pixels.
fn put_char_small(emu: &mut Emulator, c: u8, col: u8, row: u8) -> u8 {
    let bitmap_index = 6 * c as usize;
    let width = SMALL_FONT_WIDTHS[c as usize];

    emu.display
        .blit_8bit_over(col, row, &SMALL_FONT[bitmap_index..bitmap_index + 6], width);
    width
}

pub fn VPutMap(emu: &mut Emulator, core: &mut Z80) -> usize {
    // TODO handle textInverse, textEraseBelow, textWrite and fracDrawLFont flags
    let mut x = emu.mem[tios::penCol];
    let width = put_char_small(emu, core.regs().get_a(), x, emu.mem[tios::penRow]);

    x = x.wrapping_add(width);
    emu.mem[tios::penCol] = x;
    // Set carry if we've gone offscren
    if x >= 96 {
        core.set_flags(core.flags() | Flags::C);
    } else {
        core.set_flags(core.flags() - Flags::C);
    }
    VPUTC_TIME
}

const VPUTC_TIME: usize = 400;

// Small font is variable-width, 6 pixels tall
static SMALL_FONT: &[u8] = include_bytes!("smlfont.bin");
static SMALL_FONT_WIDTHS: &[u8] = include_bytes!("smlfont_widths.bin");
