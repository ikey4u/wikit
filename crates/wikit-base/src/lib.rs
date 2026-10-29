//! Reusable dictionary parsing, indexing and lookup, without an application runtime.
//!
//! ```no_run
//! use wikit_base::Dictionary;
//! let dictionary = Dictionary::open("example.wikit")?;
//! for (word, html) in dictionary.lookup("hello")? {
//!     println!("{word}: {html}");
//! }
//! # Ok::<(), wikit_base::WikitError>(())
//! ```

pub mod dictionary;
pub mod resources;
pub use dictionary::Dictionary;
pub use error::{WikitError, WikitResult};
pub use mdict::{MDXDict, ParseOption, parse_mdx};
pub use wikit::LocalDictionary;
pub mod cache;
pub mod config;
pub mod crypto;
pub mod error;
pub mod index;
pub mod mac;
pub mod mdict;
#[cfg(feature = "preview")]
pub mod preview;
pub mod reader;
pub mod util;
pub mod wikit;

#[test]
fn test_core_debug() {}
