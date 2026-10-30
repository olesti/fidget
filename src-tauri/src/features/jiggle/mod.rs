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

use crate::{feature::{Feature, Permission}, i18n::t};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Always,
    IdleOnly,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    /// Small step and back.
    Subtle,
    /// Soft curve out and back with a little jitter.
    Natural,
    /// Cursor stays put; only a zero-distance move event is sent.
    Zen,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct JiggleConfig {
    pub style: Style,
    pub schedule_enabled: bool,
    /// "HH:MM" local time.
    pub schedule_start: String,
    pub schedule_end: String,
    pub weekdays_only: bool,
    pub interval_min: u64,
    pub interval_max: u64,
    pub mode: Mode,
    pub idle_threshold: u64,
    pub distance: i32,
}

impl Default for JiggleConfig {
    fn default() -> Self {
        Self {
            style: Style::Subtle,
            schedule_enabled: false,
            schedule_start: "09:00".into(),
            schedule_end: "18:00".into(),
            weekdays_only: true,
            interval_min: 30,
            interval_max: 120,
            mode: Mode::IdleOnly,
            idle_threshold: 20,
            distance: 2,
        }
    }
}

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

fn parse_hm(s: &str) -> Option<u32> {
    let (h, m) = s.split_once(':')?;
    Some(h.parse::<u32>().ok()? * 60 + m.parse::<u32>().ok()?)
}

/// `minute` = minutes since local midnight; `weekday` 0 = Monday. Window may wrap past midnight.
fn in_schedule(c: &JiggleConfig, weekday: u32, minute: u32) -> bool {
    if !c.schedule_enabled {
        return true;
    }
    if c.weekdays_only && weekday >= 5 {
        return false;
    }
    let (Some(a), Some(b)) = (parse_hm(&c.schedule_start), parse_hm(&c.schedule_end)) else { return true };
    if a <= b { (a..b).contains(&minute) } else { minute >= a || minute < b }
}

fn schedule_ok(c: &JiggleConfig) -> bool {
    use chrono::{Datelike, Local, Timelike};
    let n = Local::now();
    in_schedule(c, n.weekday().num_days_from_monday(), n.hour() * 60 + n.minute())
}

/// Move the cursor according to `style` and put it back where it was.
fn nudge(c: &JiggleConfig) -> anyhow::Result<()> {
    let mut e = Enigo::new(&Settings::default())?;
    let (x, y) = e.location()?;
    match c.style {
        Style::Subtle => {
            e.move_mouse(c.distance, 0, Coordinate::Rel)?;
            e.move_mouse(x, y, Coordinate::Abs)?;
        }
        Style::Natural => {
            let (a, steps) = (c.distance as f64 * 5.0, 10);
            let mut rng = rand::rng();
            for i in 1..steps {
                let t = i as f64 / steps as f64;
                let dx = (a * (std::f64::consts::PI * t).sin()).round() as i32;
                let dy = (a * 0.5 * (2.0 * std::f64::consts::PI * t).sin()).round() as i32 + rng.random_range(-1..=1);
                e.move_mouse(x + dx, y + dy, Coordinate::Abs)?;
                std::thread::sleep(Duration::from_millis(rng.random_range(10..25)));
            }
            e.move_mouse(x, y, Coordinate::Abs)?;
        }
        Style::Zen => e.move_mouse(0, 0, Coordinate::Rel)?,
    }
    Ok(())
}

impl Feature for Jiggle {
    fn id(&self) -> &'static str {
        "jiggle"
    }

    fn name(&self) -> &'static str {
        t("name.jiggle")
    }

    fn config_schema(&self) -> serde_json::Value {
        let c = self.config.lock().unwrap().clone();
        let f = |key: &str, kind: &str, value: serde_json::Value| {
            serde_json::json!({"key": key, "label": t(&format!("f.{key}")), "kind": kind, "value": value})
        };
        let mut style = f("style", "select", serde_json::json!(c.style));
        style["options"] = serde_json::json!(["subtle", "natural", "zen"]);
        let mut mode = f("mode", "select", serde_json::json!(c.mode));
        mode["options"] = serde_json::json!(["idle_only", "always"]);
        serde_json::json!([
            style,
            f("interval_min", "number", c.interval_min.into()),
            f("interval_max", "number", c.interval_max.into()),
            mode,
            f("idle_threshold", "number", c.idle_threshold.into()),
            f("distance", "number", c.distance.into()),
            f("schedule_enabled", "bool", c.schedule_enabled.into()),
            f("schedule_start", "time", c.schedule_start.into()),
            f("schedule_end", "time", c.schedule_end.into()),
            f("weekdays_only", "bool", c.weekdays_only.into()),
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
                if !schedule_ok(&c) {
                    continue;
                }
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
                let cfg = c.clone();
                match async_runtime::spawn_blocking(move || nudge(&cfg)).await {
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
            0 => t("jiggle.last_never").into(),
            at => t("jiggle.last_ago").replace("{}", &now().saturating_sub(at).to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_window() {
        let c = JiggleConfig { schedule_enabled: true, ..Default::default() }; // 09:00-18:00 weekdays
        assert!(in_schedule(&c, 0, 9 * 60) && !in_schedule(&c, 0, 18 * 60) && !in_schedule(&c, 5, 12 * 60));
        let night = JiggleConfig { schedule_start: "22:00".into(), schedule_end: "06:00".into(), weekdays_only: false, ..c };
        assert!(in_schedule(&night, 6, 23 * 60) && in_schedule(&night, 6, 60) && !in_schedule(&night, 6, 12 * 60));
    }

    #[test]
    fn delay_stays_in_bounds_even_if_min_gt_max() {
        let c = JiggleConfig { interval_min: 50, interval_max: 10, ..Default::default() };
        for _ in 0..200 {
            let s = next_delay(&c).as_secs();
            assert!((10..=50).contains(&s));
        }
    }
}
