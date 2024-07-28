#[macro_export]
macro_rules! tracer {
    () => {
        if std::env::var("ENABLE_VRP_TRACING").is_ok() {
            println!();
        }
    };
    ($x:expr) => {
        if std::env::var("ENABLE_VRP_TRACING").is_ok() {
            dbg!($x);
        }
    };
    ($($arg:tt)*) => {
        if std::env::var("ENABLE_VRP_TRACING").is_ok() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_output {
    () => {
        if std::env::var("ENABLE_OUTPUT_TRACING").is_ok() {
            println!();
        }
    };
    ($x:expr) => {
        if std::env::var("ENABLE_OUTPUT_TRACING").is_ok() {
            dbg!($x);
        }
    };
}

#[macro_export]
macro_rules! prettify {
    () => {
        if std::env::var("ENABLE_PRETTY_TRACING").is_ok() {
            println!();
        }
    };
    ($x:expr) => {
        if std::env::var("ENABLE_PRETTY_TRACING").is_ok() {
            println!("{:?}", $x);
        }
    };
}

#[macro_export]
macro_rules! traced {
    () => {
        if std::env::var("ENABLE_SOME_TRACING").is_ok() {
            println!();
        }
    };
    ($x:expr) => {
        if std::env::var("ENABLE_SOME_TRACING").is_ok() {
            dbg!($x);
        }
    };
}
