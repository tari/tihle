#![allow(non_snake_case)]

use crate::bcalls::test_flag;
use crate::display::ScrollDirection;
use crate::include::tios;
use crate::{Emulator, Z80};

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

pub fn PutS(emu: &mut Emulator, cpu: &Z80) -> usize {
    let mut addr = cpu.regs().hl;
    let mut len = 0;

    loop {
        let c = emu.mem.get(addr).unwrap_or(0);
        if c == 0 {
            break;
        }

        put_char_scrolling(emu, cpu, c);
        addr = addr.wrapping_add(1);
        len += 1;
    }

    PUTC_TIME * len
}

pub fn PutC(emu: &mut Emulator, cpu: &Z80) -> usize {
    put_char_scrolling(emu, cpu, cpu.regs().get_a());
    PUTC_TIME
}

pub fn PutMap(emu: &mut Emulator, cpu: &Z80) -> usize {
    put_char(
        emu,
        cpu.regs().get_a(),
        emu.mem[tios::curRow],
        emu.mem[tios::curCol],
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
    if x >= 15 {
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
    unimplemented!();
}

/// Display a small font character, returning the character width in pixels.
fn put_char_small(emu: &mut Emulator, c: u8, col: u8, row: u8) -> u8 {
    let bitmap_index = 6 * c as usize;
    let width = SMALL_FONT_WIDTHS[c as usize];

    emu.display.blit_8bit_over(col, row, &SMALL_FONT[bitmap_index..bitmap_index+6], width);
    width
}

pub fn VPutS(emu: &mut Emulator, core: &mut Z80) -> usize {
    let mut n_chars = 0;
    let mut ptr = core.regs().hl;
    let mut x = emu.mem[tios::penCol];
    let y = emu.mem[tios::penRow];

    loop {
        let c = emu.mem[ptr];
        if c == 0 {
            break;
        }

        let width = put_char_small(emu, c, x, y);
        x = std::cmp::min(96, x + width);
        ptr += 1;
        n_chars += 1;
    }

    emu.mem[tios::penCol] = x;
    VPUTC_TIME * n_chars
}

pub fn VPutMap(emu: &mut Emulator, core: &mut Z80) -> usize {
    // TODO handle textInverse, textEraseBelow, textWrite and fracDrawLFont flags
    put_char_small(emu, core.regs().get_a(), emu.mem[tios::penCol], emu.mem[tios::penRow]);
    VPUTC_TIME
}

const VPUTC_TIME: usize = 400;

// Small font is variable-width, 6 pixels tall
static SMALL_FONT: &[u8] = include_bytes!("smlfont.bin");
static SMALL_FONT_WIDTHS: &[u8] = include_bytes!("smlfont_widths.bin");

