use alloc::vec::{Vec};
use alloc::vec;
use core::cmp::{max, min};
use lazy_static::lazy_static;
use vga::colors::Color16;
use crate::vga::VGA;
use spin::Mutex;
use vga::writers::GraphicsWriter;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, serial_print, serial_println};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer {
    buf: usize,
    x0: usize,
    y0: usize,
    xsize: usize,
    ysize: usize,
    is_used: bool,
    z: Option<usize>,
    transparent: Option<Color16>
}

impl Layer {
    pub fn new() -> Layer {
        Layer {
            x0: 0,
            y0: 0,
            xsize: 0,
            ysize: 0,
            is_used: false,
            transparent: None,
            z: None,
            buf: 0
        }
    }

    pub fn set(&mut self, buf: &mut Vec<Color16>, xsize: usize, ysize: usize, transparent: Option<Color16>) {
        self.buf = buf.as_mut_ptr() as usize;
        self.xsize = xsize;
        self.ysize = ysize;
        self.transparent = transparent;
    }
}

lazy_static!(
    static ref MAP: Mutex<Vec<u8>> = Mutex::new(vec![0; SCREEN_HEIGHT * SCREEN_WIDTH]);
);

pub struct LayerCtl {
    pub z_max: Option<usize>,
    pub layers: [usize; 256],
    pub layer_data: [Layer; 256],
}

impl LayerCtl {
    pub fn new() -> LayerCtl {
        LayerCtl {
            z_max: None,
            layers: [0; 256],
            layer_data: [Layer::new();256],
        }
    }

    pub fn set_buf(&mut self, layer_index: usize, buf: &mut Vec<Color16>, xsize: usize, ysize: usize, transparent: Option<Color16>) {
        self.layer_data[layer_index].set(buf, xsize, ysize, transparent);
    }

    pub fn alloc(&mut self) -> Option<usize> {
        for i in 0..256 {
            if self.layer_data[i].is_used == false {
                self.layer_data[i].is_used = true;
                self.layer_data[i].z = None;
                return Some(i);
            }
        }
        None
    }

    pub fn refresh_map(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, z0: usize) {
        if self.z_max.is_none() {
            return;
        }
        let x0 = max(0, x0);
        let y0 = max(0, y0);
        let x1 = min(x1, SCREEN_WIDTH);
        let y1 = min(y1, SCREEN_HEIGHT);
        for h in z0..=self.z_max.unwrap() {
            let si = self.layers[h];
            let layer = self.layer_data[si];
            let map = unsafe { layer.buf as *const Color16 };
            let bx0 = if x0 > layer.x0 { x0 - layer.x0 } else { 0 };
            let by0 = if y0 > layer.y0 { y0 - layer.y0 } else { 0 };
            let bx1 = if x1 > layer.x0 { min(x1 - layer.x0, layer.xsize) } else { 0 };
            let by1 = if y1 > layer.y0 { min(y1 - layer.y0, layer.ysize) } else { 0 };
            for by in by0..by1 {
                let vy = layer.y0 + by;
                let width = layer.xsize;
                for bx in bx0..bx1 {
                    let vx = layer.x0 + bx;
                    let c = unsafe { *map.offset((by * width + bx) as isize) };
                    if !layer.transparent.contains(&c) {
                        MAP.lock()[vy * SCREEN_WIDTH + vx] = si as u8;
                    }
                }
            }
        }
    }

    pub fn refresh_part(&self, x0: usize, y0: usize, x1: usize, y1: usize, z0: usize, z1: usize) {
        if self.z_max.is_none() {
            return;
        }
        let x0 = max(0, x0);
        let y0 = max(0, y0);
        let x1 = min(x1, SCREEN_WIDTH);
        let y1 = min(y1, SCREEN_HEIGHT);
        let mut h = z0;
        while h <= z1 {
            let si = self.layers[h];
            let layer = &self.layer_data[si];
            let bx0 = if x0 > layer.x0 { x0 - layer.x0 } else { 0 };
            let by0 = if y0 > layer.y0 { y0 - layer.y0 } else { 0 };
            let bx1 = if x1 > layer.x0 { min(x1 - layer.x0, layer.xsize) } else { 0 };
            let by1 = if y1 > layer.y0 { min(y1 - layer.y0, layer.ysize) } else { 0 };
            let map = unsafe { layer.buf as *const Color16 };
            for by in by0..by1 {
                let vy = layer.y0 + by;
                let width = layer.xsize;
                for bx in bx0..bx1 {
                    let vx = layer.x0 + bx;
                    let map_si = MAP.lock()[vy * SCREEN_WIDTH + vx];
                    if si as u8 == map_si {
                        let c = unsafe { *map.offset((by * width + bx) as isize) };
                        VGA.lock().set_pixel(vx, vy, c);
                    }
                }
            }
            h += 1;
        }
    }

