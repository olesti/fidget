use crate::feature::Permission;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as os;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as os;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as os;

pub fn permission_granted(p: Permission) -> bool {
    os::permission_granted(p)
}

pub fn request_permission(p: Permission) {
    os::request_permission(p)
}
