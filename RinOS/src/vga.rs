use core::arch::asm;
use core::borrow::BorrowMut;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]

use crate::asm;
use crate::font;

const MOUSE_CURSOR_WIDTH: usize = 16;
const MOUSE_CURSOR_HEIGHT: usize = 16;

const TABLE_RGB: [[u8;3]; 16] = [
    [0x00, 0x00, 0x00], /*  0:黑 */
    [0xff, 0x00, 0x00],	/*  1:亮红 */
    [0x00, 0xff, 0x00],	/*  2:亮绿 */
    [0xff, 0xff, 0x00],	/*  3:亮黄 */
    [0x00, 0x00, 0xff],	/*  4:亮蓝 */
    [0xff, 0x00, 0xff],	/*  5:亮紫 */
    [0x00, 0xff, 0xff],	/*  6:浅亮蓝 */
    [0xff, 0xff, 0xff],	/*  7:白 */
    [0xc6, 0xc6, 0xc6],	/*  8:亮灰 */
    [0x84, 0x00, 0x00],	/*  9:暗红 */
    [0x00, 0x84, 0x00],	/* 10:暗绿 */
    [0x84, 0x84, 0x00],	/* 11:暗黄 */
    [0x00, 0x00, 0x84],	/* 12:暗青 */
    [0x84, 0x00, 0x84],	/* 13:暗紫 */
    [0x00, 0x84, 0x84],	/* 14:浅暗蓝 */
    [0x84, 0x84, 0x84]	/* 15:暗灰 */
];

const MOUSE_CURSOR: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
    *b"111.............",
    *b"100111..........",
    *b"100000111.......",
    *b".10000000111....",
    *b".1000000000011..",
    *b".10000001111111.",
    *b"..1000001.......",
    *b"..10000001......",
    *b"..100110001.....",
    *b"...101.10001....",
    *b"...101..10001...",
    *b"...101...10001..",
    *b"....11....10001.",
    *b"....11.....10001",
    *b".....1......1001",
    *b".............111"
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    LightRed = 1,
    LightGreen = 2,
    LightYellow = 3,
    LightBlue = 4,
    LightPurple = 5,
    LightCyan = 6,
    White = 7,
    LightGray = 8,
    DarkRed = 9,
    DarkGreen = 10,
    DarkYellow = 11,
    DarkBlue = 12,
    DarkPurple = 13,
    DarkCyan = 14,
    DarkGray = 15,
}

pub struct Screen {
    pub vram: &'static mut u8,
    pub xsize: u16,
    pub ysize: u16,
    pub mouse: [u8; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT]
}

impl Screen {
    pub fn new() -> Screen {
        //在rust中不能采用原书直接将内存地址赋值给结构体指针，然后访问其内部元素的方法，会黑屏
        Screen {
            xsize: 320,
            ysize: 200,
            vram: unsafe { &mut *(0xa0000 as *mut u8) },
            mouse: [Color::DarkCyan as u8; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT]
        }
    }

    //初始化调色板，这样就能用上面enum的色号来绘制图像
    pub fn set_palette(&self) {
        let flags = asm::io_load_flags();
        asm::io_cli();
        asm::io_out8(0x03c8, 0);
        for i in 0..16 {
            asm::io_out8(0x03c9, TABLE_RGB[i][0] / 4);
            asm::io_out8(0x03c9, TABLE_RGB[i][1] / 4);
            asm::io_out8(0x03c9, TABLE_RGB[i][2] / 4);
        }
        asm::io_store_flags(flags);
    }

    //使用上面enum的颜色来填充
    pub fn boxfill8(&mut self, c: Color, x0: usize, y0: usize, x1: usize, y1: usize) {
        for j in y0..=y1 {
            for i in x0..=x1 {
                let ptr = unsafe { &mut *((self.vram as *mut u8).offset((j * self.xsize as usize + i) as isize)) };
                *ptr = c as u8;
            }
        }
    }

    pub fn init_screen(&mut self) {
        let xsize = self.xsize as usize;
        let ysize = self.ysize as usize;
        //绘制桌面背景和任务栏
        self.boxfill8(Color::DarkCyan, 0, 0, xsize - 1, ysize - 29);
        self.boxfill8(Color::LightGray, 0, ysize - 28, xsize - 1, ysize - 28);
        self.boxfill8(Color::White, 0, ysize - 27, xsize - 1, ysize - 27);
        self.boxfill8(Color::LightGray, 0, ysize - 26, xsize - 1, ysize - 1);
        //绘制开始按钮
        self.boxfill8(Color::White, 3, ysize - 24, 59, ysize - 24);
        self.boxfill8(Color::White, 2, ysize - 24, 2, ysize - 4);
        self.boxfill8(Color::DarkGray, 3, ysize - 4, 59, ysize - 4);
        self.boxfill8(Color::DarkGray, 59, ysize - 23, 59, ysize - 5);
        self.boxfill8(Color::Black, 2, ysize - 3, 59, ysize - 3);
        self.boxfill8(Color::Black, 60, ysize - 24, 60, ysize - 3);
        //绘制时间显示区
        self.boxfill8(Color::DarkGray, xsize - 47, ysize - 24, xsize - 4, ysize - 24);
        self.boxfill8(Color::DarkGray, xsize - 47, ysize - 23, xsize - 47, ysize - 4);
        self.boxfill8(Color::White, xsize - 47, ysize - 3, xsize - 4, ysize - 3);
        self.boxfill8(Color::White, xsize - 3, ysize - 24, xsize - 3, ysize - 3);
    }

