mod switch;
mod task;
mod context;


use lazy_static::lazy_static;

pub use context::TaskContext;

use self::switch::__switch;
use crate::{sync::UPSafeCell, loader::{init_app_cx, get_num_app}};


pub use self::task::{TaskStatus, TaskStatsInfo};

use task::TaskControlBlock;

use crate::config::MAX_APP_NUM;

use crate::timer::get_time_us;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}


impl TaskManager {



    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }


    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();

        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;

            // do statistic
            if inner.tasks[current].stats.first_run_time == 0 {
                inner.tasks[current].stats.first_run_time = get_time_us();
            }
            
            drop(inner);
            unsafe {__switch(current_task_cx_ptr, next_task_cx_ptr);}
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1 .. current + self.num_app + 1)
        .map(|id| id % self.num_app)
        .find(|id| inner.tasks[*id].task_status ==  TaskStatus::Ready)
    }


    fn update_cur_task_syscall_cnt(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].stats.syscall_times[syscall_id] += 1;
        println!("current: {}, syscall: {}, cnt: {}", current, syscall_id, inner.tasks[current].stats.syscall_times[syscall_id]);
    }

    fn get_cur_task_info(&self) -> TaskControlBlock {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let t = inner.tasks[current];
        t
    }

}


pub fn update_cur_task_syscall_cnt(syscall_id: usize) {
    TASK_MANAGER.update_cur_task_syscall_cnt(syscall_id);
}

pub fn get_cur_task_info() -> TaskControlBlock {
    TASK_MANAGER.get_cur_task_info()
}



pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}


pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            stats: TaskStatsInfo::default(),
        }; MAX_APP_NUM];
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}