macro_rules! make_io_err {
    ($kind:ident, $($err:tt)+) => {
        std::io::Error::new(std::io::ErrorKind::$kind, format!($($err)+))
    }
}
