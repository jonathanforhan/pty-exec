# pty-exec

```rust
use std::os::fd::{AsRawFd, FromRawFd};
use pty_exec::Pty;

// spawn Pty
let pty = Pty::spawn(move |_fd, res| {
    println!("{}", res.unwrap());
}, move |fd| {
    println!("{fd} died");
})?;

// (optional) create new pty, this maintains the on_read and on_death callbacks
// this means a RawFd can be passed to client like in a tauri app
let pty = unsafe { Pty::from_raw_fd(pty.as_raw_fd()) };

// write to original pty with new pty from_raw_fd
pty.write("echo 'Hello, World'\r")?;

pty.kill();
```