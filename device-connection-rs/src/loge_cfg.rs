//!# Device and Device Pool
//! Log config
//! ```
//! // Init the logger level is warning
//! loge_cfg::log_init(Some("warn"))?;
//! // Default the logger level is trace
//! loge_cfg::log_init(None)?;
//! ```

use std::env;

/// Init to the logger
pub fn log_init(level: Option<&str>) -> Result<(), String> {
    if let Some(le) = level {
        env::set_var("RUST_LOG", le);
    } else {
        env::set_var("RUST_LOG", "trace");
    }
    env::set_var("LOGE_FORMAT", "fileline");
    //loge::init();
    loge::init_with_file("temp.log");
    Ok(())
}
