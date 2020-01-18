use super::*;

use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use winapi;
use winapi::ctypes::c_void;
use winapi::shared::{
    minwindef::{DWORD, LPDWORD},
    ntdef::NULL,
};
use winapi::um::{
    handleapi::INVALID_HANDLE_VALUE,
    minwinbase::{OVERLAPPED, SECURITY_ATTRIBUTES},
    winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
    winnt::HANDLE,
};

pub struct Impl;

impl Create for Impl {
    fn create_files() -> io::Result<(File, File)> {
        let mut read_handle: HANDLE = NULL;
        let mut write_handle: HANDLE = NULL;

        let create_pipe_result = unsafe {
            winapi::um::namedpipeapi::CreatePipe(
                &mut read_handle,
                &mut write_handle,
                NULL as *mut SECURITY_ATTRIBUTES,
                0, // default buffer size
            )
        };

        match create_pipe_result {
            0 => Err(io::Error::last_os_error()),
            _ => Ok((
                unsafe { File::from_raw_handle(read_handle as Fdandle) },
                unsafe { File::from_raw_handle(write_handle as Fdandle) },
            )),
        }
    }
}

impl Divert<io::Stdout> for Impl {
    fn divert_std_stream(write_file: &File) -> io::Result<()> {
        set_std_handle(STD_OUTPUT_HANDLE, write_file.as_raw_handle())
    }

    fn reinstate_std_stream(orignal_handle: Fdandle) -> io::Result<()> {
        set_std_handle(STD_OUTPUT_HANDLE, orignal_handle)
    }
}

impl Divert<io::Stderr> for Impl {
    fn divert_std_stream(write_file: &File) -> io::Result<()> {
        set_std_handle(STD_ERROR_HANDLE, write_file.as_raw_handle())
    }

    fn reinstate_std_stream(orignal_handle: Fdandle) -> io::Result<()> {
        set_std_handle(STD_ERROR_HANDLE, orignal_handle)
    }
}

impl Device for io::Stdout {
    fn obtain_original() -> io::Result<Fdandle> {
        get_std_handle(STD_OUTPUT_HANDLE)
    }
}

impl Device for io::Stderr {
    fn obtain_original() -> io::Result<Fdandle> {
        get_std_handle(STD_ERROR_HANDLE)
    }
}

impl ShhRead for Impl {
    fn shh_read(read_file: &File, buf: &mut [u8]) -> io::Result<usize> {
        let read_handle = read_file.as_raw_handle();

        if !has_bytes(read_handle)? {
            return Ok(0);
        }

        let buf_len: DWORD = buf.len() as DWORD;
        let mut bytes_read: DWORD = 0;

        let read_result = unsafe {
            winapi::um::fileapi::ReadFile(
                read_handle as HANDLE,
                buf.as_mut_ptr() as *mut c_void,
                buf_len,
                &mut bytes_read,
                NULL as *mut OVERLAPPED,
            )
        };

        match read_result {
            0 => Err(io::Error::last_os_error()),
            _ => Ok(bytes_read as usize),
        }
    }
}

fn get_std_handle(device: DWORD) -> io::Result<Fdandle> {
    match unsafe { winapi::um::processenv::GetStdHandle(device) } {
        INVALID_HANDLE_VALUE => Err(io::Error::last_os_error()),
        handle => Ok(handle as Fdandle),
    }
}

fn set_std_handle(device: DWORD, handle: Fdandle) -> io::Result<()> {
    match unsafe { winapi::um::processenv::SetStdHandle(device, handle as HANDLE) } {
        0 => Err(io::Error::last_os_error()),
        _ => Ok(()),
    }
}

/// Uses PeekNamedPipe and checks TotalBytesAvail
fn has_bytes(handle: Fdandle) -> io::Result<bool> {
    let mut bytes_avail: DWORD = 0;

    let result = unsafe {
        winapi::um::namedpipeapi::PeekNamedPipe(
            handle as HANDLE,
            NULL as *mut c_void,
            0,
            NULL as LPDWORD,
            &mut bytes_avail,
            NULL as LPDWORD,
        )
    };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(bytes_avail > 0)
}
