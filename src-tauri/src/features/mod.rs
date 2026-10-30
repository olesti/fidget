use std::sync::Arc;

use crate::feature::Feature;

pub mod jiggle;

/// Add new features here; tray section and toggle come for free.
pub fn register_all() -> Vec<Arc<dyn Feature>> {
    vec![Arc::new(jiggle::Jiggle::new())]
}
