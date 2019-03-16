use super::*;

use std::io;
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

pub struct Impl {
	write_handle: Fdandle,
	read_handle: Fdandle,
}

impl Create for Impl {
	fn create_resource() -> io::Result<Self> {
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
			_ => Ok(Impl {
				write_handle,
				read_handle,
			}),
		}
	}
}

impl Close for Impl {
	fn close_resource(&self) {
		unsafe {
			winapi::um::handleapi::CloseHandle(self.read_handle);
		}
		unsafe {
			winapi::um::handleapi::CloseHandle(self.write_handle);
		}
	}
}

impl Divert<io::Stdout> for Impl {
	fn divert_std_stream(&self) -> io::Result<()> {
		set_std_handle(STD_OUTPUT_HANDLE, self.write_handle)
	}

	fn reinstate_std_stream(orignal_handle: Fdandle) -> io::Result<()> {
		set_std_handle(STD_OUTPUT_HANDLE, orignal_handle)
	}
}

impl Divert<io::Stderr> for Impl {
	fn divert_std_stream(&self) -> io::Result<()> {
		set_std_handle(STD_ERROR_HANDLE, self.write_handle)
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

impl io::Read for Impl {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if !has_bytes(self.read_handle)? {
			return Ok(0);
		}

		let buf_len: DWORD = buf.len() as DWORD;
		let mut bytes_read: DWORD = 0;

		let read_result = unsafe {
			winapi::um::fileapi::ReadFile(
				self.read_handle,
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

fn get_std_handle(device: DWORD) -> io::Result<HANDLE> {
	match unsafe { winapi::um::processenv::GetStdHandle(device) } {
		INVALID_HANDLE_VALUE => Err(io::Error::last_os_error()),
		handle => Ok(handle),
	}
}

fn set_std_handle(device: DWORD, handle: HANDLE) -> io::Result<()> {
	match unsafe { winapi::um::processenv::SetStdHandle(device, handle) } {
		0 => Err(io::Error::last_os_error()),
		_ => Ok(()),
	}
}

/// Uses PeekNamedPipe and checks TotalBytesAvail
fn has_bytes(handle: HANDLE) -> io::Result<bool> {
	let mut bytes_avail: DWORD = 0;

	let result = unsafe {
		winapi::um::namedpipeapi::PeekNamedPipe(
			handle,
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
