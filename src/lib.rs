pub mod app;
pub mod app_info;
pub mod cacher;
pub mod crates;
pub mod github;
pub mod opts;
pub mod probe;

use once_cell::sync::Lazy;
use semver::Version;

pub const RI_USER_AGENT: &str = "Rust Insight! Launcher (support@rustinsight.com)";

pub static VERSION: Lazy<Version> = Lazy::new(|| built_info::PKG_VERSION.parse().unwrap());

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
