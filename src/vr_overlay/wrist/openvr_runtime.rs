use super::{WristError, WristFrame, WristFrameSink};
use crate::vr_overlay::config::{Hand, WristConfig};
use openvr::{
    ApplicationType, Context, Overlay, System, TrackedControllerRole, overlay::OverlayHandle,
    pose::Matrix3x4,
};

/// Live connection to SteamVR's compositor through `IVROverlay`.
pub struct OpenVrWristRenderer {
    _context: Context,
    overlay: Overlay,
    handle: OverlayHandle,
}

impl OpenVrWristRenderer {
    pub fn connect(config: &WristConfig) -> Result<Self, WristError> {
        if !openvr::is_runtime_installed() {
            return Err(WristError::Runtime(
                "SteamVR/OpenVR is not installed".into(),
            ));
        }
        // Safety: the context is owned by this renderer and outlives every derived interface.
        let context = unsafe { openvr::init(ApplicationType::Overlay) }
            .map_err(|e| WristError::Runtime(format!("OpenVR initialization: {e:?}")))?;
        let system = context.system().map_err(init_error)?;
        let mut overlay = context.overlay().map_err(init_error)?;
        let handle = overlay
            .create_overlay("dev.vrcx-bir.wrist\0", "VRCX-BIR Wrist\0")
            .map_err(overlay_error)?;
        overlay
            .set_width(handle, config.width_metres * config.scale)
            .map_err(overlay_error)?;
        overlay
            .set_opacity(handle, config.opacity.clamp(0.0, 1.0))
            .map_err(overlay_error)?;
        attach_to_hand(&mut overlay, handle, &system, config)?;
        overlay
            .set_visibility(handle, true)
            .map_err(overlay_error)?;
        Ok(Self {
            _context: context,
            overlay,
            handle,
        })
    }
}

impl WristFrameSink for OpenVrWristRenderer {
    fn submit_frame(&mut self, frame: &WristFrame) -> Result<(), WristError> {
        // SetOverlayRaw uploads the RGBA buffer into the runtime-owned GPU overlay texture.
        self.overlay
            .set_raw_data(
                self.handle,
                &frame.rgba,
                frame.width as usize,
                frame.height as usize,
                4,
            )
            .map_err(overlay_error)
    }
    fn set_visible(&mut self, visible: bool) -> Result<(), WristError> {
        self.overlay
            .set_visibility(self.handle, visible)
            .map_err(overlay_error)
    }
}

fn attach_to_hand(
    overlay: &mut Overlay,
    handle: OverlayHandle,
    system: &System,
    config: &WristConfig,
) -> Result<(), WristError> {
    let role = match config.hand {
        Hand::Left => TrackedControllerRole::LeftHand,
        Hand::Right => TrackedControllerRole::RightHand,
    };
    let device = system
        .tracked_device_index_for_controller_role(role)
        .ok_or_else(|| {
            WristError::Runtime(format!("{:?} controller is not tracked", config.hand))
        })?;
    let [rx, ry, rz] = config.rotation_degrees.map(f32::to_radians);
    let (sx, cx) = rx.sin_cos();
    let (sy, cy) = ry.sin_cos();
    let (sz, cz) = rz.sin_cos();
    let [x, y, z] = config.offset_metres;
    let transform = Matrix3x4([
        [cy * cz, cz * sx * sy - cx * sz, sx * sz + cx * cz * sy, x],
        [cy * sz, cx * cz + sx * sy * sz, cx * sy * sz - cz * sx, y],
        [-sy, cy * sx, cx * cy, z],
    ]);
    overlay
        .set_transform_tracked_device_relative(handle, device, &transform)
        .map_err(overlay_error)
}

fn init_error(e: openvr::InitError) -> WristError {
    WristError::Runtime(format!("OpenVR interface: {e:?}"))
}
fn overlay_error(e: openvr::overlay::VROverlayError) -> WristError {
    WristError::Render(format!("OpenVR overlay: {e:?}"))
}

impl Drop for OpenVrWristRenderer {
    fn drop(&mut self) {
        let _ = self.overlay.set_visibility(self.handle, false);
    }
}
