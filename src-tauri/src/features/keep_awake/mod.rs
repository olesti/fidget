use std::sync::Mutex;

use crate::{feature::Feature, i18n::t};

// ponytail: indefinite only; add a duration setting when asked.
#[derive(Default)]
pub struct KeepAwake {
    guard: Mutex<Option<keepawake::KeepAwake>>,
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
    }

    fn status(&self) -> String {
        t(if self.guard.lock().unwrap().is_some() { "awake.on" } else { "awake.off" }).into()
    }
}
