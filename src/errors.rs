// Handy macro for when we want to simultaneously print an error message and return an Err

macro_rules! log_and_return_err {
    ($($t:tt)*) => {
        {
            let msg = format!($($t)*);
            log::error!("{}", msg);
            Err(msg)
        }
    };
}

pub(crate) use log_and_return_err;
