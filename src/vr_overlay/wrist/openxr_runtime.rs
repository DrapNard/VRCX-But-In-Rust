use super::WristError;

#[derive(Debug, Clone)]
pub struct OpenXrCapability {
    pub runtime_name: Option<String>,
    pub overlay_extension: bool,
    pub reason: Option<String>,
}

/// Loads the system OpenXR loader and checks the provisional external-overlay extension.
/// A normal quad layer is insufficient: it is only visible inside the application's own session.
pub fn probe_openxr_overlay() -> Result<OpenXrCapability, WristError> {
    // Safety: `Entry` owns the dynamically loaded OpenXR loader for the duration of this call.
    let entry = unsafe { openxr::Entry::load() }
        .map_err(|e| WristError::Runtime(format!("OpenXR loader: {e}")))?;
    let extensions = entry
        .enumerate_extensions()
        .map_err(|e| WristError::Runtime(format!("OpenXR extensions: {e}")))?;
    let runtime_name = entry
        .create_instance(
            &openxr::ApplicationInfo {
                application_name: "VRCX-BIR capability probe",
                application_version: 1,
                engine_name: "VRCX-BIR",
                engine_version: 1,
                api_version: openxr::Version::new(1, 0, 0),
            },
            &openxr::ExtensionSet::default(),
            &[],
        )
        .ok()
        .and_then(|instance| instance.properties().ok())
        .map(|p| p.runtime_name);
    Ok(OpenXrCapability {
        runtime_name,
        overlay_extension: extensions.extx_overlay,
        reason: (!extensions.extx_overlay)
            .then(|| "XR_EXTX_overlay is unavailable; use WayVR as the compositor bridge".into()),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn missing_loader_is_a_regular_runtime_error() {
        // The real probe is intentionally not run in CI: it reflects the host runtime.
        assert_eq!(openxr::raw::OverlayEXTX::NAME, b"XR_EXTX_overlay\0");
    }
}
