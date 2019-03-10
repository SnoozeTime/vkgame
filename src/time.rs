use std::time::Duration;

// Should be alright with the conversion. A frame duration will not exceed f32...
pub fn dt_as_secs(dt: Duration) -> f64 {
    dt.subsec_nanos() as f64/1000_000_000.0 + (dt.as_secs() as f64)
}


//#[macro_export]
macro_rules! timed {
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match std::time::Instant::now() {
            now => {
                match $val {
                    tmp => {
                        let elapsed = $crate::time::dt_as_secs(std::time::Instant::now() - now);
                        eprintln!("[{}:{}] {} = {:#?}",
                                  file!(), line!(), stringify!($val), elapsed);
                        tmp
                    }
                }
            }
        }
    }
}
