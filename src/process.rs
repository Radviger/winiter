use std::ffi::OsString;
use winapi::shared::minwindef::{DWORD, TRUE};
use winapi::um::errhandlingapi::SetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPTHREAD};
use winapi::um::winnt::HANDLE;
use wio::wide::FromWide;
use crate::thread::ThreadIterator;

pub struct ProcessIterator {
    processes_snapshot: HANDLE,
    first: bool,
    entry: PROCESSENTRY32W,
}

impl ProcessIterator {
    pub fn new(flags: DWORD) -> Option<ProcessIterator> {
        let processes_snapshot = unsafe { CreateToolhelp32Snapshot(flags, 0) };
        unsafe { SetLastError(0) };
        if processes_snapshot != INVALID_HANDLE_VALUE {
            Some(ProcessIterator {
                processes_snapshot,
                first: true,
                entry: PROCESSENTRY32W {
                    dwSize: size_of::<PROCESSENTRY32W>() as u32,
                    ..Default::default()
                },
            })
        } else {
            None
        }
    }
}

impl Iterator for ProcessIterator {
    type Item = ProcessEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            if unsafe { Process32FirstW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ProcessEntry::from(self.entry.clone()));
            }
        } else {
            while unsafe { Process32NextW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ProcessEntry::from(self.entry.clone()));
            }
        }
        None
    }
}

impl Drop for ProcessIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.processes_snapshot) };
    }
}


pub struct ProcessEntry {
    inner: PROCESSENTRY32W,
}

impl From<PROCESSENTRY32W> for ProcessEntry {
    fn from(inner: PROCESSENTRY32W) -> Self {
        ProcessEntry { inner }
    }
}

impl ProcessEntry {
    pub fn get_name(&self) -> OsString {
        OsString::from_wide_null(&self.inner.szExeFile)
    }

    pub fn get_pid(&self) -> u32 {
        self.inner.th32ProcessID
    }

    pub fn get_threads(&self) -> Option<ThreadIterator> {
        ThreadIterator::new(TH32CS_SNAPTHREAD, self.get_pid())
    }
}