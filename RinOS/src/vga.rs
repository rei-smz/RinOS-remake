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

pub fn boxfill(buf: &mut Vec<Color16>, c: Color16, x0: usize, y0: usize, x1: usize, y1: usize) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            // let ptr = unsafe { &mut *((buf as isize + y * self.xsize + x) as *mut Color16) };
            // * ptr = c;
            buf[y * SCREEN_WIDTH + x] = c;
        }
    }

    serial_println!("[boxfill] x0: {}, y0: {}, x1: {}, y1: {}", x0, y0, x1, y1);
}

pub fn putfont(buf: &mut Vec<Color16>, x: usize, y: usize, c: Color16, chr: char) {
    let fnt = font::FONTS[chr as usize];
    let offset = y * SCREEN_WIDTH as usize + x;
    for j in 0..font::FONT_HEIGHT {
        for i in 0..font::FONT_WIDTH {
            if fnt[j][i] {
                unsafe {
                    let cell = j * SCREEN_WIDTH as usize + i;
                    // let ptr = unsafe { &mut *((buf + cell + offset) as *mut Color16) };
                    // *ptr = c;
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
    boxfill(buf, Color16::Cyan, 0, 0, xsize - 1, ysize - 29);
    //VGA.lock().clear_screen(Color16::Cyan);
    boxfill(buf, Color16::LightGrey, 0, ysize - 28, xsize - 1, ysize - 28);
    boxfill(buf, Color16::White, 0, ysize - 27, xsize - 1, ysize - 27);
    boxfill(buf, Color16::LightGrey, 0, ysize - 26, xsize - 1, ysize - 1);
    //绘制开始按钮
    boxfill(buf, Color16::White, 3, ysize - 24, 59, ysize - 24);
    boxfill(buf, Color16::White, 2, ysize - 24, 2, ysize - 4);
    boxfill(buf, Color16::DarkGrey, 3, ysize - 4, 59, ysize - 4);
    boxfill(buf, Color16::DarkGrey, 59, ysize - 23, 59, ysize - 5);
    boxfill(buf, Color16::Black, 2, ysize - 3, 59, ysize - 3);
    boxfill(buf, Color16::Black, 60, ysize - 24, 60, ysize - 3);
    //绘制时间显示区
    boxfill(buf, Color16::DarkGrey, xsize - 47, ysize - 24, xsize - 4, ysize - 24);
    boxfill(buf, Color16::DarkGrey, xsize - 47, ysize - 23, xsize - 47, ysize - 4);
    boxfill(buf, Color16::White, xsize - 47, ysize - 3, xsize - 4, ysize - 3);
    boxfill(buf, Color16::White, xsize - 3, ysize - 24, xsize - 3, ysize - 3);
}

pub fn hide_mouse_cursor(bg_index: usize) {
    LAYERCTL.lock().refresh(bg_index, 32, 0, 32 + 15 * 8 , 16);
}

pub fn update_mouse_cursor(mouse_layer_index: usize, dx: isize, dy: isize) {
    LAYERCTL.lock().slide_by_diff(mouse_layer_index, dx, dy, MOUSE_CURSOR_WIDTH as isize, MOUSE_CURSOR_HEIGHT as isize);
}

// pub struct Screen {
//     mouse: Vec<Color16>,
//     background: Vec<Color16>,
//     mouse_x: isize,
//     mouse_y: isize,
//     mouse_layer_index: usize,
//     bg_layer_index: usize,
// }
//
// impl Screen {
//     pub fn new() -> Screen {
//         //在rust中不能采用原书直接将内存地址赋值给结构体指针，然后访问其内部元素的方法，会黑屏
//         Screen {
//             mouse: vec![Color16::Cyan; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT],
//             background: vec![Color16::Cyan; 640 * 480],
//             mouse_x: (640 - 16) / 2,
//             mouse_y: (480 - 28 - 16) / 2,
//             mouse_layer_index: 0,
//             bg_layer_index: 0
//         }
//     }
//
//     //使用上面enum的颜色来填充
//
//
//     // //显示图像
//     // pub fn putblock8_8(&mut self, buf: *mut Color16, pxsize: usize, pysize: usize, px0: usize, py0: usize, pic: *const Color16, bxsize: isize) {
//     //     for j in 0..pysize {
//     //         for i in 0..pxsize {
//     //             let color = unsafe { *pic.offset(bxsize * j as isize + i as isize)};
//     //             self.mode.set_pixel(px0 + i, py0 + j, color);
//     //         }
//     //     }
//     // }
//
//
//     pub fn init(&mut self) {
//         self.mode.set_mode();
//         self.init_mouse_cursor8(Color16::Cyan);
//         self.bg_layer_index = self.layerctl.alloc().unwrap();
//         self.mouse_layer_index = self.layerctl.alloc().unwrap();
//         self.layerctl.set_buf(
//             self.bg_layer_index,
//             &mut self.background,
//             self.xsize as usize,
//             self.ysize as usize,
//             None
//         );
//         self.layerctl.set_buf(
//             self.mouse_layer_index,
//             &mut self.mouse,
//             MOUSE_CURSOR_WIDTH,
//             MOUSE_CURSOR_HEIGHT,
//             Some(Color16::Cyan)
//         );
//         self.init_screen();
//         self.slide_layer(self.mouse_layer_index, self.mouse_x as usize, self.mouse_y as usize);
//         self.up_down(self.bg_layer_index, Some(0));
//         self.up_down(self.mouse_layer_index, Some(1));
//     }
// }

//实现写入字符串
//不能像原书那样实现，报错需要&str的内存分配函数
pub struct LineWriter<'a> {
    init_x: usize,
    x: usize, //当前列
    y: usize, //当前行
    color: Color16,
    buf: &'a mut Vec<Color16>
}

impl LineWriter<'_> {
    pub fn new(buf: &mut Vec<Color16>, color: Color16, x: usize, y: usize) -> LineWriter {
        LineWriter {
            init_x: x,
            x,
            y,
            color,
            buf
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
}

impl fmt::Write for LineWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let str_b = s.as_bytes();
        let height = SCREEN_HEIGHT; //屏幕高度
        let width = SCREEN_WIDTH; //屏幕宽度
        for i in 0..str_b.len() {
            if str_b[i] == b'\n' {
                self.new_line();
                return Ok(());
            }

            if self.x + font::FONT_WIDTH < width && self.y + font::FONT_HEIGHT < height {
                putfont(self.buf, self.x, self.y, self.color, str_b[i] as char);
            } else if self.y + font::FONT_HEIGHT * 2 < height {
                self.new_line();
                putfont(self.buf, self.x, self.y, self.color, str_b[i] as char);
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

// lazy_static!{
//     pub static ref SCREEN: Mutex<Screen> = {
//         let mut screen = Screen::new();
//         Mutex::new(screen)
//     };
// }

// lazy_static!{
//     pub static ref WRITER: Mutex<LineWriter> = {
//         let mut writer = LineWriter::new(&mut SCREEN.lock().background, Screen::new(), Color16::White, 0, 0);
//         Mutex::new(writer)
//     };
// }
