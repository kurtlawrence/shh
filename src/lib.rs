//! (Rust) Silence stderr and stdout, optionally rerouting it.
//!
//! # Stdout Gagging
//! ```rust
//! println!("STDOUT GAGGING", );
//! println!("you will see this");
//! let shh = shh::stdout().unwrap();
//! println!("but not this");
//! drop(shh);
//! println!("and this");
//! ```
//!
//! # Stderr Gagging
//! ```rust
//! println!("STDERR GAGGING", );
//! eprintln!("you will see this");
//! let shh = shh::stderr().unwrap();
//! eprintln!("but not this");
//! drop(shh);
//! eprintln!("and this");
//! ```
//!
//! # Redirecting Example
//! ```rust
//! println!("REDIRECTING", );
//! use std::io::{Read, Write};
//!
//! std::thread::spawn(move || {
//! 	let mut shh = shh::stdout().unwrap();
//! 	let mut stderr = std::io::stderr();
//! 	loop {
//! 		let mut buf = Vec::new();
//! 		shh.read_to_end(&mut buf).unwrap();
//! 		stderr.write_all(&buf).unwrap();
//! 	}
//! });
//!
//! println!("This should be printed on stderr");
//! eprintln!("This will be printed on stderr as well");
//!
//! // This will exit and close the spawned thread.
//! // In most cases you will want to setup a channel and send a break signal to the loop,
//! // and then join the thread back into it once you are finished.
//! ```
//!
//! # Scoping
//!
//! The struct `Shh` implements the `Drop` trait. Upon going out of scope, the redirection is reset and resources are cleaned up. A `Shh` will only last for the scope, and where no local variable is used, the silencing will not work.
//!
//! ## Example - Silencing Dropped Early
//! ```rust
//! println!("you will see this");
//! shh::stdout().unwrap();		// Shh struct is created, and dropped, here
//! println!("and expect not to see this");
//! ```
//!
//! To fix this, just assign a local variable
//! ```rust
//! println!("you will see this");
//! let shh = shh::stdout().unwrap();		// Shh struct is created here
//! println!("and expect not to see this");
//! drop(shh);	// and dropped here
//! println!("now it works!");
//! ```

#![warn(missing_docs)]

// WINDOWS API ////////////////////////////////////////////////////
#[cfg(windows)]
mod windows;

/// My pet name for the unix Fd or windows Handle types.
#[cfg(windows)]
type Fdandle = std::os::windows::io::RawHandle;

#[cfg(windows)]
pub type ShhStdio = Shh<windows::Impl, io::Stdout>;

#[cfg(windows)]
pub type ShhStderr = Shh<windows::Impl, io::Stderr>;

/// Silence and redirect the stdout stream.
///
/// `Shh` implements `io::Read`, with all captured output able to be read back out.
///
/// # Example
/// ```rust
/// #![cfg(windows)]
/// println!("you will see this");
/// let shh = shh::stdout().unwrap();
/// println!("but not this");
/// drop(shh);
/// println!("and this");
/// ```
#[cfg(windows)]
pub fn stdout() -> io::Result<ShhStdio> {
	Shh::new()
}

/// Silence and redirect the stderr stream.
///
/// `Shh` implements `io::Read`, with all captured output able to be read back out.
///
/// # Example
/// ```rust
/// #![cfg(windows)]
/// eprintln!("you will see this");
/// let shh = shh::stderr().unwrap();
/// eprintln!("but not this");
/// drop(shh);
/// eprintln!("and this");
/// ```
#[cfg(windows)]
pub fn stderr() -> io::Result<ShhStderr> {
	Shh::new()
}

///////////////////////////////////////////////////////////////////

// UNIX API ///////////////////////////////////////////////////////
#[cfg(unix)]
mod unix;

/// My pet name for the unix Fd or windows Handle types.
#[cfg(unix)]
type Fdandle = std::os::unix::io::RawFd;

/// Gags the stdout stream.
///
/// # Example
/// ```rust
/// #![cfg(unix)]
/// println!("you will see this");
/// let shh = shh::stdout().unwrap();
/// println!("but not this");
/// drop(shh);
/// println!("and this");
/// ```
#[cfg(unix)]
pub fn stdout() -> io::Result<Ssh<unix::UnixImpl, io::Stdout>> {
	Ssh::new()
}

/// Gags the stderr stream.
///
/// # Example
/// ```rust
/// #![cfg(unix)]
/// eprintln!("you will see this");
/// let shh = shh::stderr().unwrap();
/// eprintln!("but not this");
/// drop(shh);
/// eprintln!("and this");
/// ```
#[cfg(unix)]
pub fn stderr() -> io::Result<Ssh<unix::UnixImpl, io::Stderr>> {
	Ssh::new()
}

///////////////////////////////////////////////////////////////////

use std::fs::File;
use std::io::{self, Read};
use std::marker::PhantomData;

pub trait Close {
	fn close_resource(&self);
}

trait Create {
	/// Create the read and write handles, taking ownership with `File`.
	/// Return in (read_file, write_file)
	fn create_resource() -> io::Result<(File, File)>;
}

pub trait Divert<D> {
	fn divert_std_stream(&self) -> io::Result<()>;
	fn reinstate_std_stream(original_fdandle: Fdandle) -> io::Result<()>;
}

trait Device {
	fn obtain_original() -> io::Result<Fdandle>;
}

/// A structure holding the redirection data.
///
/// `Shh` implements `io::Read`, with all captured output able to be read back out.
pub struct Shh<D> {
	original: Fdandle,
	write_file: File,
	read_file: File,
	device_mrker: PhantomData<D>,
}

impl<D> Shh<D>
where
	D: Device,
{
	fn new<T: Create>() -> io::Result<Self> {
		// obtain current ptr to device
		let original = <D as Device>::obtain_original()?;

		// create data (a pipe or file) that can give a Fdandle to redirect to, and be closed
		let (read_file, write_file) = Create::create_resource()?;

		// redirect the device to the write fdandle
		inner.divert_std_stream()?;

		Ok(Shh {
			original,
			read_file,
			write_file,
			device_mrker: PhantomData,
		})
	}
}

impl<T: Close + Divert<D>, D> Drop for Shh<T, D> {
	fn drop(&mut self) {
		Divert::reinstate_std_stream(self.original);
	}
}

impl<D> Read for Shh<D> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.read_file.read(buf)
	}
}

/// We can say `Shh` is safe as the `Fdandle`s are meant to last until we close them.
/// As this is done on the `drop` of `Shh`, the handles should live for `Shh`s lifetime.
unsafe impl<T: Divert<D> + Close, D> Send for Shh<T, D> {}
