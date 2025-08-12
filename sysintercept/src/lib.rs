mod syscalls;

use lazy_static::lazy_static;
use libc::exit;
use std::{
    cell::{Cell, RefCell},
    io::{BufWriter, Write},
    net::TcpStream,
    sync::Mutex,
};
use syscall_intercept::*;

static mut CLIENT: Option<Mutex<BufWriter<TcpStream>>> = None;

#[ctor::ctor]
fn start() {
    unsafe {
        CLIENT = Some({
            match TcpStream::connect("127.0.0.1:1234") {
                Ok(stream) => {
                    let writer = BufWriter::new(stream);
                    Mutex::new(writer)
                }
                Err(e) => {
                    eprintln!("Failed to connect to server: {}", e);
                    exit(1);
                }
            }
        });
    }

    unsafe { set_hook_fn(hook) };
}

// fn print_direct(str: &str)

thread_local! {
    /// A flag indicating whether the current thread is in an intercept context.
    static INTERCEPTED: Cell<bool> = Cell::new(false);
}

// lazy_static! {

// }

struct InterceptGuard;

impl InterceptGuard {
    fn try_lock() -> Option<Self> {
        INTERCEPTED.with(|x| {
            if x.get() {
                None
            } else {
                x.set(true);
                Some(InterceptGuard)
            }
        })
    }
}

impl Drop for InterceptGuard {
    fn drop(&mut self) {
        INTERCEPTED.with(|x| x.set(false));
    }
}

extern "C" fn hook(
    syscall_num: isize,
    arg0: isize,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
    result: &mut isize,
) -> InterceptResult {
    // detect and avoid recursive interception
    let _guard = match InterceptGuard::try_lock() {
        Some(g) => g,
        None => return InterceptResult::Forward,
    };

    // unsafe {
    //     unset_hook_fn();
    // }

    // Return if is print
    // if syscall_num == libc::SYS_write as _ && arg0 == 1 {
    //     return InterceptResult::Forward;
    // }

    let args = [arg0, arg1, arg2, arg3, arg4, arg5];
    let desc = syscalls::get_syscall_desc(syscall_num, args);

    unsafe {
        #[allow(static_mut_refs)]
        if let Some(client) = CLIENT.as_ref() {
            client
                .lock()
                .unwrap()
                .write_all(&format!("Data: {:?}\n", desc).as_bytes())
                .unwrap();
            client.lock().unwrap().flush().unwrap();
        }
    }

    // unsafe {
    //     set_hook_fn(hook);
    // }

    InterceptResult::Forward
}
