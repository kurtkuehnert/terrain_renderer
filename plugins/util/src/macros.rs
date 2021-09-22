/// Simple macro to unwrap and return or continue.
#[macro_export]
macro_rules! unwrap_or {
    ( $els: expr, $val:expr ) => {
        match $val {
            Ok(x) => x,
            Err(_) => $els,
        }
    };
}
