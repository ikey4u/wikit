pub mod config;
pub mod error;
pub mod mac;
pub mod mdict;
pub mod reader;
pub mod util;
#[cfg(feature = "server")]
pub mod router;
pub mod index;
pub mod wikit;
pub mod cache;
pub mod crypto;
#[cfg(feature = "preview")]
pub mod preview;

#[test]
fn test_core_debug() {
}
