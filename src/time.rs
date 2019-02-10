use std::time::Duration;

// Should be alright with the conversion. A frame duration will not exceed f32...
pub fn dt_as_secs(dt: Duration) -> f32 {
    dt.subsec_millis() as f32/1000.0 + (dt.as_secs() as f32)
}

