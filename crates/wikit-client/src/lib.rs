//! Native business logic shared by all three NSH applications.
mod service;

use nsh_core::prelude::*;

#[native_module("wikit/native")]
fn native(registry: &mut Registry) {
    registry.module::<Wikit>();
}

/// Dictionary operations using the shared app request and response schema.
#[module]
pub struct Wikit;

#[module]
impl Wikit {
    /// Execute an operation outside the UI thread, in the host's app data directory.
    pub async fn request(sys: System, payload: String) -> Result<String, String> {
        let directory = sys
            .data_dir
            .ok_or("The native host did not provide an app data directory")?;
        blocking(move || service::dispatch(&directory, &payload))
            .await
            .map_err(|e| e.to_string())?
    }

    /// Pick a dictionary on desktop. Mobile clients import by download or sandbox path.
    pub async fn pick_dictionary() -> Result<String, String> {
        use nsh_core::file_dialog::{FileFilter, OpenDialog};
        let paths =
            pick_paths(OpenDialog::files().filter(FileFilter::new("Wikit dictionary", ["wikit"])))
                .await?;
        Ok(paths
            .and_then(|v| v.into_iter().next())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default())
    }
}
