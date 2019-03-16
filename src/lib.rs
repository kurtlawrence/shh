#![warn(missing_docs)]

// WINDOWS API ////////////////////////////////////////////////////
#[cfg(windows)]
mod windows;

/// My pet name for the unix Fd or windows Handle types.
#[cfg(windows)]
type Fdandle = std::os::windows::io::RawHandle;

/// Gags the stdout stream.
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
pub fn stdout() -> io::Result<Ssh<windows::Impl, io::Stdout>> {
	Ssh::new()
}

/// Gags the stderr stream.
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
pub fn stderr() -> io::Result<Ssh<windows::Impl, io::Stderr>> {
	Ssh::new()
}

///////////////////////////////////////////////////////////////////

// UNIX API ///////////////////////////////////////////////////////
#[cfg(unix)]
mod unix;

/// My pet name for the unix Fd or windows Handle types.
#[cfg(unix)]
type Fdandle = std::os::unix::RawFd;

// /// Gags the stdout stream.
// ///
// /// # Example
// /// ```rust
// /// #![cfg(windows)]
// /// println!("you will see this");
// /// let shh = shh::stdout().unwrap();
// /// println!("but not this");
// /// drop(shh);
// /// println!("and this");
// /// ```
// #[cfg(unix)]
// pub fn stdout() -> io::Result<Ssh<windows::Impl, io::Stdout>> {
// 	Ssh::new()
// }

// /// Gags the stderr stream.
// ///
// /// # Example
// /// ```rust
// /// #![cfg(windows)]
// /// eprintln!("you will see this");
// /// let shh = shh::stderr().unwrap();
// /// eprintln!("but not this");
// /// drop(shh);
// /// eprintln!("and this");
// /// ```
// #[cfg(unix)]
// pub fn stderr() -> io::Result<Ssh<windows::Impl, io::Stderr>> {
// 	Ssh::new()
// }

///////////////////////////////////////////////////////////////////

use std::io::{self, Read};
use std::marker::PhantomData;

pub trait Close {
	fn close_resource(&self);
}

trait Create: Sized {
	fn create_resource() -> io::Result<Self>;
}

pub trait Divert<D> {
	fn divert_std_stream(&self) -> io::Result<()>;
	fn reinstate_std_stream(original_fdandle: Fdandle) -> io::Result<()>;
}

trait Device {
	fn obtain_original() -> io::Result<Fdandle>;
}

pub struct Ssh<T: Close + Divert<D>, D> {
	inner: T,
	original: Fdandle,
	device_mrker: PhantomData<D>,
}

impl<T, D> Ssh<T, D>
where
	T: Create + Divert<D> + Close,
	D: Device,
{
	fn new() -> io::Result<Self> {
		// obtain current ptr to device
		let original = <D as Device>::obtain_original()?;

		// create data (a pipe or file) that can give a Fdandle to redirect to, and be closed
		let inner: T = Create::create_resource()?;

		// redirect the device to the write fdandle
		inner.divert_std_stream()?;

		Ok(Ssh {
			inner,
			original,
			device_mrker: PhantomData,
		})
	}
}

impl<T: Close + Divert<D>, D> Drop for Ssh<T, D> {
	fn drop(&mut self) {
		<T as Divert<D>>::reinstate_std_stream(self.original).unwrap();
		self.inner.close_resource();
	}
}

impl<T: Divert<D> + Close + Read, D> Read for Ssh<T, D> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner.read(buf)
	}
}
