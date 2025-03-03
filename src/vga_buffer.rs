use core::convert::TryFrom;
use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl TryFrom<u8> for Color {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Color::Black),
            1 => Ok(Color::Blue),
            2 => Ok(Color::Green),
            3 => Ok(Color::Cyan),
            4 => Ok(Color::Red),
            5 => Ok(Color::Magenta),
            6 => Ok(Color::Brown),
            7 => Ok(Color::LightGray),
            8 => Ok(Color::DarkGray),
            9 => Ok(Color::LightBlue),
            10 => Ok(Color::LightGreen),
            11 => Ok(Color::LightCyan),
            12 => Ok(Color::LightRed),
            13 => Ok(Color::Pink),
            14 => Ok(Color::Yellow),
            15 => Ok(Color::White),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub fn default() -> ColorCode {
        ColorCode::new(Color::White, Color::Black)
    }

    pub fn foreground(mut self, color: Color) -> ColorCode {
        self.0 = (self.0 & 0xf0) | (color as u8);
        self
    }

    pub fn background(mut self, color: Color) -> ColorCode {
        self.0 = (self.0 & 0x0f) | ((color as u8) << 4);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].clone();
                    self.buffer.chars[row - 1][col] = character;
                }
            }
            // Clear last row
            self.clear_row(BUFFER_HEIGHT - 1);
        }

        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: ColorCode::default(),
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn background(&mut self, color: Color) {
        self.color_code = self.color_code.background(color);
    }

    pub fn foreground(&mut self, color: Color) {
        self.color_code = self.color_code.foreground(color);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
