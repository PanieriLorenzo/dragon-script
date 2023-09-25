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
    backtrace::Backtrace,
    cell::Cell,
    fmt::Display,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak,
    },
};

use ariadne::{Config, Label, Report, Source};
use log::debug;
use thiserror::Error;

use crate::{
    lexer::{Token, TokenType},
    source::{SourceArena, SourceView},
};

#[derive(Debug, Clone)]
#[repr(u16)]
enum ErrorType {
    SyntaxError = 0x1,
}

#[derive(Error, Debug)]
#[error("{msg}")]
pub struct DragonError {
    msg: String,
    ty: ErrorType,
    span: Option<SourceView>,
}

impl DragonError {
    fn report(&self, src: String) -> Result<(), std::io::Error> {
        let mut rep = Report::build(ariadne::ReportKind::Error, (), 12)
            .with_code(self.ty.clone() as u16)
            .with_message(self.msg.clone());
        if let Some(span) = self.span.clone() {
            rep = rep.with_label(Label::new(span.span.clone()).with_message("here"));
        } else {
            rep = rep.with_label(Label::new(src.len()..src.len()).with_message("here"));
        }
        rep.finish().eprint(Source::from(src))
    }
}

pub struct ErrorHandler {
    had_error: AtomicBool,
    src: Rc<SourceArena>,
    errors: Cell<Vec<DragonError>>,
    warnings: Cell<Vec<DragonError>>,
}

impl ErrorHandler {
    pub fn new(src: &Rc<SourceArena>) -> Self {
        Self {
            had_error: AtomicBool::new(false),
            src: src.clone(),
            errors: Cell::new(vec![]),
            warnings: Cell::new(vec![]),
        }
    }

    pub fn report_all(self: &Rc<Self>) {
        let inner = self.errors.take();
        for e in inner.iter() {
            e.report(self.src.to_string()).unwrap();
        }
    }

    pub fn syntax_error(self: Rc<Self>, span: SourceView, msg: String) {
        let mut errors = self.errors.take();
        errors.push(DragonError {
            msg,
            ty: ErrorType::SyntaxError,
            span: Some(span),
        });
        self.had_error.store(true, Ordering::Relaxed);
        self.errors.set(errors);
    }

    pub fn unexpected_char(self: Rc<Self>, span: SourceView, c: char) {
        let mut errors = self.errors.take();
        log::trace!("unexpected_char");
        errors.push(DragonError {
            msg: format!("unexpected character: '{}'", c),
            ty: ErrorType::SyntaxError,
            span: Some(span),
        });
        self.had_error.store(true, Ordering::Relaxed);
        self.errors.set(errors);
    }

    pub fn unexpected_token(
        self: Rc<Self>,
        span: SourceView,
        expected: &[TokenType],
        got: TokenType,
    ) {
        let mut errors = self.errors.take();
        errors.push(DragonError {
            msg: format!("unexpected token: {}, expected one of {:?}", got, expected),
            ty: ErrorType::SyntaxError,
            span: Some(span),
        });
        self.errors.set(errors);
    }

    pub fn unexpected_end_of_input(self: Rc<Self>) {
        let mut errors = self.errors.take();
        errors.push(DragonError {
            msg: format!("unexpected end of input"),
            ty: ErrorType::SyntaxError,
            span: None,
        });
        self.errors.set(errors);
    }
}

#[macro_export]
macro_rules! fatal {
    ($context:expr) => {
        log::error!("{}, terminating", $context);
        exit(1);
    };
}

#[macro_export]
macro_rules! internal_error {
    ($context:literal) => {{
        const FATAL_COPYPASTA: &str = concat!(
            "Ooops, that was unespected :/\n\n",
            "The compiler encountered an internal fatal error, from which it ",
            "cannot safely recover. This is most likely a bug in the compiler, ",
            "please reach out to the developer by reporting an issue on GitHub:\n",
            "https://github.com/PanieriLorenzo/dragon-script/issues\n",
            "include the following in the issue title: \n\n   ",
            $context,
            "\n\ninclude the following in the issue description: \n\n",
        );
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        log::error!("{}", FATAL_COPYPASTA.to_string() + &bt);
        std::process::exit(1)
    }};
}

#[macro_export]
macro_rules! assert_unreachable {
    () => {
        $crate::internal_error!("unreachable code was reached")
    };
}

#[macro_export]
macro_rules! assert_pre_condition {
    ($condition:expr) => {
        if !$condition {
            $crate::error_handler::fatal_pre_condition(stringify!($condition))
        }
    };
}

#[macro_export]
macro_rules! assert_invariant {
    ($condition:expr) => {
        if !$condition {
            $crate::error_handler::fatal_invariant(stringify!($condition))
        }
    };
}

