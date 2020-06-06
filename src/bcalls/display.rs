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
    unimplemented!();
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
    let char_index = (c * 7) as usize;

    emu.display
        .blit_8bit_over(col * 6, row * 8, &LARGE_FONT[char_index..char_index + 7], 6);
}

static LARGE_FONT: &[u8] = include_bytes!("lgfont.bin");
