use crate::feature::Permission;

pub fn permission_granted(_: Permission) -> bool {
    true
}

pub fn request_permission(_: Permission) {}
