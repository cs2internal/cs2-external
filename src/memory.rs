use std::mem::{size_of, size_of_val, transmute_copy, zeroed};
use std::ptr::null_mut;
use std::str;
use winapi::um::tlhelp32::{Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE};
use winapi::um::winnt::PROCESS_ALL_ACCESS;
use winapi::{
    shared::{
        minwindef::{LPCVOID, LPVOID},
        ntdef::HANDLE,
    },
    um::{
        handleapi::CloseHandle,
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        processthreadsapi::OpenProcess,
        tlhelp32::{CreateToolhelp32Snapshot, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS},
    },
};

pub struct Memory {
    pub id: u32,
    pub process: HANDLE,
}

impl Memory {
    pub fn new(process_name: &str) -> Self {
        let mut id: u32 = 0;
        let mut process: HANDLE = null_mut();
        unsafe {
            let mut entry: PROCESSENTRY32 = zeroed();
            entry.dwSize = size_of::<PROCESSENTRY32>() as u32;

            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

            while Process32Next(snapshot, &mut entry) == 1 {
                let process_entry_u8: [u8; 260] = transmute_copy(&entry.szExeFile);
                let process_entry = std::str::from_utf8(&process_entry_u8[..]).unwrap();

                if process_entry.starts_with(process_name) {
                    id = entry.th32ProcessID;
                    process = OpenProcess(PROCESS_ALL_ACCESS, 0, id);
                    break;
                }
            }

            CloseHandle(snapshot);
        }
        Self { id, process }
    }

    pub fn get_module_adress(&self, module_name: &str) -> usize {
        unsafe {
            let mut entry: MODULEENTRY32 = zeroed();
            entry.dwSize = size_of::<MODULEENTRY32>() as u32;

            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, self.id);
            while Module32Next(snapshot, &mut entry) == 1 {
                let module_entry_u8: [u8; 256] = transmute_copy(&entry.szModule);
                let module_entry = std::str::from_utf8(&module_entry_u8[..]).unwrap();
                if module_entry.starts_with(module_name) {
                    CloseHandle(snapshot);
                    return entry.modBaseAddr as usize;
                }
            }
        }
        unreachable!("Could not find the module")
    }

    pub fn read<T: Default>(&self, address: usize) -> T {
        let mut buffer: T = Default::default();
        unsafe {
            ReadProcessMemory(
                self.process,
                address as LPCVOID,
                &mut buffer as *mut T as LPVOID,
                size_of_val(&buffer),
                null_mut(),
            );
        }
        buffer
    }

    pub fn write<T: Default>(&self, address: usize, buffer: T) -> bool {
        unsafe {
            WriteProcessMemory(
                self.process,
                address as LPVOID,
                &buffer as *const T as LPVOID,
                size_of_val(&buffer),
                null_mut(),
            ) == 1
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.process) };
    }
}
