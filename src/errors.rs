//! A utility that provides a useful error-handling macro.

// Formats the inputs, logs the result, and returns an error with the same message. 
#[macro_export]
macro_rules! log_and_return_err {
    ($($t:tt)*) => {
        {
            let msg = format!($($t)*);
            error!("{}", msg);
            return Err(msg);
        }
    };
}