    pub fn init_mouse_cursor8(&mut self, bc: Color) {
        for j in 0..MOUSE_CURSOR_HEIGHT {
            for i in 0..MOUSE_CURSOR_WIDTH {
                match MOUSE_CURSOR[j][i] {
                    b'1' => self.mouse[j * MOUSE_CURSOR_WIDTH + i] = Color::Black as u8,
                    b'0' => self.mouse[j * MOUSE_CURSOR_WIDTH + i] = Color::White as u8,
                    _ => self.mouse[j * MOUSE_CURSOR_WIDTH + i] = bc as u8
                }
            }
        }
    }

    //显示图像
    pub fn putblock8_8(&mut self, pxsize: usize, pysize: usize, px0: isize, py0: isize, pic: *const u8, bxsize: isize) {
        let vptr = unsafe { self.vram as *mut u8 };
        for j in 0..pysize {
            for i in 0..pxsize {
                let ptr = unsafe { &mut *(vptr.offset((py0 + j as isize) * self.xsize as isize + (px0 + i as isize)))};
                *ptr = unsafe { *pic.offset(bxsize * j as isize + i as isize) };
            }
        }
    }

    pub fn init(&mut self) {
        self.set_palette();
        self.init_screen();
        self.init_mouse_cursor8(Color::DarkCyan);
        self.putblock8_8(MOUSE_CURSOR_WIDTH,
                         MOUSE_CURSOR_HEIGHT,
                         ((self.xsize - 16) / 2) as isize,
                         ((self.ysize - 28 - 16) / 2) as isize,
                         self.mouse.as_ptr(),
                         MOUSE_CURSOR_WIDTH as isize);
    }

    pub fn putfont8(&mut self, x: usize, y: usize, c: Color, chr: char) {
        let fnt = font::FONTS[chr as usize];
        let vptr = unsafe { self.vram as *mut u8 };
        let start_offset = x + y * self.xsize as usize;
        for j in 0..font::FONT_HEIGHT {
            for i in 0..font::FONT_WIDTH {
                if fnt[j][i] {
                    let fnt_offset = j * self.xsize as usize + i;
                    let ptr = unsafe { &mut *(vptr.offset(start_offset as isize + fnt_offset as isize)) };
                    * ptr = c as u8;
                }
            }
        }
    }
}

//实现写入字符串
//不能像原书那样实现，报错需要&str的内存分配函数
pub struct LineWriter {
    init_x: usize,
    x: usize, //当前列
    y: usize, //当前行
    color: Color,
    screen: Screen
}

impl LineWriter {
    pub fn new(sc: Screen, color: Color, x: usize, y: usize) -> LineWriter {
        LineWriter {
            init_x: x,
            x,
            y,
            color,
            screen: sc
        }
    }

    //换行之后x回到起点，y到下一行
    fn new_line(&mut self) {
        self.x = self.init_x;
        self.y = self.y + font::FONT_HEIGHT;
    }

    pub fn set(&mut self, color: Color, new_x: usize, new_y: usize) {
        self.init_x = new_x;
        self.x = new_x;
        self.y = new_y;
        self.color = color;
    }
}

impl fmt::Write for LineWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let str_b = s.as_bytes();
        let height = self.screen.ysize as usize; //屏幕高度
        let width = self.screen.xsize as usize; //屏幕宽度
        for i in 0..str_b.len() {
            if str_b[i] == b'\n' {
                self.new_line();
                return Ok(());
            }

            if self.x + font::FONT_WIDTH < width && self.y + font::FONT_HEIGHT < height {
                self.screen.putfont8(self.x, self.y, self.color, str_b[i] as char);
            } else if self.y + font::FONT_HEIGHT * 2 < height {
                self.new_line();
                self.screen.putfont8(self.x, self.y, self.color, str_b[i] as char);
            }

            //写完之后改变指针位置
            if self.x + font::FONT_WIDTH < width {
                self.x += font::FONT_WIDTH;
            } else if self.y + font::FONT_HEIGHT < height {
                self.new_line();
            } else {
                self.x = width;
                self.y = height;
            }
        }
        Ok(())
    }
}

lazy_static!{
    pub static ref SCREEN: Mutex<Screen> = {
        let mut screen = Screen::new();
        Mutex::new(screen)
    };
}

lazy_static!{
    pub static ref WRITER: Mutex<LineWriter> = {
        let mut writer = LineWriter::new(Screen::new(), Color::White, 0, 0);
        Mutex::new(writer)
    };
}
