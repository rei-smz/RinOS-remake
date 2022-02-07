use alloc::vec::Vec;
use vga::colors::Color16;
use crate::LineWriter;
use crate::vga::boxfill;

const CLOSE_BUTTON: [&[u8; 16]; 14] = [
    b"OOOOOOOOOOOOOOO@",
    b"OQQQQQQQQQQQQQ$@",
    b"OQQQQQQQQQQQQQ$@",
    b"OQQQ@@QQQQ@@QQ$@",
    b"OQQQQ@@QQ@@QQQ$@",
    b"OQQQQQ@@@@QQQQ$@",
    b"OQQQQQQ@@QQQQQ$@",
    b"OQQQQQ@@@@QQQQ$@",
    b"OQQQQ@@QQ@@QQQ$@",
    b"OQQQ@@QQQQ@@QQ$@",
    b"OQQQQQQQQQQQQQ$@",
    b"OQQQQQQQQQQQQQ$@",
    b"O$$$$$$$$$$$$$$@",
    b"@@@@@@@@@@@@@@@@",
];

pub fn make_window(buf: &mut Vec<Color16>, xsize: usize, ysize: usize, caption: &str) {
    boxfill(buf, Color16::LightGrey, 0, 0, xsize - 1, 0, xsize);
    boxfill(buf, Color16::White, 1, 1, xsize - 2, 1, xsize);
    boxfill(buf, Color16::LightGrey, 0, 0, 0, ysize - 1, xsize);
    boxfill(buf, Color16::White, 1, 1, 1, ysize - 2, xsize);
    boxfill(buf, Color16::LightGrey, xsize - 2, 1, xsize - 2, ysize - 2, xsize);
    boxfill(buf, Color16::Black, xsize - 1, 0, xsize - 1, ysize - 1, xsize);
    boxfill(buf, Color16::LightGrey, 2, 2, xsize - 3, ysize - 3, xsize);
    boxfill(buf, Color16::Blue, 3, 3, xsize - 4, 20, xsize);
    boxfill(buf, Color16::DarkGrey, 1, ysize - 2, xsize - 2, ysize - 2, xsize);
    boxfill(buf, Color16::Black, 0, ysize - 1, xsize - 1, ysize - 1, xsize);
    let mut writer = LineWriter::new(Color16::White, 24, 4, xsize, ysize);
    writer.write_str(caption, buf);

    for j in 0..14 as usize {
        for i in 0..16 as usize {
            match CLOSE_BUTTON[j][i] {
                b'@' => buf[(j + 5) * xsize + (xsize - 21 + i)] = Color16::Black,
                b'$' => buf[(j + 5) * xsize + (xsize - 21 + i)] = Color16::DarkGrey,
                b'Q' => buf[(j + 5) * xsize + (xsize - 21 + i)] = Color16::LightGrey,
                _ => buf[(j + 5) * xsize + (xsize - 21 + i)] = Color16::White,
            }
        }
    }
}