use std::io;
use std::io::Write;
use std::io::{Stderr, Stdout};
use std::mem::zeroed;

use crate::bytes::Bytes;

use winapi::shared::minwindef::{DWORD, WORD};
use winapi::shared::ntdef::{HANDLE, WCHAR};
use winapi::um::wincon;
use winapi::um::wincon::{
    BACKGROUND_BLUE, BACKGROUND_GREEN, BACKGROUND_INTENSITY, BACKGROUND_RED, FOREGROUND_BLUE,
    FOREGROUND_GREEN, FOREGROUND_INTENSITY, FOREGROUND_RED,
};
use winapi::um::wincon::{CONSOLE_CURSOR_INFO, CONSOLE_SCREEN_BUFFER_INFO};
use winapi::um::wincontypes::{COORD, SMALL_RECT};

pub const FOREGROUND_MASK: WORD =
    (FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_INTENSITY);
pub const BACKGROUND_MASK: WORD =
    (BACKGROUND_RED | BACKGROUND_BLUE | BACKGROUND_GREEN | BACKGROUND_INTENSITY);

pub struct Colorable<W: Write> {
    out: W,
    handle: HANDLE,
    althandle: HANDLE,
    oldattr: WORD,
    oldpos: COORD,
    rest: Bytes,
}

pub fn colorable_stdout() -> Colorable<Stdout> {
    use std::os::windows::io::AsRawHandle;
    let handle = io::stdout().as_raw_handle() as HANDLE;
    colorable(io::stdout(), handle)
}

pub fn colorable_stderr() -> Colorable<Stderr> {
    use std::os::windows::io::AsRawHandle;
    let handle = io::stderr().as_raw_handle() as HANDLE;
    colorable(io::stderr(), handle)
}

fn colorable<W: Write>(out: W, handle: HANDLE) -> Colorable<W> {
    use std::ptr::null_mut;

    let csbi = get_console_screen_buffer_info(handle);

    Colorable {
        out,
        handle,
        althandle: null_mut(),
        oldattr: csbi.wAttributes,
        oldpos: COORD { X: 0, Y: 0 },
        rest: Bytes::new(),
    }
}

