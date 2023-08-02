//! ```rust
//! use std::os::fd::{AsRawFd, FromRawFd};
//! use pty_exec::Pty;
//!
//! // spawn Pty
//! let pty = Pty::spawn(move |_fd, res| {
//!     println!("-> {}", res.unwrap());
//! }, move |fd| {
//!     println!("-> {fd} died");
//! })?;
//!
//! // (optional) create new pty, this maintains the on_read and on_death callbacks
//! let pty = unsafe { Pty::from_raw_fd(pty.as_raw_fd()) };
//!
//! // write to original pty with new pty from_raw_fd
//! pty.write("echo 'Hello, World'\r")?;
//!
//! pty.kill();
//! ```

pub mod error;
mod unix;

pub use error::PtyError;
use std::error::Error;
use std::os::fd::{FromRawFd, AsRawFd, RawFd};
use crate::unix::window::WindowSize;

/// Pty struct that encapsulates pid of our tty
/// _DOES NOT_ close pty on drop() _ONLY_ on Pty::kill()
/// this is so that a pty process can outlive this struct
pub struct Pty {
    pid: RawFd
}

impl Pty {
    /// Spawns a new pty,
    /// on_read: callback called when there is something to read
    /// on_death: callback called when there the pty dies
    pub fn spawn<F, G>(on_read: F, on_death: G) -> Result<Pty, Box<dyn Error>>
        where
            F: FnMut(RawFd, Result<String, Box<dyn Error>>) + Send + 'static,
            G: FnMut(RawFd) + Send + 'static
    {
        let master = unix::pty::spawn()?;
        unix::pty::poll(master, on_read, on_death)?;

        Ok(Pty { pid: master })
    }

    /// write to pty
    pub fn write(&self, s: &str) -> Result<(), Box<dyn Error>> {
        unix::pty::write(self.pid, s.as_bytes())
    }

    /// resize pty with syscall
    pub fn resize(&self, window_size: WindowSize) -> Result<(), Box<dyn Error>> {
        unix::pty::resize(self.pid, window_size)
    }

    /// kill pty
    pub fn kill(&self) {
        unix::pty::kill(self.pid)
    }
}

impl FromRawFd for Pty {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Pty { pid: fd }
    }
}

impl AsRawFd for Pty {
    fn as_raw_fd(&self) -> RawFd {
        self.pid
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::sync::{Arc, Mutex};
    use super::*;

    #[test]
    fn spawn() -> Result<(), Box<dyn Error>> {
        let read_buf = Arc::new(Mutex::new(String::new()));
        let die_buf = Arc::new(Mutex::new(String::new()));

        let (read_buf_async, die_buf_async) = (read_buf.clone(), die_buf.clone());

        // spawn Pty
        let pty = Pty::spawn(move |_fd, res| {
            read_buf_async.lock().unwrap().push_str(res.unwrap().as_str());
        }, move |fd| {
            die_buf_async.lock().unwrap().push_str(format!("{fd} dead").as_str());
        })?;
        std::thread::sleep(Duration::from_millis(100));

        // create new pty, this maintains the on_read and on_death callbacks
        let pty = unsafe { Pty::from_raw_fd(pty.as_raw_fd()) };
        // write to original pty with new pty from_raw_fd
        pty.write("echo 'Hello, World'\r")?;
        std::thread::sleep(Duration::from_millis(100));

        pty.kill();
        std::thread::sleep(Duration::from_millis(100));

        // read_buf are effected whether using Pty::spawn or Pty::from_raw_fd() on a
        // pre-existing spawned pty
        assert!(read_buf.lock().unwrap().contains("echo 'Hello, World'"));
        assert_eq!(die_buf.lock().unwrap().as_str(), format!("{} dead", pty.as_raw_fd()).as_str());

        Ok(())
    }
}
