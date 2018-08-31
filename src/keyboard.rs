use cpuio;
use spin::Mutex;

/// A pair of keys wich appear on both left and right side
/// such as "left shift" and "right shift"
#[derive(Debug)]
struct KeyPair {
    left: bool,
    right: bool,
}

impl KeyPair {
    // Creates new KeyPair with false values
    const fn new() -> Self {
        KeyPair {
            left: false,
            right: false,
        }
    }
    // Check if either is true
    fn is_pressed(&self) -> bool {
        self.left || self.right
    }
}

/// All of our supported keyboard modifiers.
#[derive(Debug)]
struct Modifiers {
    shift: KeyPair,
    control: KeyPair,
    alt: KeyPair,
    caps_lock: bool,
}

impl Modifiers {
    const fn new() -> Self {
        Modifiers {
            shift: KeyPair::new(),
            control: KeyPair::new(),
            alt: KeyPair::new(),
            caps_lock: false,
        }
    }

    // Given modifiers should we convert letter to uppercase?
    fn use_uppercase_letters(&self) -> bool {
        self.shift.is_pressed() ^ self.caps_lock
    }

    // Apply all of our modifiers to an ASCII char and return new one
    fn apply_to(&self, ascii: u8) -> u8 {
        if b'a' <= ascii && ascii <= b'z' {
            if self.use_uppercase_letters() {
                return ascii - b'a' + b'A';
            }
        } else {
            if self.shift.is_pressed() {
                match ascii {
                    b'1' => return ascii - b'1' + b'!',
                    b'2' => return ascii - b'2' + b'@',
                    b'3' => return ascii - b'3' + b'#',
                    b'4' => return ascii - b'4' + b'$',
                    b'5' => return ascii - b'5' + b'%',
                    b'6' => return ascii - b'6' + b'^',
                    b'7' => return ascii - b'7' + b'&',
                    b'8' => return ascii - b'8' + b'*',
                    b'9' => return ascii - b'9' + b'(',
                    b'0' => return ascii - b'0' + b')',
                    b'-' => return ascii - b'-' + b'_',
                    b'=' => return ascii - b'=' + b'+',
                    b',' => return ascii - b',' + b'<',
                    b'.' => return ascii - b'.' + b'>',
                    b'/' => return ascii - b'/' + b'?',
                    b'[' => return ascii - b'[' + b'{',
                    b']' => return ascii - b']' + b'}',
                    b';' => return ascii - b';' + b':',
                    b'\'' => return ascii - b'\'' + b'"',

                    _ => {}
                }
            } else {
                return ascii;
            }
        }

        ascii
    }

    // Given scancode update current modifer state
    fn update(&mut self, scancode: u8) {
        match scancode {
            0x1D => self.control.left = true,
            0x2A => self.shift.left = true,
            0x36 => self.shift.right = true,
            0x38 => self.alt.left = true,
            // Caps lock toggles on leading edge
            0x3A => self.caps_lock = !self.caps_lock,
            0x9D => self.control.left = false,
            0xAA => self.shift.left = false,
            0xB6 => self.shift.right = false,
            0xB8 => self.alt.left = false,

            _ => {}
        }
    }
}

/// Our keyboard state, including I/O port, pressed modifiers, etc
struct State {
    port: cpuio::Port<u8>,
    modifiers: Modifiers,
}

static STATE: Mutex<State> = Mutex::new(State {
    port: unsafe { cpuio::Port::new(0x60) },
    modifiers: Modifiers::new(),
});

// Convert scancode to ASCII if we understand it
fn find_asii(scancode: u8) -> Option<u8> {
    let index = scancode as usize;
    match scancode {
        0x01...0x0E => Some(b"\x1B1234567890-=\0x02"[index - 0x01]),
        0x0F...0x1C => Some(b"\tqwertyuiop[]\r"[index - 0x0F]),
        0x1E...0x28 => Some(b"asdfghjkl;'"[index - 0x1E]),
        0x2C...0x35 => Some(b"zxcvbnm,./"[index - 0x2C]),
        0x39 => Some(b' '),
        _ => None,
    }
}

/// Try to read a single input character
pub fn read_char() -> Option<char> {
    let mut state = STATE.lock();
    let scancode = state.port.read();
    state.modifiers.update(scancode);

    if let Some(ascii) = find_asii(scancode) {
        Some(state.modifiers.apply_to(ascii) as char)
    } else {
        None
    }
}
