//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{translated_byte_buffer, MapPermission, VPNRange, VirtAddr};
use crate::task::{
    current_user_token, exit_current_and_run_next, get_task_info, mmap, munmap,
    suspend_current_and_run_next, TaskStatus,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

impl TaskInfo {
    pub fn new(task: (TaskStatus, usize, [u32; MAX_SYSCALL_NUM])) -> TaskInfo {
        TaskInfo {
            status: task.0,
            time: task.1,
            syscall_times: task.2,
        }
    }
}
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    let mut _ts = translated_byte_buffer(
        current_user_token(),
        _ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    let _ts = _ts[0].as_mut_ptr() as *mut TimeVal;
    unsafe {
        *_ts = TimeVal {
            sec: _us / 1_000_000,
            usec: _us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    mmap(_start, _len, _port)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    munmap(_start, _len)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let mut ti = translated_byte_buffer(
        current_user_token(),
        ti as *const u8,
        core::mem::size_of::<TaskInfo>(),
    );
    let ti = ti[0].as_mut_ptr() as *mut TaskInfo;
    let task_info = get_task_info();
    let _us = get_time_us();
    let time = (_us - task_info.time) / 1000;
    unsafe {
        *ti = TaskInfo {
            status: task_info.status,
            syscall_times: task_info.syscall_times,
            time: time,
        };
    }
    0
}
