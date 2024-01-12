#[macro_export]
macro_rules! debug_if {
    ($env:literal, $($arg:tt)+) => {
        match std::env::var($env) {
            Ok(var) if &var == "1" => log::debug!($($arg)+),
            _ => {}
        }
    };
}