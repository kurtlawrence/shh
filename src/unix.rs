use super::*;

use libc;
use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};

pub struct Impl;

impl Create for Impl {
    fn create_files() -> io::Result<(File, File)> {
        let mut outputs = [0; 2];

        let create_pipe_result = unsafe { libc::pipe(outputs.as_mut_ptr()) };

		let read_fd = outputs[0];
		let write_fd = outputs[1];

        let read_file = unsafe { FromRawFd::from_raw_fd(read_fd) };
        let write_file = unsafe { FromRawFd::from_raw_fd(write_fd) };

        match create_pipe_result {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok((read_file, write_file)),
        }
    }
}

impl Divert<io::Stdout> for Impl {
    fn divert_std_stream(write_file: &File) -> io::Result<()> {
        set_std_fd(libc::STDOUT_FILENO, write_file.as_raw_fd())
    }

    fn reinstate_std_stream(original_fd: Fdandle) -> io::Result<()> {
        set_std_fd(libc::STDOUT_FILENO, original_fd)
    }
}

impl Divert<io::Stderr> for Impl {
    fn divert_std_stream(write_file: &File) -> io::Result<()> {
        set_std_fd(libc::STDERR_FILENO, write_file.as_raw_fd())
    }

    fn reinstate_std_stream(original_fd: Fdandle) -> io::Result<()> {
        set_std_fd(libc::STDERR_FILENO, original_fd)
    }
}

impl Device for io::Stdout {
    fn obtain_original() -> io::Result<Fdandle> {
        get_std_handle(libc::STDOUT_FILENO)
    }
}

impl Device for io::Stderr {
    fn obtain_original() -> io::Result<Fdandle> {
        get_std_handle(libc::STDERR_FILENO)
    }
}

impl ShhRead for Impl {
	fn shh_read(mut read_file: &File, buf: &mut [u8]) -> io::Result<usize> {
		let avail = 0;

		let r = unsafe { libc::ioctl(read_file.as_raw_fd(), libc::FIONREAD) };

		if r == -1 {
			 Err(io::Error::last_os_error())
		} else if avail == 0 {
			 Ok(0)
		} else {
			read_file.read(buf)
		}
	}
}


fn get_std_handle(device: Fdandle) -> io::Result<Fdandle> {
    match unsafe { libc::dup(device) } {
        -1 => Err(io::Error::last_os_error()),
        handle => Ok(handle),
    }
}

fn set_std_fd(device: Fdandle, fd: Fdandle) -> io::Result<()> {
    match unsafe { libc::dup2(fd, device) } {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(()),
    }
}
