//! (Rust) Silence stderr and stdout, optionally rerouting it.
//!
//! Stdout Gagging
//!
//! ```rust
//! println!("STDOUT GAGGING", );
//! println!("you will see this");
//! let shh = shh::stdout().unwrap();
//! println!("but not this");
//! drop(shh);
//! println!("and this");
//! ```
//!
//! Stderr Gagging
//!
//! ```rust
//! println!("STDERR GAGGING", );
//! eprintln!("you will see this");
//! let shh = shh::stderr().unwrap();
//! eprintln!("but not this");
//! drop(shh);
//! eprintln!("and this");
//! ```
//!
//! Redirecting Example
//!
//! ```rust
//! println!("REDIRECTING", );
//! use std::io::{Read, Write};
//!
//! std::thread::spawn(move || {
//!     let mut shh = shh::stdout().unwrap();
//!     let mut stderr = std::io::stderr();
//!     loop {
//!         let mut buf = Vec::new();
//!         shh.read_to_end(&mut buf).unwrap();
//!         stderr.write_all(&buf).unwrap();
//!     }
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
//! shh::stdout().unwrap();        // Shh struct is created, and dropped, here
//! println!("and expect not to see this, but you will");
//! ```
//!
//! To fix this, just assign a local variable
//! ```rust
//! println!("you will see this");
//! let shh = shh::stdout().unwrap();        // Shh struct is created here
//! println!("and expect not to see this");
//! drop(shh);    // and dropped here
//! println!("now it works!");
//! ```

#![warn(missing_docs)]

/// Just string replace the standard api interface.
macro_rules! create_impl_interface {
    ($os:tt, $fdandle:ty) => {
        #[cfg($os)]
        mod $os;

        /// My pet name for the unix Fd or windows Handle types.
        #[cfg($os)]
        type Fdandle = $fdandle;

        /// Type alias for `Shh`ing the stdout.
        #[cfg($os)]
        pub type ShhStdout = Shh<$os::Impl, io::Stdout>;

        /// Type alias for `Shh`ing the stderr.
        #[cfg($os)]
        pub type ShhStderr = Shh<$os::Impl, io::Stderr>;

        /// Silence and redirect the stdout stream.
        ///
        /// `Shh` implements `io::Read`, with all captured output able to be read back out.
        ///
        /// # Example
        /// ```rust
        /// println!("you will see this");
        /// let shh = shh::stdout().unwrap();
        /// println!("but not this");
        /// drop(shh);
        /// println!("and this");
        /// ```
        #[cfg($os)]
        pub fn stdout() -> io::Result<ShhStdout> {
            Shh::new()
        }

        /// Silence and redirect the stderr stream.
        ///
        /// `Shh` implements `io::Read`, with all captured output able to be read back out.
        ///
        /// # Example
        /// ```rust
        /// eprintln!("you will see this");
        /// let shh = shh::stderr().unwrap();
        /// eprintln!("but not this");
        /// drop(shh);
        /// eprintln!("and this");
        /// ```
        #[cfg($os)]
        pub fn stderr() -> io::Result<ShhStderr> {
            Shh::new()
        }
    };
}

create_impl_interface!(windows, std::os::windows::io::RawHandle);
create_impl_interface!(unix, std::os::unix::io::RawFd);

use std::fs::File;
use std::io::{self, Read};
use std::marker::PhantomData;

/// Trait which can create a read and write pipe as files.
pub trait Create {
    /// Create the read and write handles, taking ownership with `File`.
    /// Return in (read_file, write_file)
    fn create_files() -> io::Result<(File, File)>;
}

/// Trait which defines functions that divert and reinstate streams for std devices.
pub trait Divert<D> {
    /// Divert from the device into the `write_file`'s handle/fd;
    fn divert_std_stream(write_file: &File) -> io::Result<()>;
    /// Reinstate the std device to output. Gives the original handle/fd for use.
    fn reinstate_std_stream(original_fdandle: Fdandle) -> io::Result<()>;
}

/// Trait to enable getting the handle/fd a std device was looking at.
pub trait Device {
    /// Obtain the original handle/fd the std device was using.
    fn obtain_original() -> io::Result<Fdandle>;
}

/// Trait to read from file.
///
/// Unfortunately, the windows implementation needs a little more massaging to read from the pipe,
/// without completely blocking the thread. This trait effectively gives implementor a little more control
/// over the reading if they want to inject their own method. Otherwise just call `.read()` on the `File`.
pub trait ShhRead {
    /// Read contents of file into `buf`. Works the same way as [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
    fn shh_read(read_file: &File, buf: &mut [u8]) -> io::Result<usize>;
}

/// A structure holding the redirection data.
///
/// `Shh` implements `io::Read`, with all captured output able to be read back out.
pub struct Shh<Impl, Device>
where
    Impl: Divert<Device>,
{
    original: Fdandle,
    write_file: File,
    read_file: File,
    impl_mrker: PhantomData<Impl>,
    device_mrker: PhantomData<Device>,
}

impl<I, D> Shh<I, D>
where
    I: Create + Divert<D>,
    D: Device,
{
    fn new() -> io::Result<Self> {
        // obtain current ptr to device
        let original = <D as Device>::obtain_original()?;

        // create data (a pipe or file) that can give a Fdandle to redirect to, and be closed
        let (read_file, write_file) = <I as Create>::create_files()?;

        let shh = Shh {
            original,
            read_file,
            write_file,
            impl_mrker: PhantomData,
            device_mrker: PhantomData,
        };

        // redirect the device to the write fdandle
        <I as Divert<D>>::divert_std_stream(&shh.write_file)?;

        Ok(shh)
    }
}

impl<I: Divert<D>, D> Drop for Shh<I, D> {
    fn drop(&mut self) {
        <I as Divert<D>>::reinstate_std_stream(self.original).unwrap_or(());
    }
}

impl<I: Divert<D> + ShhRead, D> Read for Shh<I, D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        <I as ShhRead>::shh_read(&self.read_file, buf)
    }
}

/// Unsafe because of the `original: Fdandle`. This is retrieved from os and does not need
/// cleaning up.
unsafe impl<I: Divert<D>, D> Send for Shh<I, D> {}
