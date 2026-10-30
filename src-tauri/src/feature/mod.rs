use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Permission {
    Accessibility,
}

impl Permission {
    pub fn id(self) -> &'static str {
        match self {
            Permission::Accessibility => "accessibility",
        }
    }
}

/// Every feature implements this. `start` must return quickly (spawn a task).
// ponytail: sync trait, features spawn their own tasks; async_trait if start/stop ever need to await.
pub trait Feature: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// Settings form fields: `[{key,label,kind:"number"|"select",value,options?}]`.
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!([])
    }
    /// Current config values, persisted as-is.
    fn config(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    fn set_config(&self, _: serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }
    /// Default global shortcut, e.g. "CmdOrControl+Alt+J".
    fn shortcut(&self) -> Option<&'static str> {
        None
    }
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
    paused_until: Mutex<Option<Instant>>,
}

impl FeatureManager {
    pub fn new(features: Vec<Arc<dyn Feature>>) -> Self {
        Self { features, enabled: Mutex::default(), paused_until: Mutex::default() }
    }

    pub fn features(&self) -> &[Arc<dyn Feature>] {
        &self.features
    }

    pub fn is_enabled(&self, id: &str) -> bool {
        self.enabled.lock().unwrap().contains(id)
    }

    pub fn enabled_ids(&self) -> Vec<&'static str> {
        self.enabled.lock().unwrap().iter().copied().collect()
    }

    pub fn is_paused(&self) -> bool {
        self.paused_until.lock().unwrap().is_some()
    }

    /// Stop every enabled feature for `d`; they stay "enabled" and restart on resume.
    pub fn pause(&self, d: Duration) {
        for f in self.running() {
            f.stop();
        }
        *self.paused_until.lock().unwrap() = Some(Instant::now() + d);
    }

    pub fn resume(&self) {
        if self.paused_until.lock().unwrap().take().is_some() {
            for f in self.running() {
                if let Err(e) = f.start() {
                    tracing::warn!("resume {}: {e}", f.id());
                }
            }
        }
    }

    /// Call periodically: ends an expired pause.
    pub fn tick(&self) {
        let expired = self.paused_until.lock().unwrap().is_some_and(|t| Instant::now() >= t);
        if expired {
            self.resume();
        }
    }

    pub fn set_config(&self, id: &str, v: serde_json::Value) -> anyhow::Result<()> {
        let f = self.find(id)?;
        f.set_config(v)?;
        if self.is_enabled(id) && !self.is_paused() {
            f.stop();
            f.start()?;
        }
        Ok(())
    }

    fn find(&self, id: &str) -> anyhow::Result<&Arc<dyn Feature>> {
        self.features.iter().find(|f| f.id() == id).ok_or_else(|| anyhow::anyhow!("unknown feature {id}"))
    }

    fn running(&self) -> Vec<Arc<dyn Feature>> {
        let on = self.enabled.lock().unwrap();
        self.features.iter().filter(|f| on.contains(f.id())).cloned().collect()
    }

    pub fn any_enabled(&self) -> bool {
        !self.enabled.lock().unwrap().is_empty()
    }

    /// Flip a feature on/off. Returns the resulting state.
    pub fn toggle(&self, id: &str) -> anyhow::Result<bool> {
        let f = self.find(id)?;
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
        if !self.is_paused() {
            f.start()?;
        }
        self.enabled.lock().unwrap().insert(f.id());
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Default)]
    struct Fake(AtomicBool);
    impl Feature for Fake {
        fn id(&self) -> &'static str { "fake" }
        fn name(&self) -> &'static str { "Fake" }
        fn start(&self) -> anyhow::Result<()> { self.0.store(true, Ordering::SeqCst); Ok(()) }
        fn stop(&self) { self.0.store(false, Ordering::SeqCst) }
        fn status(&self) -> String { String::new() }
    }

    #[test]
    fn pause_stops_and_resume_restarts() {
        let fake = Arc::new(Fake::default());
        let m = FeatureManager::new(vec![fake.clone()]);
        assert!(m.toggle("fake").unwrap());
        assert!(fake.0.load(Ordering::SeqCst));
        m.pause(Duration::from_secs(60));
        assert!(!fake.0.load(Ordering::SeqCst) && m.is_enabled("fake"));
        m.resume();
        assert!(fake.0.load(Ordering::SeqCst) && !m.is_paused());
        m.pause(Duration::ZERO);
        m.tick(); // expired pause ends by itself
        assert!(fake.0.load(Ordering::SeqCst));
    }
}
