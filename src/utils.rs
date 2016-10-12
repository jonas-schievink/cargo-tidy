use std::fmt::Display;
use std::process;

/// `println!`, but for stderr instead of stdout.
macro_rules! errprintln {
    ( $($args:tt)* ) => {{
        use std::io::Write;
        let stderr = ::std::io::stderr();
        let mut stderr = stderr.lock();
        writeln!(stderr, $($args)*).unwrap();
    }}
}

/// Unwraps a `Result`, exiting the application in the error case.
///
/// If the given result is an `Err`, prints the error to stderr and exits the application with an
/// error code.
pub fn unwrap_or_exit<T, E: Display>(result: Result<T, E>) -> T {
    match result {
        Ok(val) => val,
        Err(e) => {
            errprintln!("{}", e);
            errprintln!("Exiting.");
            process::exit(1);
        }
    }
}
