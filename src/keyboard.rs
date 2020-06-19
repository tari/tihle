use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

/// Keyboard keys, excluding ON (which goes through the interrupt controller)
///
/// Variant values are the same as the scan codes returned by _GetCSC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
pub enum Key {
    Down = 1,
    Left = 2,
    Right = 3,
    Up = 4,
    Enter = 9,
    Plus = 0xA,
    Minus = 0xB,
    Multiply = 0xC,
    Divide = 0xD,
    Caret = 0xE,
    Clear = 0xF,
    Negate = 0x11,
    Three = 0x12,
    Six = 0x13,
    Nine = 0x14,
    CloseParen = 0x15,
    Tangent = 0x16,
    Vars = 0x17,
    Period = 0x19,
    Two = 0x1A,
    Five = 0x1B,
    Eight = 0x1C,
    OpenParen = 0x1D,
    Cosine = 0x1E,
    Program = 0x1F,
    Stat = 0x20,
    Zero = 0x21,
    One = 0x22,
    Four = 0x23,
    Seven = 0x24,
    Comma = 0x25,
    Sine = 0x26,
    Apps = 0x27,
    GraphVar = 0x28,
    Store = 0x2A,
    NaturalLog = 0x2B,
    Log = 0x2C,
    Square = 0x2D,
    Reciprocal = 0x2E,
    Math = 0x2F,
    Alpha = 0x30,
    Graph = 0x31,
    Trace = 0x32,
    Zoom = 0x33,
    Window = 0x34,
    YEquals = 0x35,
    Second = 0x36,
    Mode = 0x37,
    Del = 0x38,
}

pub struct Keyboard {
    /// Bits reset when keys are down; active-low.
    ///
    /// Unimplemented bits always read inactive.
    ///
    /// The mapping from key code to location in this bitmap is expressed by
    /// the `key_group` and `key_mask` functions, which return the index of the
    /// group and bitmask for the key, respectively.
    keys_up: [u8; 8],
    /// Bits clear in this bitmask are active for polling.
    ///
    /// Writing all ones deselects all groups; clearing bit `n` causes reads to
    /// include keys in group `n`.
    active_mask: u8,

    last_scan: Option<Key>,
    repeat_recurring: bool,
    repeat_timer: u8,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            keys_up: [0xFF; 8],
            active_mask: 0xFF,
            last_scan: None,
            repeat_recurring: false,
            repeat_timer: 0,
        }
    }

    pub fn key_down(&mut self, key: Key) {
        self.keys_up[key_group(key)] &= !(1 << key_bit(key));
    }

    pub fn key_up(&mut self, key: Key) {
        self.keys_up[key_group(key)] |= 1 << key_bit(key);
    }

    /// Write the keyboard port (1).
    ///
    /// This sets which groups are included in the result of a [read].
    pub fn set_active_mask(&mut self, mask: u8) {
        self.active_mask = mask;
    }

    /// Read the keyboard port (1).
    ///
    /// A bit in the returned value is reset if a key for that bit in any
    /// enabled group is currently being pressed.
    pub fn read(&self) -> u8 {
        let mut result = 0xFF;

        // For each group, include the active keys if the group is enabled.
        for (idx, &group_bits) in self.keys_up.iter().enumerate() {
            if self.active_mask & (1 << idx as u8) == 0 {
                result &= group_bits;
            }
        }

        result
    }

    pub fn scan(&mut self) -> Option<Key> {
        // Scan codes are 8 * group number + key bit + 1.
        // For example, '+' is in group 1 and is on bit 1, so the scan code is
        // 8 + 1 + 1 = 0x0A.
        let mut scan_code: Option<Key> = None;

        for (group_idx, &group) in self.keys_up.iter().enumerate() {
            for bit in 0..7 {
                if group & (1 << bit) == 0 {
                    if scan_code.is_some() {
                        // Multiple keys are pressed
                        return None;
                    }
                    scan_code = Some(Key::from_u8((8 * group_idx as u8) + bit + 1).unwrap());
                }
            }
        }

        let can_repeat = scan_code
            .map(|k| REPEATABLE_KEYS.contains(&k))
            .unwrap_or(false);
        if self.last_scan == scan_code {
            if !can_repeat {
                // Ignore held non-repeating key, or nothing being pressed
                return None;
            }
            // Repeating key is being held
            if self.repeat_timer == 0 {
                // Timed out; reset timer and switch to higher frequency repeat for
                // ones after the first.
                self.repeat_timer = if self.repeat_recurring { 0x0A } else { 0x32 };
                self.repeat_recurring = true;
            } else {
                // Count timer down and suppress keypress
                self.repeat_timer -= 1;
                return None;
            }
        } else {
            // Scanned a different key from last iteration, store it and reset the repeat timer.
            self.last_scan = scan_code;
            self.repeat_recurring = false;
            self.repeat_timer = 0x32;
        }

        scan_code
    }
}

/// Keys that can repeat when polled with GetCSC
static REPEATABLE_KEYS: [Key; 5] = [Key::Up, Key::Right, Key::Down, Key::Left, Key::Del];

fn key_group(k: Key) -> usize {
    use Key::*;

    match k {
        Down | Left | Right | Up => 0,
        Enter | Plus | Minus | Multiply | Divide | Caret | Clear => 1,
        Negate | Three | Six | Nine | CloseParen | Tangent | Vars => 2,
        Period | Two | Five | Eight | OpenParen | Cosine | Program | Stat => 3,
        Zero | One | Four | Seven | Comma | Sine | Apps | GraphVar => 4,
        Store | NaturalLog | Log | Square | Reciprocal | Math | Alpha => 5,
        Graph | Trace | Zoom | Window | YEquals | Second | Mode | Del => 6,
    }
}

fn key_bit(k: Key) -> u8 {
    use Key::*;

    match k {
        Down | Enter | Negate | Period | Zero | Graph => 0,
        Left | Plus | Three | Two | One | Store | Trace => 1,
        Right | Minus | Six | Five | Four | NaturalLog | Zoom => 2,
        Up | Multiply | Nine | Eight | Seven | Log | Window => 3,
        Divide | CloseParen | OpenParen | Comma | Square | YEquals => 4,
        Caret | Tangent | Cosine | Sine | Reciprocal | Second => 5,
        Clear | Vars | Program | Apps | Math | Mode => 6,
        Stat | GraphVar | Alpha | Del => 7,
    }
}
