//! Error reporting module, keeps track of all errors during compilation.
//!
//! Note that lints as not supported yet, these are all hard-coded errors,
//! whereas lints can be disabled or enabled by the user. Perhaps in the
//! future we will have lint reporting.
//!
//! ## Code Ranges
//! Each error has a code, but equivalent errors of different severities,
//! e.g. `err_io_generic` and `warn_io_generic` share the same code.
//!
//! - range `00xxx`: generic or top-level errors, used also for CLI errors
//!     - `00000`: unused, represents lack of errors
//!     - `00001`: generic error
//! - range `01xxx`: I/O errors
//! - range `02xxx`: syntax errors
//!
//! ## Error Severities
//! - warn:  the program will compile, but will likey fail at run-time or if
//!          requirements change
//! - err:   the program cannot be compiled, this is a normal part of
//!          development workflow, so we need to provide high-quality
//!          reporting
//! - fatal: the compiler did something it shouldn't have done. These errors
//!          are the bad ones, they should never occur.

use std::{
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

/// Print the errors collected so far and returns the most appropriate UNIX
/// error code.
pub fn display_errors() -> i32 {
    // display and discard errors in the buffer
    {
        let mut ew = error_writer();
        for e in ew.iter() {
            println!("{}", e);
        }
        ew.clear();
    }

    // TODO: right now it always returns 0 or 1
    // calculate error code and reset error status
    let exit_code = if HAS_ERROR.load(Ordering::Relaxed) {
        1
    } else {
        0
    };
    HAS_ERROR.store(false, Ordering::Relaxed);

    exit_code
}

/// Reports an uncategorized error, should be considered as a To-Do for adding
/// new error types in later versions.
pub fn err_generic(msg: impl std::fmt::Debug) {
    const CODE: u32 = 00001;
    push_error(CODE, msg);
}

pub fn fatal_io_generic(msg: impl std::fmt::Debug) -> ! {
    const CODE: u32 = 01000;
    crash_and_burn(CODE, msg);
}

pub fn err_io_not_found(path: &str) {
    const CODE: u32 = 01001;
    push_error(CODE, "path ".to_owned() + path + " does not exist");
}

/// Same as `err_generic` but fatal (doesn't attempt to recover).
pub fn fatal_generic(msg: &str) -> ! {
    const CODE: u32 = 00001;
    crash_and_burn(CODE, msg);
}

pub fn fatal_unreachable() -> ! {
    const CODE: u32 = 00002;
    crash_and_burn(CODE, "unreachable code was reached");
}

// TODO: store errors as a data type, rather than raw String
static ERRORS: OnceLock<RwLock<Vec<String>>> = OnceLock::new();
static HAS_ERROR: AtomicBool = AtomicBool::new(false);

/// Get singleton writer to underlying error vector
fn error_writer() -> RwLockWriteGuard<'static, Vec<String>> {
    ERRORS
        .get_or_init(|| RwLock::new(vec![]))
        .write()
        .unwrap_or_else(|_| fatal_generic("poisoned lock"))
}

/// Get singleton reader to underlying error vector, prefer using this over
/// `error_writer` as it is more efficient
fn error_reader() -> RwLockReadGuard<'static, Vec<String>> {
    ERRORS
        .get_or_init(|| RwLock::new(vec![]))
        .read()
        .unwrap_or_else(|_| fatal_generic("poisoned lock"))
}

fn push_error(code: u32, msg: impl std::fmt::Debug) {
    let mut eh = error_writer();
    HAS_ERROR.store(true, Ordering::Relaxed);
    eh.push(format!("[E{:0>5}] {:#?}", code, msg));
}

fn crash_and_burn(code: u32, msg: impl std::fmt::Debug) -> ! {
    const FATAL_COPYPASTA: &str = concat!(
        "The compiler encountered an internal fatal error, from which it",
        "cannot safely recover. This is most likely a bug in the compiler, ",
        "please reach out to the developer by reporting an issue on github:\n",
        "https://github.com/PanieriLorenzo/dragon-script/issues\n",
        "include the following in the issue title: "
    );

    HAS_ERROR.store(true, Ordering::Relaxed);

    println!("[F{:0>5}] {} {:#?}", code, FATAL_COPYPASTA, msg);
    panic!("{:#?}", msg);
}
