use nix::libc;
use std::mem::MaybeUninit;
use std::ffi::{CStr};
use std::{env, ptr};
use std::error::Error;
use crate::error::PtyError;

/**
 * Shell User composed of environment variables
 */
pub(crate) struct ShellUser {
    pub user: String,
    pub home: String,
    pub shell: String,
}

impl ShellUser {
    /**
     * Constructs a shell user from environment
     */
    pub(crate) fn from_env() -> Result<ShellUser, Box<dyn Error>> {
        let mut buf: [u8; 1024] = [0; 1024];
        // Create zeroed passwd struct.
        let mut entry: MaybeUninit<libc::passwd> = MaybeUninit::uninit();
        let mut res: *mut libc::passwd = ptr::null_mut();

        // Try and read the pw file.
        let uid = unsafe { libc::getuid() };
        let status = unsafe { libc::getpwuid_r(
            uid,
            entry.as_mut_ptr(),
            buf.as_mut_ptr() as *mut _,
            buf.len(),
            &mut res
        )};
        let entry = unsafe { entry.assume_init() };

        if status < 0 {
            return Err(Box::new(PtyError("session password UID status error".into())));
        }

        if res.is_null() {
            return Err(Box::new(PtyError("session password response error".into())));
        }
        // Sanity check.
        assert_eq!(entry.pw_uid, uid);

        let user = match env::var("USER") {
            Ok(user) => user,
            Err(_) => unsafe {
                CStr::from_ptr(entry.pw_name).to_str()?.to_owned()
            }
        };

        let home = match env::var("HOME") {
            Ok(home) => home,
            Err(_) => unsafe {
                CStr::from_ptr(entry.pw_dir).to_str()?.to_owned()
            }
        };

        let shell = match env::var("SHELL") {
            Ok(shell) => shell,
            Err(_) => unsafe {
                CStr::from_ptr(entry.pw_shell).to_str()?.to_owned()
            }
        };

        Ok(Self {
            user,
            home,
            shell
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_from_env() {
        let _shell_user = ShellUser::from_env().unwrap();
    }
}
