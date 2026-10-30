use crate::feature::Permission;

/// X11 needs nothing. On Wayland enigo uses the wlroots virtual-pointer protocol or the
/// RemoteDesktop portal (the compositor shows a one-time consent dialog), so there is
/// no permission we can pre-check or open a settings page for.
pub fn permission_granted(_: Permission) -> bool {
    true
}

pub fn request_permission(_: Permission) {}
