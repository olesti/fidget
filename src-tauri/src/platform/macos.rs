use crate::feature::Permission;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn permission_granted(p: Permission) -> bool {
    match p {
        Permission::Accessibility => unsafe { AXIsProcessTrusted() },
    }
}

/// Opens System Settings → Privacy & Security → Accessibility.
pub fn request_permission(p: Permission) {
    match p {
        Permission::Accessibility => {
            let _ = std::process::Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn();
        }
    }
}
