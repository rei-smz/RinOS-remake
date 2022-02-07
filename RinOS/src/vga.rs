use alloc::vec;
use alloc::vec::Vec;
use core::cmp::min;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

#[allow(dead_code)]
use crate::font;
use crate::{MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR_WIDTH, serial_print, serial_println};
use crate::layer::LAYERCTL;

pub(crate) const SCREEN_WIDTH: usize = 640;
pub(crate) const SCREEN_HEIGHT: usize = 480;

lazy_static! {
    pub static ref VGA: Mutex<Graphics640x480x16> = {
        Mutex::new(Graphics640x480x16::new())
    };
}

pub fn boxfill(buf: &mut Vec<Color16>, c: Color16, x0: usize, y0: usize, x1: usize, y1: usize, xsize: usize) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            buf[y * xsize + x] = c;
        }
    }
}

pub fn putfont(buf: &mut Vec<Color16>, x: usize, y: usize, c: Color16, chr: char, xsize: usize) {
    let fnt = font::FONTS[chr as usize];
    let offset = y * xsize as usize + x;
    for j in 0..font::FONT_HEIGHT {
        for i in 0..font::FONT_WIDTH {
            if fnt[j][i] {
                unsafe {
                    let cell = j * xsize as usize + i;
                    buf[cell + offset] = c;
                }
            }
        }
    }
}

pub fn init_screen(buf: &mut Vec<Color16>) {
    let xsize = SCREEN_WIDTH;
    let ysize = SCREEN_HEIGHT;
    //绘制桌面背景和任务栏
    boxfill(buf, Color16::Cyan, 0, 0, xsize - 1, ysize - 29, xsize);
    //VGA.lock().clear_screen(Color16::Cyan);
    boxfill(buf, Color16::LightGrey, 0, ysize - 28, xsize - 1, ysize - 28, xsize);
    boxfill(buf, Color16::White, 0, ysize - 27, xsize - 1, ysize - 27, xsize);
    boxfill(buf, Color16::LightGrey, 0, ysize - 26, xsize - 1, ysize - 1, xsize);
    //绘制开始按钮
    boxfill(buf, Color16::White, 3, ysize - 24, 59, ysize - 24, xsize);
    boxfill(buf, Color16::White, 2, ysize - 24, 2, ysize - 4, xsize);
    boxfill(buf, Color16::DarkGrey, 3, ysize - 4, 59, ysize - 4, xsize);
    boxfill(buf, Color16::DarkGrey, 59, ysize - 23, 59, ysize - 5, xsize);
    boxfill(buf, Color16::Black, 2, ysize - 3, 59, ysize - 3, xsize);
    boxfill(buf, Color16::Black, 60, ysize - 24, 60, ysize - 3, xsize);
    //绘制时间显示区
    boxfill(buf, Color16::DarkGrey, xsize - 47, ysize - 24, xsize - 4, ysize - 24, xsize);
    boxfill(buf, Color16::DarkGrey, xsize - 47, ysize - 23, xsize - 47, ysize - 4, xsize);
    boxfill(buf, Color16::White, xsize - 47, ysize - 3, xsize - 4, ysize - 3, xsize);
    boxfill(buf, Color16::White, xsize - 3, ysize - 24, xsize - 3, ysize - 3, xsize);
}

pub fn update_mouse_cursor(bg_index: usize, mouse_layer_index: usize, dx: isize, dy: isize) {
    LAYERCTL.lock().refresh(bg_index, 32, 0, 32 + 15 * 8 , 16);
    LAYERCTL.lock().slide_by_diff(mouse_layer_index, dx, dy, MOUSE_CURSOR_WIDTH as isize, MOUSE_CURSOR_HEIGHT as isize);
}

//实现写入字符串
//不能像原书那样实现，报错需要&str的内存分配函数
pub struct LineWriter {
    init_x: usize,
    x: usize, //当前列
    y: usize, //当前行
    xsize: usize,
    ysize: usize,
    color: Color16,
}

impl LineWriter {
    pub fn new(color: Color16, x: usize, y: usize, xsize: usize, ysize: usize) -> LineWriter {
        LineWriter {
            init_x: x,
            x,
            y,
            xsize,
            ysize,
            color
        }
    }

    //换行之后x回到起点，y到下一行
    fn new_line(&mut self) {
        self.x = self.init_x;
        self.y = self.y + font::FONT_HEIGHT;
    }

    pub fn set(&mut self, color: Color16, new_x: usize, new_y: usize) {
        self.init_x = new_x;
        self.x = new_x;
        self.y = new_y;
        self.color = color;
    }

    pub fn write_str(&mut self, s: &str, buf: &mut Vec<Color16>) {
        let str_b = s.as_bytes();
        let height = self.ysize; //屏幕高度
        let width = self.xsize; //屏幕宽度
        for i in 0..str_b.len() {
            if str_b[i] == b'\n' {
                self.new_line();
                continue;
            }

            if self.x + font::FONT_WIDTH <= width && self.y + font::FONT_HEIGHT <= height {
                putfont(buf, self.x, self.y, self.color, str_b[i] as char, self.xsize);
            } else if self.y + font::FONT_HEIGHT * 2 < height {
                self.new_line();
                putfont(buf, self.x, self.y, self.color, str_b[i] as char, self.xsize);
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
    }
}
