#[macro_export]
macro_rules! errno {
    ($msg:expr $(, $args:expr)*) => {
        std::result::Result::Err($crate::errors::Errno::New(format!($msg $(, $args)*)))
    };
}


#[macro_export]
macro_rules! errno_new {
    ($msg:expr $(, $args:expr)*) => {
        $crate::errors::Errno::New(format!($msg $(, $args)*))
    };
}