use std::sync::atomic::{AtomicU8, Ordering};

use serde_json::{Map, Value};

static LANG: AtomicU8 = AtomicU8::new(0); // 0 = en, 1 = tr

/// (key, English, Türkçe). Settings UI gets this whole table via `dump()`.
const STRINGS: &[(&str, &str, &str)] = &[
    ("name.jiggle", "Jiggle", "Jiggle"),
    ("name.keep_awake", "Keep Awake", "Uyanık Tut"),
    ("jiggle.last_never", "Last move: never", "Son hareket: hiç"),
    ("jiggle.last_ago", "Last move: {} s ago", "Son hareket: {} sn önce"),
    ("awake.on", "Preventing sleep", "Uyku engelleniyor"),
    ("awake.off", "Off", "Kapalı"),
    ("f.style", "Style", "Stil"),
    ("f.interval_min", "Min interval (s)", "En kısa aralık (sn)"),
    ("f.interval_max", "Max interval (s)", "En uzun aralık (sn)"),
    ("f.mode", "Mode", "Mod"),
    ("f.idle_threshold", "Idle threshold (s)", "Boşta sayılma (sn)"),
    ("f.distance", "Distance (px)", "Mesafe (px)"),
    ("f.schedule_enabled", "Only on a schedule", "Sadece belirli saatlerde"),
    ("f.schedule_start", "From", "Başlangıç"),
    ("f.schedule_end", "To", "Bitiş"),
    ("f.weekdays_only", "Weekdays only", "Sadece hafta içi"),
    ("opt.idle_only", "Only when idle", "Sadece boştayken"),
    ("opt.always", "Always", "Her zaman"),
    ("opt.subtle", "Subtle", "Hafif"),
    ("opt.natural", "Natural", "Doğal"),
    ("opt.zen", "Zen (cursor stays)", "Zen (imleç yerinde)"),
    ("tray.pause", "Pause All", "Tümünü Duraklat"),
    ("tray.p15", "for 15 minutes", "15 dakika"),
    ("tray.p60", "for 1 hour", "1 saat"),
    ("tray.ptomorrow", "until tomorrow", "yarına kadar"),
    ("tray.resume", "Resume", "Devam Et"),
    ("tray.settings", "Settings…", "Ayarlar…"),
    ("tray.autostart", "Launch at Login", "Açılışta Başlat"),
    ("tray.update", "Check for Updates…", "Güncellemeleri Denetle…"),
    ("tray.quit", "Quit Fidget", "Fidget'tan Çık"),
    ("ui.tab.general", "General", "Genel"),
    ("ui.tab.features", "Features", "Özellikler"),
    ("ui.tab.shortcuts", "Shortcuts", "Kısayollar"),
    ("ui.tab.permissions", "Permissions", "İzinler"),
    ("ui.tab.about", "About", "Hakkında"),
    ("ui.language", "Language", "Dil"),
    ("ui.theme", "Theme", "Tema"),
    ("ui.theme.system", "System", "Sistem"),
    ("ui.theme.light", "Light", "Açık"),
    ("ui.theme.dark", "Dark", "Koyu"),
    ("ui.autostart", "Launch at login", "Açılışta başlat"),
    ("ui.save", "Save", "Kaydet"),
    ("ui.saved", "Saved ✓", "Kaydedildi ✓"),
    ("ui.paused", "⏸ Paused — resume from the menu bar", "⏸ Duraklatıldı — menü çubuğundan devam ettirin"),
    ("ui.shortcut_hint", "Click, then press the keys. Backspace clears.", "Tıklayın, sonra tuşlara basın. Backspace temizler."),
    ("ui.shortcut_none", "None", "Yok"),
    ("ui.granted", "Granted", "Verildi"),
    ("ui.missing", "Not granted", "Verilmedi"),
    ("ui.grant", "Grant…", "İzin ver…"),
    ("ui.no_perms", "No permissions needed.", "Gerekli izin yok."),
    ("ui.perm.accessibility", "Accessibility (mouse control)", "Erişilebilirlik (fare kontrolü)"),
    ("ui.version", "Version", "Sürüm"),
    ("ui.license", "License: MIT", "Lisans: MIT"),
    ("ui.update.check", "Check for updates", "Güncellemeleri denetle"),
    ("ui.update.checking", "Checking…", "Denetleniyor…"),
    ("ui.update.uptodate", "You're up to date.", "Güncelsiniz."),
    ("ui.update.available", "Version {} is available.", "{} sürümü mevcut."),
    ("ui.update.install", "Install and restart", "Yükle ve yeniden başlat"),
    ("ui.update.installing", "Downloading…", "İndiriliyor…"),
    ("ui.open_logs", "Open logs folder", "Log klasörünü aç"),
];

pub fn init() {
    let tr = sys_locale::get_locale().is_some_and(|l| l.to_lowercase().starts_with("tr"));
    LANG.store(tr as u8, Ordering::Relaxed);
}

pub fn lang() -> &'static str {
    if LANG.load(Ordering::Relaxed) == 1 { "tr" } else { "en" }
}

pub fn set_lang(l: &str) {
    LANG.store((l == "tr") as u8, Ordering::Relaxed);
}

pub fn t(key: &str) -> &'static str {
    let tr = LANG.load(Ordering::Relaxed) == 1;
    STRINGS.iter().find(|s| s.0 == key).map_or("?", |s| if tr { s.2 } else { s.1 })
}

pub fn dump() -> Value {
    Value::Object(STRINGS.iter().map(|s| (s.0.to_string(), Value::from(t(s.0)))).collect::<Map<_, _>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_languages() {
        assert!(STRINGS.iter().all(|s| !s.1.is_empty() && !s.2.is_empty()));
        set_lang("tr");
        assert_eq!(t("tray.quit"), "Fidget'tan Çık");
        set_lang("en");
        assert_eq!(t("tray.quit"), "Quit Fidget");
    }
}
