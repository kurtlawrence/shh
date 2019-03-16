use super::*;

use libc;
use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};

pub struct UnixImpl {
    read_file: File,
    write_file: File,
}

impl Create for UnixImpl {
    fn create_resource() -> io::Result<Self> {
        let mut outputs = [0; 2];

        let create_pipe_result = unsafe { libc::pipe(outputs.as_mut_ptr()) };

        let read_file = unsafe { FromRawFd::from_raw_fd(outputs[0]) };
        let write_file = unsafe { FromRawFd::from_raw_fd(outputs[1]) };

        match create_pipe_result {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(UnixImpl {
                read_file,
                write_file,
            }),
        }
    }
}

impl Close for UnixImpl {
    fn close_resource(&self) {
        // we actually don't have to do anything here as `File` will close it.
        // It has taken ownership of the file descriptors.
        // see
        //	- https://doc.rust-lang.org/std/fs/struct.File.html#impl-FromRawFd
        //	- https://doc.rust-lang.org/std/os/unix/io/trait.FromRawFd.html#tymethod.from_raw_fd
    }
}

impl Divert<io::Stdout> for UnixImpl {
    fn divert_std_stream(&self) -> io::Result<()> {
        set_std_fd(libc::STDOUT_FILENO, self.write_file.as_raw_fd())
    }

    fn reinstate_std_stream(original_fd: Fdandle) -> io::Result<()> {
        set_std_fd(libc::STDOUT_FILENO, original_fd)
    }
}

impl Divert<io::Stderr> for UnixImpl {
    fn divert_std_stream(&self) -> io::Result<()> {
        set_std_fd(libc::STDERR_FILENO, self.write_file.as_raw_fd())
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

impl io::Read for UnixImpl {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_file.read(buf)
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

// /// Uses PeekNamedPipe and checks TotalBytesAvail
// fn has_bytes(handle: HANDLE) -> io::Result<bool> {
//     let mut bytes_avail: DWORD = 0;

//     let result = unsafe {
//         winapi::um::namedpipeapi::PeekNamedPipe(
//             handle,
//             NULL as *mut c_void,
//             0,
//             NULL as LPDWORD,
//             &mut bytes_avail,
//             NULL as LPDWORD,
//         )
//     };

//     if result == 0 {
//         return Err(io::Error::last_os_error());
//     }

//     Ok(bytes_avail > 0)
// }
