use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{feature::Feature, i18n::t};

#[derive(Default)]
pub struct KeepAwake {
    guard: Mutex<Option<keepawake::KeepAwake>>,
    deadline: Mutex<Option<Instant>>,
}

impl Feature for KeepAwake {
    fn id(&self) -> &'static str {
        "keep_awake"
    }

    fn name(&self) -> &'static str {
        t("name.keep_awake")
    }

    fn shortcut(&self) -> Option<&'static str> {
        Some("CmdOrControl+Alt+K")
    }

    fn timer_supported(&self) -> bool {
        true
    }

    fn set_timer(&self, d: Option<Duration>) {
        *self.deadline.lock().unwrap() = d.map(|d| Instant::now() + d);
    }

    fn remaining(&self) -> Option<u64> {
        // ceil, so the last second still shows "1" and 0 means expired
        self.deadline.lock().unwrap().map(|d| d.saturating_duration_since(Instant::now()).as_secs_f64().ceil() as u64)
    }

    fn start(&self) -> anyhow::Result<()> {
        let g = keepawake::Builder::default()
            .display(true)
            .idle(true)
            .reason("Fidget Keep Awake")
            .app_name("Fidget")
            .app_reverse_domain("com.fidget.app")
            .create()?;
        *self.guard.lock().unwrap() = Some(g);
        Ok(())
    }

    fn stop(&self) {
        self.guard.lock().unwrap().take();
        self.deadline.lock().unwrap().take();
    }

    fn status(&self) -> String {
        if self.guard.lock().unwrap().is_none() {
            return t("awake.off").into();
        }
        match self.remaining() {
            Some(s) => t("awake.timer").replace("{}", &s.div_ceil(60).to_string()),
            None => t("awake.on").into(),
        }
    }
}
