
use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

#[derive(Copy, Clone, Debug)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub stats: TaskStatsInfo,
}

#[derive(Copy, Clone, Debug)]
pub struct TaskStatsInfo {
    pub first_run_time: usize,
    pub syscall_times: [u32; MAX_SYSCALL_NUM], 
}

impl Default for TaskStatsInfo {
    fn default() -> Self {
        Self {
            first_run_time: 0, 
            syscall_times: [0; MAX_SYSCALL_NUM],
        }
    }
}