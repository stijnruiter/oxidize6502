
#[allow(unused_macros)]
macro_rules! log_println {
    ($($arg:tt)*) => {{
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("output.log")
            .unwrap();
        writeln!(file, $($arg)*).unwrap();
    }};
}