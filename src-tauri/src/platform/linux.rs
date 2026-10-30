use crate::feature::Permission;

// ponytail: X11 only. Wayland (uinput / RemoteDesktop portal) is v0.2.
pub fn permission_granted(_: Permission) -> bool {
    true
}

pub fn request_permission(_: Permission) {}
