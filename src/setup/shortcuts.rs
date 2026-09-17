use crate::{MainWindow, ShortcutEntry};
use crate::db;
use slint::{ComponentHandle, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type ShortcutMap = Rc<RefCell<HashMap<String, String>>>;

pub const ACTION_LIST: &[(&str, &str, &str)] = &[
    ("goto-home", "Go to Dashboard", "Alt+0"),
    ("goto-reports", "Go to Reports", "Alt+1"),
    ("goto-menu", "Go to Menu", "Alt+2"),
    ("goto-settings", "Go to Settings", "Alt+3"),
    ("close-dialog", "Close dialog / return to Dashboard", "Esc"),
];

pub fn default_shortcuts() -> HashMap<String, String> {
    ACTION_LIST
        .iter()
        .map(|(action, _, def)| (action.to_string(), def.to_string()))
        .collect()
}

pub fn load_shortcuts(conn: &rusqlite::Connection) -> ShortcutMap {
    let mut map = default_shortcuts();
    for (action, _, _) in ACTION_LIST {
        let key = format!("shortcut.{}", action);
        if let Ok(Some(v)) = db::settings::get_setting(conn, &key) {
            map.insert(action.to_string(), v);
        }
    }
    Rc::new(RefCell::new(map))
}

pub fn refresh_shortcut_entries(window: &MainWindow, map: &ShortcutMap) {
    let borrowed = map.borrow();
    let entries: Vec<ShortcutEntry> = ACTION_LIST
        .iter()
        .map(|(action, desc, _)| ShortcutEntry {
            action: (*action).into(),
            description: (*desc).into(),
            combo: borrowed
                .get(*action)
                .cloned()
                .unwrap_or_default()
                .into(),
        })
        .collect();
    let model = Rc::new(VecModel::from(entries));
    window.set_shortcut_entries(model.into());
}

fn normalize_combo(
    text: &str,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if ctrl {
        parts.push("Ctrl".into());
    }
    if shift {
        parts.push("Shift".into());
    }
    if alt {
        parts.push("Alt".into());
    }
    if meta {
        parts.push("Meta".into());
    }
    let key = if text.is_empty() {
        String::new()
    } else {
        text.to_uppercase()
    };
    parts.push(key);
    parts.join("+")
}

pub fn setup_shortcuts(
    window: &MainWindow,
    conn: Rc<rusqlite::Connection>,
    map: ShortcutMap,
) {
    // ==================== KEY HANDLER ====================
    let weak = window.as_weak();
    let map_key = map.clone();
    window.on_key_pressed(move |text, ctrl, shift, alt, meta| {
        let Some(window) = weak.upgrade() else {
            return false;
        };
        let text = text.to_string();
        let combo = normalize_combo(&text, ctrl, shift, alt, meta);

        // ---- Capture dialog intercept ----
        if window.get_shortcut_capture_open() {
            if text == "\u{1b}" {
                window.set_shortcut_capture_open(false);
                return true;
            }
            if text.is_empty() {
                return true;
            }
            window.set_shortcut_capture_new(combo.into());
            return true;
        }

        // ---- Esc handling ----
        if text == "\u{1b}" {
            if window.get_quit_dialog_open() {
                window.set_quit_dialog_open(false);
            } else if window.get_note_dialog_open() {
                window.set_note_dialog_open(false);
            } else if window.get_void_dialog_open() {
                window.set_void_dialog_open(false);
            } else if window.get_unpaid_dialog_open() {
                window.set_unpaid_dialog_open(false);
            } else if window.get_export_dialog_open() {
                window.set_export_dialog_open(false);
            } else if window.get_sidebar_open() {
                window.set_sidebar_open(false);
            } else if window.get_current_view() != 4
                && window.get_current_view() != 0
            {
                window.set_current_view(0);
            }
            return true;
        }

        // ---- Alt+F: focus search box ----
        if combo == "Alt+F" {
            let view = window.get_current_view();
            if view == 1 || view == 2 || view == 4 {
                window.set_focus_search_requested(true);
            }
            return true;
        }

        // ---- Lookup action from map ----
        let matched_action: Option<String> = {
            let borrowed = map_key.borrow();
            borrowed
                .iter()
                .find(|(_, v)| v.as_str() == combo)
                .map(|(k, _)| k.clone())
        };

        match matched_action.as_deref() {
            Some("goto-home") => {
                window.set_current_view(0);
                true
            }
            Some("goto-reports") => {
                window.set_current_view(1);
                true
            }
            Some("goto-menu") => {
                window.set_current_view(2);
                true
            }
            Some("goto-settings") => {
                window.set_current_view(3);
                true
            }
            _ => false,
        }
    });

    // ==================== START CAPTURE ====================
    let weak = window.as_weak();
    window.on_start_shortcut_capture(move |action, description, current| {
        if let Some(window) = weak.upgrade() {
            window.set_shortcut_capture_action(action);
            window.set_shortcut_capture_description(description);
            window.set_shortcut_capture_current(current);
            window.set_shortcut_capture_new("".into());
            window.set_shortcut_capture_open(true);
        }
    });

    // ==================== APPLY CAPTURE ====================
    let weak = window.as_weak();
    let conn_apply = conn.clone();
    let map_apply = map.clone();
    window.on_apply_shortcut_capture(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let action = window.get_shortcut_capture_action().to_string();
        let new_combo = window.get_shortcut_capture_new().to_string();
        if new_combo.is_empty() {
            return;
        }

        map_apply
            .borrow_mut()
            .insert(action.clone(), new_combo.clone());

        let key = format!("shortcut.{}", action);
        let _ = db::settings::set_setting(&conn_apply, &key, &new_combo);

        window.set_shortcut_capture_open(false);
        refresh_shortcut_entries(&window, &map_apply);
    });

    // ==================== CANCEL CAPTURE ====================
    let weak = window.as_weak();
    window.on_cancel_shortcut_capture(move || {
        if let Some(window) = weak.upgrade() {
            window.set_shortcut_capture_open(false);
        }
    });

    // ==================== RESET TO DEFAULT ====================
    let weak = window.as_weak();
    window.on_reset_shortcut_capture(move || {
        if let Some(window) = weak.upgrade() {
            let action = window.get_shortcut_capture_action().to_string();
            if let Some((_, _, def)) = ACTION_LIST
                .iter()
                .find(|(a, _, _)| *a == action.as_str())
            {
                window.set_shortcut_capture_new((*def).into());
            }
        }
    });
}