const COLOR256: [u32; 256] = [
    0x00_00_00, 0x80_00_00, 0x00_80_00, 0x80_80_00, 0x00_00_80, 0x80_00_80, 0x00_80_80, 0xc0_c0_c0,
    0x80_80_80, 0xff_00_00, 0x00_ff_00, 0xff_ff_00, 0x00_00_ff, 0xff_00_ff, 0x00_ff_ff, 0xff_ff_ff,
    0x00_00_00, 0x00_00_5f, 0x00_00_87, 0x00_00_af, 0x00_00_d7, 0x00_00_ff, 0x00_5f_00, 0x00_5f_5f,
    0x00_5f_87, 0x00_5f_af, 0x00_5f_d7, 0x00_5f_ff, 0x00_87_00, 0x00_87_5f, 0x00_87_87, 0x00_87_af,
    0x00_87_d7, 0x00_87_ff, 0x00_af_00, 0x00_af_5f, 0x00_af_87, 0x00_af_af, 0x00_af_d7, 0x00_af_ff,
    0x00_d7_00, 0x00_d7_5f, 0x00_d7_87, 0x00_d7_af, 0x00_d7_d7, 0x00_d7_ff, 0x00_ff_00, 0x00_ff_5f,
    0x00_ff_87, 0x00_ff_af, 0x00_ff_d7, 0x00_ff_ff, 0x5f_00_00, 0x5f_00_5f, 0x5f_00_87, 0x5f_00_af,
    0x5f_00_d7, 0x5f_00_ff, 0x5f_5f_00, 0x5f_5f_5f, 0x5f_5f_87, 0x5f_5f_af, 0x5f_5f_d7, 0x5f_5f_ff,
    0x5f_87_00, 0x5f_87_5f, 0x5f_87_87, 0x5f_87_af, 0x5f_87_d7, 0x5f_87_ff, 0x5f_af_00, 0x5f_af_5f,
    0x5f_af_87, 0x5f_af_af, 0x5f_af_d7, 0x5f_af_ff, 0x5f_d7_00, 0x5f_d7_5f, 0x5f_d7_87, 0x5f_d7_af,
    0x5f_d7_d7, 0x5f_d7_ff, 0x5f_ff_00, 0x5f_ff_5f, 0x5f_ff_87, 0x5f_ff_af, 0x5f_ff_d7, 0x5f_ff_ff,
    0x87_00_00, 0x87_00_5f, 0x87_00_87, 0x87_00_af, 0x87_00_d7, 0x87_00_ff, 0x87_5f_00, 0x87_5f_5f,
    0x87_5f_87, 0x87_5f_af, 0x87_5f_d7, 0x87_5f_ff, 0x87_87_00, 0x87_87_5f, 0x87_87_87, 0x87_87_af,
    0x87_87_d7, 0x87_87_ff, 0x87_af_00, 0x87_af_5f, 0x87_af_87, 0x87_af_af, 0x87_af_d7, 0x87_af_ff,
    0x87_d7_00, 0x87_d7_5f, 0x87_d7_87, 0x87_d7_af, 0x87_d7_d7, 0x87_d7_ff, 0x87_ff_00, 0x87_ff_5f,
    0x87_ff_87, 0x87_ff_af, 0x87_ff_d7, 0x87_ff_ff, 0xaf_00_00, 0xaf_00_5f, 0xaf_00_87, 0xaf_00_af,
    0xaf_00_d7, 0xaf_00_ff, 0xaf_5f_00, 0xaf_5f_5f, 0xaf_5f_87, 0xaf_5f_af, 0xaf_5f_d7, 0xaf_5f_ff,
    0xaf_87_00, 0xaf_87_5f, 0xaf_87_87, 0xaf_87_af, 0xaf_87_d7, 0xaf_87_ff, 0xaf_af_00, 0xaf_af_5f,
    0xaf_af_87, 0xaf_af_af, 0xaf_af_d7, 0xaf_af_ff, 0xaf_d7_00, 0xaf_d7_5f, 0xaf_d7_87, 0xaf_d7_af,
    0xaf_d7_d7, 0xaf_d7_ff, 0xaf_ff_00, 0xaf_ff_5f, 0xaf_ff_87, 0xaf_ff_af, 0xaf_ff_d7, 0xaf_ff_ff,
    0xd7_00_00, 0xd7_00_5f, 0xd7_00_87, 0xd7_00_af, 0xd7_00_d7, 0xd7_00_ff, 0xd7_5f_00, 0xd7_5f_5f,
    0xd7_5f_87, 0xd7_5f_af, 0xd7_5f_d7, 0xd7_5f_ff, 0xd7_87_00, 0xd7_87_5f, 0xd7_87_87, 0xd7_87_af,
    0xd7_87_d7, 0xd7_87_ff, 0xd7_af_00, 0xd7_af_5f, 0xd7_af_87, 0xd7_af_af, 0xd7_af_d7, 0xd7_af_ff,
    0xd7_d7_00, 0xd7_d7_5f, 0xd7_d7_87, 0xd7_d7_af, 0xd7_d7_d7, 0xd7_d7_ff, 0xd7_ff_00, 0xd7_ff_5f,
    0xd7_ff_87, 0xd7_ff_af, 0xd7_ff_d7, 0xd7_ff_ff, 0xff_00_00, 0xff_00_5f, 0xff_00_87, 0xff_00_af,
    0xff_00_d7, 0xff_00_ff, 0xff_5f_00, 0xff_5f_5f, 0xff_5f_87, 0xff_5f_af, 0xff_5f_d7, 0xff_5f_ff,
    0xff_87_00, 0xff_87_5f, 0xff_87_87, 0xff_87_af, 0xff_87_d7, 0xff_87_ff, 0xff_af_00, 0xff_af_5f,
    0xff_af_87, 0xff_af_af, 0xff_af_d7, 0xff_af_ff, 0xff_d7_00, 0xff_d7_5f, 0xff_d7_87, 0xff_d7_af,
    0xff_d7_d7, 0xff_d7_ff, 0xff_ff_00, 0xff_ff_5f, 0xff_ff_87, 0xff_ff_af, 0xff_ff_d7, 0xff_ff_ff,
    0x08_08_08, 0x12_12_12, 0x1c_1c_1c, 0x26_26_26, 0x30_30_30, 0x3a_3a_3a, 0x44_44_44, 0x4e_4e_4e,
    0x58_58_58, 0x62_62_62, 0x6c_6c_6c, 0x76_76_76, 0x80_80_80, 0x8a_8a_8a, 0x94_94_94, 0x9e_9e_9e,
    0xa8_a8_a8, 0xb2_b2_b2, 0xbc_bc_bc, 0xc6_c6_c6, 0xd0_d0_d0, 0xda_da_da, 0xe4_e4_e4, 0xee_ee_ee,
];

struct ConsoleColor {
    rgb: u32,
    red: bool,
    green: bool,
    blue: bool,
    intensity: bool,
}

impl ConsoleColor {
    const fn new(rgb: u32, red: bool, green: bool, blue: bool, intensity: bool) -> ConsoleColor {
        ConsoleColor {
            rgb,
            red,
            green,
            blue,
            intensity,
        }
    }

    fn attr_fore(&self) -> WORD {
        let mut attr = 0;

        if self.red {
            attr |= FOREGROUND_RED;
        }

        if self.green {
            attr |= FOREGROUND_GREEN;
        }

        if self.blue {
            attr |= FOREGROUND_BLUE;
        }

        attr
    }

    fn attr_back(&self) -> WORD {
        let mut attr = 0;
        if self.red {
            attr |= BACKGROUND_RED;
        }

        if self.green {
            attr |= BACKGROUND_GREEN;
        }

        if self.blue {
            attr |= BACKGROUND_BLUE;
        }

        attr
    }
}