/// Print the errors collected so far and returns the most appropriate UNIX
/// error code.
#[deprecated]
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
#[deprecated]
pub fn err_generic(msg: impl std::fmt::Debug) {
    const CODE: u32 = 00001;
    push_error(CODE, msg);
}

#[deprecated]
pub fn fatal_assertion(msg: impl std::fmt::Debug) {
    const CODE: u32 = 00002;
    crash_and_burn(CODE, format!("assertion failed: {:#?}", msg));
}

#[deprecated]
pub fn fatal_pre_condition(msg: impl std::fmt::Debug) {
    const CODE: u32 = 00003;
    crash_and_burn(CODE, format!("pre condition violation: {:#?}", msg))
}

#[deprecated]
pub fn fatal_post_condition(msg: impl std::fmt::Debug) {
    const CODE: u32 = 00004;
    crash_and_burn(CODE, format!("post condition violation: {:#?}", msg))
}

#[deprecated]
pub fn fatal_invariant(msg: impl std::fmt::Debug) {
    const CODE: u32 = 00005;
    crash_and_burn(CODE, format!("invariant violation: {:#?}", msg))
}

#[deprecated]
pub fn fatal_io_generic(msg: impl std::fmt::Debug) -> ! {
    const CODE: u32 = 01000;
    crash_and_burn(CODE, msg);
}

#[deprecated]
pub fn err_io_not_found(path: &str) {
    const CODE: u32 = 01001;
    push_error(CODE, "path ".to_owned() + path + " does not exist");
}

#[deprecated]
pub fn err_unexpected_character(c: char) {
    const CODE: u32 = 02001;
    push_error(CODE, format!("unexpected character: {}", c));
}

#[deprecated]
pub fn err_unclosed_delimiter(c: char) {
    const CODE: u32 = 02002;
    push_error(CODE, format!("missing closing delimiter '{}'", c));
}

#[deprecated]
pub fn err_unmatched_delimiter(c: char) {
    const CODE: u32 = 02003;
    push_error(
        CODE,
        format!(
            "unexpected closing delimiter '{}' with no matching opening",
            c
        ),
    );
}

#[deprecated]
pub fn err_unclosed_escape_sequence() {
    const CODE: u32 = 02004;
    push_error(
        CODE,
        "unexpected end of input while parsing escape sequence",
    );
}

#[deprecated]
pub fn err_int_too_big() {
    const CODE: u32 = 02005;
    push_error(CODE, "int literal is too large (max is 2^47)");
}

/// Same as `err_generic` but fatal (doesn't attempt to recover).
/// #[deprecated]
pub fn fatal_generic(msg: &str) -> ! {
    const CODE: u32 = 00001;
    crash_and_burn(CODE, msg);
}

#[deprecated]
pub fn fatal_unreachable() -> ! {
    const CODE: u32 = 00002;
    crash_and_burn(CODE, "unreachable code was reached");
}

// TODO: store errors as a data type, rather than raw String
#[deprecated]
static ERRORS: OnceLock<RwLock<Vec<String>>> = OnceLock::new();

#[deprecated]
static HAS_ERROR: AtomicBool = AtomicBool::new(false);

/// Get singleton writer to underlying error vector
#[deprecated]
fn error_writer() -> RwLockWriteGuard<'static, Vec<String>> {
    ERRORS
        .get_or_init(|| RwLock::new(vec![]))
        .write()
        .unwrap_or_else(|_| fatal_generic("poisoned lock"))
}

/// Get singleton reader to underlying error vector, prefer using this over
/// `error_writer` as it is more efficient
#[deprecated]
fn error_reader() -> RwLockReadGuard<'static, Vec<String>> {
    ERRORS
        .get_or_init(|| RwLock::new(vec![]))
        .read()
        .unwrap_or_else(|_| fatal_generic("poisoned lock"))
}

#[deprecated]
fn push_error(code: u32, msg: impl std::fmt::Debug) {
    let mut eh = error_writer();
    HAS_ERROR.store(true, Ordering::Relaxed);
    eh.push(format!("[E{:0>5}] {:#?}", code, msg));
}

// TODO: use macros for error handling, so that the top of the stack trace is
//       at the calling site
#[deprecated]
fn crash_and_burn(code: u32, msg: impl std::fmt::Debug) -> ! {
    const FATAL_COPYPASTA: &str = concat!(
        "Ooops, that was unespected :/\n\n",
        "The compiler encountered an internal fatal error, from which it",
        "cannot safely recover. This is most likely a bug in the compiler, ",
        "please reach out to the developer by reporting an issue on github:\n",
        "https://github.com/PanieriLorenzo/dragon-script/issues\n",
        "include the following in the issue title: "
    );

    HAS_ERROR.store(true, Ordering::Relaxed);

    println!("[F{:0>5}] {} {:#?}", code, FATAL_COPYPASTA, msg);
    println!();
    println!("{}", Backtrace::force_capture());
    panic!("{:#?}", msg);
}
