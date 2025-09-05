// Output an error message to log and return an Err with the same message
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