const COLOR16: [ConsoleColor; 16] = [
    ConsoleColor::new(0x00_00_00, false, false, false, false),
    ConsoleColor::new(0x00_00_80, false, false, true, false),
    ConsoleColor::new(0x00_80_00, false, true, false, false),
    ConsoleColor::new(0x00_80_80, false, true, true, false),
    ConsoleColor::new(0x80_00_00, true, false, false, false),
    ConsoleColor::new(0x80_00_80, true, false, true, false),
    ConsoleColor::new(0x80_80_00, true, true, false, false),
    ConsoleColor::new(0xc0_c0_c0, true, true, true, false),
    ConsoleColor::new(0x80_80_80, false, false, false, true),
    ConsoleColor::new(0x00_00_ff, false, false, true, true),
    ConsoleColor::new(0x00_ff_00, false, true, false, true),
    ConsoleColor::new(0x00_ff_ff, false, true, true, true),
    ConsoleColor::new(0xff_00_00, true, false, false, true),
    ConsoleColor::new(0xff_00_ff, true, false, true, true),
    ConsoleColor::new(0xff_ff_00, true, true, false, true),
    ConsoleColor::new(0xff_ff_ff, true, true, true, true),
];

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;

impl<W: Write> io::Write for Colorable<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let csbi = get_console_screen_buffer_info(self.handle);
        self.rest.extend(buf);

        self.rest.reset_counter();

        'mainloop: while let Some(c1) = self.rest.next() {
            if c1 != ESC {
                self.out.write_all(&[c1])?;
                continue;
            }

            let c2 = match self.rest.next() {
                None => break,
                Some(c) => c,
            };

            let handled = match c2 {
                b'7' => self.handle_save_cursor_pos(),
                b'8' => self.handle_restore_cursor_pos(),
                b'[' => self.handle_csi(),
                b']' => self.handle_osc(),
                _ => false,
            };

            if !handled {
                // if the function didn't handle them, it means it lacks some
                // parts of bytes.  unget for now and wait for things to be
                // prepared.
                self.rest.unget(c2);
                self.rest.unget(c1);
                break 'mainloop;
            }
        }

        Ok(self.rest.read_count())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<W: Write> Colorable<W> {
    fn handle_save_cursor_pos(&mut self) -> bool {
        self.oldpos = get_console_screen_buffer_info(self.handle).dwCursorPosition;
        true
    }

    fn handle_restore_cursor_pos(&mut self) -> bool {
        set_console_cursor_position(self.handle, self.oldpos);
        true
    }

    fn handle_csi(&mut self) -> bool {
        unimplemented!()
    }

    fn handle_osc(&mut self) -> bool {
        if !self.rest.exists(BEL) {
            // BEL is the end of the OSC commadns, so no BEL means entire sequence has not been
            // written yet.  waiting for next call of Write::write(), hoping the next time things
            // are got ready.
            return false;
        }

        let id = self.rest.by_ref().take_while(|&x| x != b';');
        let id = id.fold(0u32, |sum, x| sum + (x - b'0') as u32);

        let arg = self.rest.by_ref().take_while(|&x| x != BEL);
        let arg: Vec<_> = arg.collect();

        match id {
            1 | 2 | 3 => set_console_title(String::from_utf8_lossy(&arg).into()),
            4 => unimplemented!("OSC: changing color palette"),
            5 => unimplemented!("OSC: getting char color"),
            10 | 12 | 13 | 15 | 18 | 19 => unimplemented!("OSC: getting char color"),
            11 | 14 | 16 | 17 => unimplemented!("OSC: getting foreground color"),
            52 => unimplemented!("OSC: manipulating clipboard contents"),
            104 => unimplemented!("OSC: resetting color palette"),
            105 => {}
            110 | 112 | 113 | 115 | 118 | 119 => unimplemented!("OSC: resetting char color"),
            111 | 114 | 116 | 117 => unimplemented!("OSC: resetting background color"),

            1337 => unimplemented!("OSC: 1337"),

            800 => unimplemented!("OSC: changing kanji code"),

            801 => unimplemented!("OSC: pronounce by synthesized voice"), // !?

            id => panic!("OSC: unknown command id: {}", id),
        }

        true
    }
}

fn set_console_cursor_position(handle: HANDLE, pos: COORD) {
    unsafe {
        wincon::SetConsoleCursorPosition(handle, pos);
    }
}

fn get_console_screen_buffer_info(handle: HANDLE) -> CONSOLE_SCREEN_BUFFER_INFO {
    unsafe {
        let mut csbi = zeroed();
        wincon::GetConsoleScreenBufferInfo(handle, &mut csbi);
        csbi
    }
}

fn set_console_title(title: String) {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStrExt;
    unsafe {
        let title: Vec<_> = OsString::from(title).encode_wide().collect();
        wincon::SetConsoleTitleW(title.as_ptr());
    }
}
