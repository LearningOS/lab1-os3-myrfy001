use crate::task::suspend_current_and_run_next;
use crate::task::exit_current_and_run_next;
use crate::timer::{get_time_us, get_time};
use crate::task::{TaskStatus, get_cur_task_info};

use crate::config::MAX_SYSCALL_NUM;

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}



pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    unsafe{
        (*ts).sec = us / 1000000;
        (*ts).usec = us % 10000000;
    }
    0
}

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let cur_tcb = get_cur_task_info();
    unsafe {
        *ti = TaskInfo{
            status: cur_tcb.task_status,
            syscall_times: cur_tcb.stats.syscall_times.clone(),
            time: (get_time_us() - cur_tcb.stats.first_run_time) / 1000,
        };
    }
    0
}

#[repr(C)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[repr(C)]
pub struct TaskInfo {
    status: TaskStatus,
    syscall_times: [u32; MAX_SYSCALL_NUM],
    time: usize
}