use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPARAM, TRUE};
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::SetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32};
use winapi::um::winnt::HANDLE;
use winapi::um::winuser::EnumThreadWindows;

pub struct ThreadIterator {
    processes_snapshot: HANDLE,
    first: bool,
    entry: THREADENTRY32,
}

impl ThreadIterator {
    pub fn new(flags: DWORD, process: u32) -> Option<ThreadIterator> {
        let thread_snapshot = unsafe { CreateToolhelp32Snapshot(flags, process) };
        unsafe { SetLastError(0) };
        if thread_snapshot != INVALID_HANDLE_VALUE {
            Some(ThreadIterator {
                processes_snapshot: thread_snapshot,
                first: true,
                entry: THREADENTRY32 {
                    dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                    ..Default::default()
                },
            })
        } else {
            None
        }
    }
}

impl Iterator for ThreadIterator {
    type Item = ThreadEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            if unsafe { Thread32First(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ThreadEntry::from(self.entry.clone()));
            }
        } else {
            while unsafe { Thread32Next(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ThreadEntry::from(self.entry.clone()));
            }
        }
        None
    }
}

impl Drop for ThreadIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.processes_snapshot) };
    }
}


pub struct ThreadEntry {
    inner: THREADENTRY32,
}

impl From<THREADENTRY32> for ThreadEntry {
    fn from(inner: THREADENTRY32) -> Self {
        ThreadEntry { inner }
    }
}

impl ThreadEntry {
    fn get_id(&self) -> u32 {
        self.inner.th32ThreadID
    }

    fn get_pid(&self) -> u32 {
        self.inner.th32OwnerProcessID
    }

    fn enumerate_windows(&self, cb: &mut dyn FnMut(HWND) -> bool) -> bool {
        extern "system" fn callback(hwnd: HWND, l_param: LPARAM) -> BOOL {
            let ptr: Box<&mut dyn FnMut(HWND) -> bool> = unsafe { std::mem::transmute(l_param) };
            let result = match {
                (**ptr)(hwnd)
            } {
                true => TRUE,
                false => FALSE,
            };
            std::mem::forget(ptr);
            result
        }

        TRUE == unsafe { EnumThreadWindows(self.get_id(), Some(callback), std::mem::transmute(Box::new(cb))) }
    }
}