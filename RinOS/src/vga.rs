use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use vga::colors::Color16;
use vga::writers::{Graphics640x480x16, GraphicsWriter};

#[allow(dead_code)]
use crate::font;

const MOUSE_CURSOR_WIDTH: usize = 16;
const MOUSE_CURSOR_HEIGHT: usize = 16;
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

pub struct Screen {
    mode: Graphics640x480x16,
    xsize: isize,
    ysize: isize,
    mouse: [Color16; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT],
    mouse_x: isize,
    mouse_y: isize
}

impl Screen {
    pub fn new() -> Screen {
        //在rust中不能采用原书直接将内存地址赋值给结构体指针，然后访问其内部元素的方法，会黑屏
        Screen {
            xsize: 640,
            ysize: 480,
            mode: Graphics640x480x16::new(),
            mouse: [Color16::Cyan; MOUSE_CURSOR_WIDTH * MOUSE_CURSOR_HEIGHT],
            mouse_x: (640 - 16) / 2,
            mouse_y: (480 - 28 - 16) / 2
        }
    }

    //使用上面enum的颜色来填充
    pub fn boxfill8(&mut self, c: Color16, x0: isize, y0: isize, x1: isize, y1: isize) {
        for i in y0..=y1 {
            self.mode.draw_line((x0, i), (x1, i), c);
        }
    }

    pub fn init_screen(&mut self) {
        let xsize = self.xsize;
        let ysize = self.ysize;
        //绘制桌面背景和任务栏
        //self.boxfill8(Color16::Cyan, 0, 0, (xsize - 1) as usize, (ysize - 29) as usize);
        self.mode.clear_screen(Color16::Cyan);
        self.boxfill8(Color16::LightGrey, 0, ysize - 28, xsize - 1, ysize - 28);
        self.boxfill8(Color16::White, 0, ysize - 27, xsize - 1, ysize - 27);
        self.boxfill8(Color16::LightGrey, 0, ysize - 26, xsize - 1, ysize - 1);
        //绘制开始按钮
        self.boxfill8(Color16::White, 3, ysize - 24, 59, ysize - 24);
        self.boxfill8(Color16::White, 2, ysize - 24, 2, ysize - 4);
        self.boxfill8(Color16::DarkGrey, 3, ysize - 4, 59, ysize - 4);
        self.boxfill8(Color16::DarkGrey, 59, ysize - 23, 59, ysize - 5);
        self.boxfill8(Color16::Black, 2, ysize - 3, 59, ysize - 3);
        self.boxfill8(Color16::Black, 60, ysize - 24, 60, ysize - 3);
        //绘制时间显示区
        self.boxfill8(Color16::DarkGrey, xsize - 47, ysize - 24, xsize - 4, ysize - 24);
        self.boxfill8(Color16::DarkGrey, xsize - 47, ysize - 23, xsize - 47, ysize - 4);
        self.boxfill8(Color16::White, xsize - 47, ysize - 3, xsize - 4, ysize - 3);
        self.boxfill8(Color16::White, xsize - 3, ysize - 24, xsize - 3, ysize - 3);
    }

    pub fn init_mouse_cursor8(&mut self, bc: Color16) {
        for j in 0..MOUSE_CURSOR_HEIGHT {
            for i in 0..MOUSE_CURSOR_WIDTH {
                match MOUSE_CURSOR[j][i] {
                    b'1' => self.mouse[j * MOUSE_CURSOR_WIDTH + i] = Color16::Black,
                    b'0' => self.mouse[j * MOUSE_CURSOR_WIDTH + i] = Color16::White,
                    _ => self.mouse[j * MOUSE_CURSOR_WIDTH + i] = bc
                }
            }
        }
    }

    //显示图像
    pub fn putblock8_8(&mut self, pxsize: usize, pysize: usize, px0: usize, py0: usize, pic: *const Color16, bxsize: isize) {
        for j in 0..pysize {
            for i in 0..pxsize {
                let color = unsafe { *pic.offset(bxsize * j as isize + i as isize)};
                self.mode.set_pixel(px0 + i, py0 + j, color);
            }
        }
    }

    pub fn init(&mut self) {
        self.mode.set_mode();
        self.init_screen();
        self.init_mouse_cursor8(Color16::Cyan);
        self.putblock8_8(MOUSE_CURSOR_WIDTH,
                         MOUSE_CURSOR_HEIGHT,
                         self.mouse_x as usize,
                         self.mouse_y as usize,
                         self.mouse.as_ptr(),
                         MOUSE_CURSOR_WIDTH as isize);
    }

    pub fn putfont8(&mut self, x: usize, y: usize, c: Color16, chr: char) {
        let fnt = font::FONTS[chr as usize];
        for j in 0..font::FONT_HEIGHT {
            for i in 0..font::FONT_WIDTH {
                if fnt[j][i] {
                    self.mode.set_pixel(x + i, y + j, c);
                }
            }
        }
    }

    pub fn set_mouse_pos(&mut self, dx: isize, dy: isize) {
        self.mouse_x += dx;
        self.mouse_y -= dy;
        if self.mouse_x < 0 {
            self.mouse_x = 0;
        }
        if self.mouse_y < 0 {
            self.mouse_y = 0;
        }
        if self.mouse_x > (self.xsize - 16) as isize {
            self.mouse_x = (self.xsize - 16) as isize;
        }
        if self.mouse_y > (self.ysize - 16) as isize {
            self.mouse_y = (self.ysize - 16) as isize;
        }
    }

    pub fn hide_mouse_cursor(&mut self) {
        self.boxfill8(Color16::Cyan,
                      self.mouse_x,
                      self.mouse_y,
                      self.mouse_x + 15,
                      self.mouse_y + 15);
    }

    pub fn update_mouse_cursor(&mut self) {
        self.putblock8_8(MOUSE_CURSOR_WIDTH,
                         MOUSE_CURSOR_HEIGHT,
                         self.mouse_x as usize,
                         self.mouse_y as usize,
                         self.mouse.as_ptr(),
                         MOUSE_CURSOR_WIDTH as isize);
    }
}

//实现写入字符串
//不能像原书那样实现，报错需要&str的内存分配函数
pub struct LineWriter {
    init_x: usize,
    x: usize, //当前列
    y: usize, //当前行
    color: Color16,
    screen: Screen
}

impl LineWriter {
    pub fn new(sc: Screen, color: Color16, x: usize, y: usize) -> LineWriter {
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

    pub fn set(&mut self, color: Color16, new_x: usize, new_y: usize) {
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
        let mut writer = LineWriter::new(Screen::new(), Color16::White, 0, 0);
        Mutex::new(writer)
    };
}