    pub fn up_down(&mut self, layer_index: usize, oz: Option<usize>) {
        let layer = self.layer_data[layer_index];
        let old = layer.z;
        let oz = if let Some(z) = oz {
            Some(
                min(
                    if let Some(z_max) = self.z_max {
                        z_max + 1
                    } else {
                        0
                    },
                    z,
                )
            )
        } else {
            None
        };
        self.layer_data[layer_index].z = oz;
        if old != oz {
            let z0: usize;
            let z1: usize;
            if let Some(o) = old {
                if let Some(z) = oz {
                    if o > z { // down
                        let mut h = o;
                        while h > z {
                            self.layers[h] = self.layers[h - 1];
                            self.layer_data[self.layers[h]].z = Some(h);
                            h -= 1;
                        }
                        self.layers[z] = layer_index;
                        z0 = z;
                        z1 = o;
                    } else if o < z { // up
                        let mut h = o;
                        while h < z {
                            self.layers[h] = self.layers[h + 1];
                            self.layer_data[self.layers[h]].z = Some(h);
                            h += 1;
                        }
                        self.layers[z] = layer_index;
                        z0 = z;
                        z1 = z;
                    } else {
                        return;
                    }
                } else {
                    if let Some(z_max) = self.z_max {
                        if z_max > o {
                            for h in o..z_max {
                                self.layers[h] = self.layers[h + 1];
                                self.layer_data[self.layers[h]].z = Some(h);
                            }
                        }
                        self.layers[z_max + 1] = layer_index;
                        self.z_max = if z_max > 0 { Some(z_max - 1) } else { None };
                    }
                    z0 = 0;
                    z1 = o - 1;
                }
            } else {
                if let Some(z) = oz {
                    let z_max = if let Some(z_max) = self.z_max {
                        z_max
                    } else {
                        0
                    };
                    for h in z..z_max {
                        self.layers[h + 1] = self.layers[h];
                        self.layer_data[self.layers[h + 1]].z = Some(h + 1);
                    }
                    self.layers[z] = layer_index;
                    if let Some(z_max) = self.z_max {
                        self.z_max = Some(z_max + 1);
                    } else {
                        self.z_max = Some(0);
                    }
                    z0 = z;
                    z1 = z;
                } else {
                    return;
                }
            }
            self.refresh_map(layer.x0, layer.y0, layer.x0 + layer.xsize, layer.y0 + layer.ysize, z0);
            self.refresh_part(layer.x0, layer.y0, layer.x0 + layer.xsize, layer.y0 + layer.ysize, z0, z1);
        }
    }

    pub fn refresh(&mut self, layer_index: usize, x0: usize, y0: usize, x1: usize, y1: usize) {
        let layer = self.layer_data[layer_index];
        if let Some(z) = layer.z {
            self.refresh_part(layer.x0 + x0, layer.y0 + y0, layer.x0 + x1, layer.y0 + y1, z, z);
        }
    }

    pub fn slide(&mut self, layer_index: usize, x: usize, y: usize) {
        let layer = self.layer_data[layer_index];
        let old_x = layer.x0;
        let old_y = layer.y0;
        self.layer_data[layer_index].x0 = x;
        self.layer_data[layer_index].y0 = y;
        if let Some(z) = layer.z {
            self.refresh_map(old_x, old_y, old_x + layer.xsize, old_y + layer.ysize, 0);
            self.refresh_map(x, y, x + layer.xsize, y + layer.ysize, z);
            self.refresh_part(old_x, old_y, old_x + layer.xsize, old_y + layer.ysize, 0, z - 1);
            self.refresh_part(x, y, x + layer.xsize, y + layer.ysize, z, z);
        }
    }

    pub fn slide_by_diff(&mut self, layer_index: usize, dx: isize, dy: isize, width: isize, height: isize) {
        let layer = self.layer_data[layer_index];
        let mut new_x = layer.x0 as isize + dx;
        let mut new_y = layer.y0 as isize + dy;
        let x_max = SCREEN_WIDTH as isize - 1;
        let y_max = SCREEN_HEIGHT as isize - 1;
        if new_x < 0 {
            new_x = 0;
        } else if new_x > x_max {
            new_x = x_max;
        }
        if new_y < 0 {
            new_y = 0;
        } else if new_y > y_max {
            new_y = y_max;
        }
        self.slide(layer_index, new_x as usize, new_y as usize);
    }

    pub fn free(&mut self, layer_index: usize) {
        let layer = self.layer_data[layer_index];
        if layer.z.is_some() {
            self.up_down(layer_index, None);
        }
        self.layer_data[layer_index].is_used = false;
    }
}

lazy_static! {
    pub static ref LAYERCTL: Mutex<LayerCtl> = {
        Mutex::new(LayerCtl::new())
    };
}

lazy_static!(
    pub static ref mouse_layer_index: Mutex<usize> = Mutex::new(0);
    pub static ref bg_layer_index: Mutex<usize> = Mutex::new(0);
    pub static ref win_layer_index: Mutex<usize> = Mutex::new(0);
);
