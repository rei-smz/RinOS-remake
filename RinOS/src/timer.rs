use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::InterruptStackFrame;
use crate::asm::{io_cli, io_load_flags, io_store_flags};
use crate::fifo::Fifo;
use crate::int::{InterruptIndex, PICS};
use crate::serial_print;

const MAX_TIMER_COUNT: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Available,
    InUse,
    Running,
}

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub timeout: u32,
    pub flag: TimerState,
    pub fifo_addr: usize,
    pub data: u8
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            timeout: 0,
            flag: TimerState::Available,
            fifo_addr: 0,
            data: 0
        }
    }
}

pub struct TimerCtl {
    pub count: u32,
    pub next: u32,
    pub counting: u32,
    pub timers: [usize; MAX_TIMER_COUNT],
    pub timers_data: [Timer; MAX_TIMER_COUNT],
}

impl TimerCtl {
    pub fn new() -> TimerCtl {
        TimerCtl {
            count: 0,
            next: 0,
            counting: 0,
            timers: [0; MAX_TIMER_COUNT],
            timers_data: [Timer::new(); MAX_TIMER_COUNT],
        }
    }

    pub fn alloc(&mut self) -> Result<usize, &'static str> {
        for i in 0..MAX_TIMER_COUNT {
            if self.timers_data[i].flag == TimerState::Available {
                self.timers_data[i].flag = TimerState::InUse;
                return Ok(i);
            }
        }
        Err("No available timer")
    }

    pub fn set_time(&mut self, timer_id: usize, timeout: u32) {
        let mut timer = self.timers_data[timer_id];
        timer.timeout = timeout + self.count;
        timer.flag = TimerState::Running;
        let eflags = io_load_flags();
        io_cli();
        let mut insert_idx: usize = 0;
        for i in 0..self.counting {
            insert_idx = i as usize;
            let t = self.timers_data[self.timers[i as usize]];
            if t.timeout >= timer.timeout {
                break;
            }
        }
        let mut j = self.counting as usize;
        while j > insert_idx {
            self.timers[j] = self.timers[j - 1];
            j -= 1;
        }
        self.counting += 1;
        self.timers[insert_idx] = timer_id;
        self.next = self.timers_data[self.timers[0]].timeout;
        io_store_flags(eflags);
    }

    pub fn set_flag(&mut self, timer_id: usize, flag: TimerState) {
        self.timers_data[timer_id].flag = flag;
    }

    pub fn init_timer(&mut self, timer_id: usize, fifo_addr: &Fifo, data: u8) {
        let mut timer = &mut self.timers_data[timer_id];
        timer.fifo_addr = fifo_addr as *const Fifo as usize;
        timer.data = data;
    }

    pub fn free(&mut self, timer_id: usize) {
        self.timers_data[timer_id].flag = TimerState::Available;
    }
}

lazy_static! {
    pub static ref TIMER_CTL: Mutex<TimerCtl> = Mutex::new(TimerCtl::new());
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    //serial_print!(".");
    interrupts::without_interrupts(|| {
        let mut timer_ctl = TIMER_CTL.lock();
        timer_ctl.count += 1;
        if timer_ctl.next > timer_ctl.count {
            return;
        }
        let mut timeout_cnt = 0;
        for i in 0..timer_ctl.counting {
            timeout_cnt = i;
            let timer_id = timer_ctl.timers[i as usize];
            let timer = timer_ctl.timers_data[timer_id];
            if timer.timeout > timer_ctl.count {
                break;
            }
            timer_ctl.free(timer_id);
            let fifo = unsafe { &*(timer.fifo_addr as *const Fifo) };
            fifo.put(timer.data).unwrap();
        }
        timer_ctl.counting -= timeout_cnt;
        for i in 0..timer_ctl.counting {
            timer_ctl.timers[i as usize] = timer_ctl.timers[i as usize + timeout_cnt as usize];
        }
        if timer_ctl.counting > 0 {
            timer_ctl.next = timer_ctl.timers_data[timer_ctl.timers[0]].timeout;
        } else {
            timer_ctl.next = 0xffffffff;
        }
    });
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}