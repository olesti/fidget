use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Permission {
    Accessibility,
}

/// Every feature implements this. `start` must return quickly (spawn a task).
// ponytail: sync trait, features spawn their own tasks; async_trait if start/stop ever need to await.
pub trait Feature: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    #[allow(dead_code)] // used by the v0.2 settings form
    fn config_schema(&self) -> serde_json::Value;
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
    fn start(&self) -> anyhow::Result<()>;
    fn stop(&self);
    /// Live status line shown under the feature in the tray menu.
    fn status(&self) -> String;
}

#[derive(Default)]
pub struct FeatureManager {
    features: Vec<Arc<dyn Feature>>,
    enabled: Mutex<HashSet<&'static str>>,
}

impl FeatureManager {
    pub fn new(features: Vec<Arc<dyn Feature>>) -> Self {
        Self { features, enabled: Mutex::default() }
    }

    pub fn features(&self) -> &[Arc<dyn Feature>] {
        &self.features
    }

    pub fn is_enabled(&self, id: &str) -> bool {
        self.enabled.lock().unwrap().contains(id)
    }

    pub fn any_enabled(&self) -> bool {
        !self.enabled.lock().unwrap().is_empty()
    }

    /// Flip a feature on/off. Returns the resulting state.
    pub fn toggle(&self, id: &str) -> anyhow::Result<bool> {
        let f = self
            .features
            .iter()
            .find(|f| f.id() == id)
            .ok_or_else(|| anyhow::anyhow!("unknown feature {id}"))?;
        if self.is_enabled(id) {
            f.stop();
            self.enabled.lock().unwrap().remove(f.id());
            return Ok(false);
        }
        for p in f.required_permissions() {
            if !crate::platform::permission_granted(p) {
                crate::platform::request_permission(p);
                anyhow::bail!("{:?} permission missing", p);
            }
        }
        f.start()?;
        self.enabled.lock().unwrap().insert(f.id());
        Ok(true)
    }
}
