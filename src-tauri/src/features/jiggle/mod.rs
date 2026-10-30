use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use enigo::{Coordinate, Enigo, Mouse, Settings};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::{self, JoinHandle};

use crate::feature::{Feature, Permission};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Always,
    IdleOnly,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JiggleConfig {
    pub interval_min: u64,
    pub interval_max: u64,
    pub mode: Mode,
    pub idle_threshold: u64,
    pub distance: i32,
}

impl Default for JiggleConfig {
    fn default() -> Self {
        Self { interval_min: 30, interval_max: 120, mode: Mode::IdleOnly, idle_threshold: 20, distance: 2 }
    }
}

// ponytail: only the `subtle` style, no schedule; add `natural`/`zen`/`schedule` in v0.2.
#[derive(Default)]
pub struct Jiggle {
    config: Mutex<JiggleConfig>,
    task: Mutex<Option<JoinHandle<()>>>,
    last_move: Arc<AtomicU64>, // unix secs, 0 = never
}

impl Jiggle {
    pub fn new() -> Self {
        Self::default()
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn next_delay(c: &JiggleConfig) -> Duration {
    let (lo, hi) = (c.interval_min.min(c.interval_max), c.interval_min.max(c.interval_max));
    Duration::from_secs(rand::rng().random_range(lo..=hi))
}

/// Nudge the cursor by `distance` px and put it back.
fn nudge(distance: i32) -> anyhow::Result<()> {
    let mut e = Enigo::new(&Settings::default())?;
    let (x, y) = e.location()?;
    e.move_mouse(distance, 0, Coordinate::Rel)?;
    e.move_mouse(x, y, Coordinate::Abs)?;
    Ok(())
}

impl Feature for Jiggle {
    fn id(&self) -> &'static str {
        "jiggle"
    }

    fn name(&self) -> &'static str {
        "Jiggle"
    }

    fn config_schema(&self) -> serde_json::Value {
        let c = self.config.lock().unwrap().clone();
        let num = |key, label, value: u64| serde_json::json!({"key": key, "label": label, "kind": "number", "value": value});
        serde_json::json!([
            num("interval_min", "Min interval (s)", c.interval_min),
            num("interval_max", "Max interval (s)", c.interval_max),
            {"key": "mode", "label": "Mode", "kind": "select", "value": c.mode, "options": ["idle_only", "always"]},
            num("idle_threshold", "Idle threshold (s)", c.idle_threshold),
            num("distance", "Distance (px)", c.distance as u64),
        ])
    }

    fn config(&self) -> serde_json::Value {
        serde_json::to_value(&*self.config.lock().unwrap()).unwrap_or_default()
    }

    fn set_config(&self, v: serde_json::Value) -> anyhow::Result<()> {
        let mut c: JiggleConfig = serde_json::from_value(v)?;
        c.interval_min = c.interval_min.max(1);
        c.interval_max = c.interval_max.max(1);
        c.distance = c.distance.clamp(1, 50);
        *self.config.lock().unwrap() = c;
        Ok(())
    }

    fn shortcut(&self) -> Option<&'static str> {
        Some("CmdOrControl+Alt+J")
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::Accessibility]
    }

    fn start(&self) -> anyhow::Result<()> {
        let c = self.config.lock().unwrap().clone();
        let last = self.last_move.clone();
        let handle = async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(next_delay(&c)).await;
                if c.mode == Mode::IdleOnly {
                    match user_idle::UserIdle::get_time() {
                        Ok(i) if i.as_seconds() < c.idle_threshold => continue,
                        Err(e) => {
                            tracing::warn!("idle query failed: {e}");
                            continue;
                        }
                        _ => {}
                    }
                }
                let d = c.distance;
                match async_runtime::spawn_blocking(move || nudge(d)).await {
                    Ok(Ok(())) => last.store(now(), Ordering::Relaxed),
                    Ok(Err(e)) => tracing::warn!("jiggle failed: {e}"),
                    Err(e) => tracing::warn!("jiggle task failed: {e}"),
                }
            }
        });
        *self.task.lock().unwrap() = Some(handle);
        Ok(())
    }

    fn stop(&self) {
        if let Some(h) = self.task.lock().unwrap().take() {
            h.abort();
        }
    }

    fn status(&self) -> String {
        match self.last_move.load(Ordering::Relaxed) {
            0 => "Last move: never".into(),
            t => format!("Last move: {} s ago", now().saturating_sub(t)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_stays_in_bounds_even_if_min_gt_max() {
        let c = JiggleConfig { interval_min: 50, interval_max: 10, ..Default::default() };
        for _ in 0..200 {
            let s = next_delay(&c).as_secs();
            assert!((10..=50).contains(&s));
        }
    }
}
