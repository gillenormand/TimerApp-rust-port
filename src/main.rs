#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::rc::Rc;
use std::cell::RefCell;
use slint::{ModelRc, VecModel, Timer, TimerMode};

fn format_time(seconds: i32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{}:{:02}:{:02}", hours, minutes, secs)
}

fn parse_float(s: &str) -> Option<f64> {
    let cleaned = s.trim().replace(',', ".");
    if cleaned.is_empty() {
        return Some(0.0);
    }
    cleaned.parse::<f64>().ok()
}

fn parse_int_seconds(s: &str) -> Option<i32> {
    let cleaned = s.trim();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<i32>().ok()
}

struct AppState {
    entries: HashMap<String, i32>,
    current_entry: Option<String>,
    current_time: i32,
    session_time: i32,
    timer_running: bool,
    last_entry: Option<String>,
}

fn main() -> Result<(), slint::PlatformError> {
    std::env::set_var("SLINT_BACKEND", "winit-femtovg");
    let ui = AppWindow::new()?;

    // Center the window on the primary monitor
    ui.show()?;
    let screen_w = 1920;
    let screen_h = 1080;
    let win_size = ui.window().size();
    let x = (screen_w - win_size.width as i32) / 2;
    let y = (screen_h - win_size.height as i32) / 2;
    ui.window().set_position(slint::PhysicalPosition::new(x, y));
  
    let state = Rc::new(RefCell::new(AppState {
        entries: HashMap::new(),
        current_entry: None,
        current_time: 0,
        session_time: 0,
        timer_running: false,
        last_entry: None,
    }));

    // ---------- Load apps.json ----------
    if let Ok(mut file) = File::open("apps.json") {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(entries) = serde_json::from_str::<HashMap<String, i32>>(&contents) {
                state.borrow_mut().entries = entries;
            }
        }
    }

    // ---------- Load settings.json ----------
    if let Ok(mut file) = File::open("settings.json") {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(last) = settings.get("lastEntry").and_then(|v| v.as_str()) {
                    state.borrow_mut().last_entry = Some(last.to_string());
                }
            }
        }
    }

    // ---------- Helper: refresh entries list on screen ----------
    let update_ui_list = {
        let ui_weak = ui.as_weak();
        let state_clone = state.clone();
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                let slint_entries: Vec<AppEntry> = {
                    let st = state_clone.borrow();
                    let mut sorted_entries: Vec<(String, i32)> =
                        st.entries.clone().into_iter().collect();
                    sorted_entries.sort_by(|a, b| b.1.cmp(&a.1));
                    sorted_entries
                        .into_iter()
                        .map(|(name, time)| AppEntry {
                            name: name.clone().into(),
                            time,
                            time_str: format_time(time).into(),
                        })
                        .collect()
                };

                let model = Rc::new(VecModel::from(slint_entries));
                ui.set_entries_list(ModelRc::from(model));
            }
        }
    };

    // ---------- Restore last selected entry ----------
    let last_entry_opt = { state.borrow().last_entry.clone() };

    if let Some(last) = last_entry_opt {
        let current_time = {
            let st = state.borrow();
            if st.entries.contains_key(&last) {
                Some(*st.entries.get(&last).unwrap_or(&0))
            } else {
                None
            }
        };

        if let Some(ct) = current_time {
            {
                let mut st = state.borrow_mut();
                st.current_entry = Some(last.clone());
                st.current_time = ct;
            }

            ui.set_current_entry_name(last.into());
            ui.set_main_timer_str(format_time(ct).into());
            ui.set_has_selected_entry(true);
        }
    }

    update_ui_list();

    // ---------- Save helper ----------
    let save_entries_fn = {
        let state_clone = state.clone();
        move || {
            let json = {
                let st = state_clone.borrow();
                serde_json::to_string_pretty(&st.entries).ok()
            };
            if let Some(json) = json {
                if let Ok(mut file) = File::create("apps.json") {
                    let _ = file.write_all(json.as_bytes());
                }
            }
        }
    };

    // ---------- Timer + save counter ----------
    let main_timer = Timer::default();
    let save_counter = Rc::new(RefCell::new(0u32));

    // ---------- Entry selected ----------
    let ui_weak_select = ui.as_weak();
    let state_select = state.clone();
    ui.on_entry_selected(move |name| {
        let Some(ui) = ui_weak_select.upgrade() else { return; };
        let name_str = name.to_string();

        let current_time = {
            let mut st = state_select.borrow_mut();
            st.current_entry = Some(name_str.clone());
            st.current_time = *st.entries.get(&name_str).unwrap_or(&0);
            st.session_time = 0;
            st.current_time
        };

        ui.set_current_entry_name(name.clone());
        ui.set_main_timer_str(format_time(current_time).into());
        ui.set_session_timer_str("Session: 0:00:00".into());
        ui.set_has_selected_entry(true);
    });

    // ---------- Start ----------
    let ui_weak_start = ui.as_weak();
    let state_start = state.clone();
    let save_entries_clone = save_entries_fn.clone();
    let update_list_start = update_ui_list.clone();
    let save_counter_start = save_counter.clone();
    ui.on_start_clicked(move || {
        let Some(ui) = ui_weak_start.upgrade() else { return; };

        let started_name = {
            let mut st = state_start.borrow_mut();
            if st.timer_running {
                return;
            }
            st.timer_running = true;

            // Remember last *started* entry
            if let Some(current) = st.current_entry.clone() {
                st.last_entry = Some(current.clone());
                Some(current)
            } else {
                None
            }
        };

        // Persist last-started entry to settings.json
        if let Some(name) = started_name {
            if let Ok(mut file) = File::create("settings.json") {
                let json = serde_json::json!({ "lastEntry": name });
                let _ = file.write_all(json.to_string().as_bytes());
            }
        }

        ui.set_timer_running(true);

        let ui_tick_weak = ui_weak_start.clone();
        let state_tick_clone = state_start.clone();
        let save_clone = save_entries_clone.clone();
        let update_list_tick = update_list_start.clone();
        let save_counter_tick = save_counter_start.clone();

        main_timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            move || {
                let (running, current_time, session_time) = {
                    let mut st = state_tick_clone.borrow_mut();
                    if !st.timer_running {
                        (false, 0, 0)
                    } else {
                        st.current_time += 1;
                        st.session_time += 1;
                        if let Some(current) = st.current_entry.clone() {
                            let ct = st.current_time;
                            st.entries.insert(current, ct);
                        }
                        (true, st.current_time, st.session_time)
                    }
                };

                if !running {
                    return;
                }

                if let Some(ui) = ui_tick_weak.upgrade() {
                    ui.set_main_timer_str(format_time(current_time).into());
                    ui.set_session_timer_str(format!(
                        "Session: {}",
                        format_time(session_time)
                    ).into());
                }

                update_list_tick();

                // Save only every 10 seconds
                let should_save = {
                    let mut counter = save_counter_tick.borrow_mut();
                    *counter += 1;
                    if *counter >= 10 {
                        *counter = 0;
                        true
                    } else {
                        false
                    }
                };
                if should_save {
                    save_clone();
                }
            },
        );
    });

    // ---------- Pause ----------
    let ui_weak_pause = ui.as_weak();
    let state_pause = state.clone();
    let save_pause = save_entries_fn.clone();
    ui.on_pause_clicked(move || {
        let Some(ui) = ui_weak_pause.upgrade() else { return; };
        {
            let mut st = state_pause.borrow_mut();
            st.timer_running = false;
        }
        ui.set_timer_running(false);
        // Flush to disk on pause
        save_pause();
    });

    // ---------- Add New: open dialog (mode 0) ----------
    let ui_weak_add = ui.as_weak();
    ui.on_add_entry_clicked(move || {
        let Some(ui) = ui_weak_add.upgrade() else { return; };
        ui.set_dialog_mode(0);
        ui.set_dialog_title("Add Entry".into());
        ui.set_dialog_prompt("Enter entry name:".into());
        ui.set_dialog_input("".into());
        ui.set_dialog_error("".into());
        ui.set_dialog_visible(true);
    });

    // ---------- Remove current entry ----------
    let ui_weak_remove = ui.as_weak();
    let state_remove = state.clone();
    let update_list_remove = update_ui_list.clone();
    let save_remove = save_entries_fn.clone();
    ui.on_remove_entry_clicked(move || {
        let Some(ui) = ui_weak_remove.upgrade() else { return; };

        let removed = {
            let mut st = state_remove.borrow_mut();
            if let Some(current) = st.current_entry.clone() {
                st.entries.remove(&current);
                if st.last_entry.as_deref() == Some(current.as_str()) {
                    st.last_entry = None;
                }
                st.current_entry = None;
                st.timer_running = false;
                true
            } else {
                false
            }
        };

        if !removed {
            return;
        }

        ui.set_timer_running(false);
        ui.set_has_selected_entry(false);
        ui.set_current_entry_name("!!!pick an entry!!!".into());
        ui.set_main_timer_str("0:00:00".into());
        ui.set_session_timer_str("Session: 0:00:00".into());

        save_remove();
        update_list_remove();
    });

    // ---------- +/- Min: open dialog (mode 2) ----------
    let ui_weak_min = ui.as_weak();
    let state_min = state.clone();
    ui.on_add_minutes_clicked(move || {
        let Some(ui) = ui_weak_min.upgrade() else { return; };
        if state_min.borrow().current_entry.is_none() {
            return;
        }
        ui.set_dialog_mode(2);
        ui.set_dialog_title("+/- minutes".into());
        ui.set_dialog_prompt("Enter minutes (e.g. -5.5 or -5,5):".into());
        ui.set_dialog_input("".into());
        ui.set_dialog_error("".into());
        ui.set_dialog_visible(true);
    });

    // ---------- Edit Time: open dialog (mode 1) ----------
    let ui_weak_edit = ui.as_weak();
    let state_edit = state.clone();
    ui.on_edit_time_clicked(move || {
        let Some(ui) = ui_weak_edit.upgrade() else { return; };
        let current = {
            let st = state_edit.borrow();
            if let Some(name) = st.current_entry.clone() {
                Some(*st.entries.get(&name).unwrap_or(&0))
            } else {
                None
            }
        };
        let Some(cur) = current else { return; };
        ui.set_dialog_mode(1);
        ui.set_dialog_title("Edit Time".into());
        ui.set_dialog_prompt(format!("Enter new time in seconds (current: {}):", cur).into());
        ui.set_dialog_input(cur.to_string().into());
        ui.set_dialog_error("".into());
        ui.set_dialog_visible(true);
    });

    // ---------- Dialog: Cancel ----------
    let ui_weak_cancel = ui.as_weak();
    ui.on_dialog_cancelled(move || {
        let Some(ui) = ui_weak_cancel.upgrade() else { return; };
        ui.set_dialog_visible(false);
    });

    // ---------- Dialog: OK ----------
    let ui_weak_ok = ui.as_weak();
    let state_ok = state.clone();
    let update_list_ok = update_ui_list.clone();
    let save_ok = save_entries_fn.clone();
    ui.on_dialog_accepted(move || {
        let Some(ui) = ui_weak_ok.upgrade() else { return; };

        let mode = ui.get_dialog_mode();
        let input: String = ui.get_dialog_input().to_string();

        match mode {
            // ---- Mode 0: Add new entry ----
            0 => {
                let name = input.trim().to_string();
                if name.is_empty() {
                    ui.set_dialog_error("Entry name cannot be empty.".into());
                    return;
                }
                let mut already_exists = false;
                {
                    let mut st = state_ok.borrow_mut();
                    if st.entries.contains_key(&name) {
                        already_exists = true;
                    } else {
                        st.entries.insert(name.clone(), 0);
                        st.current_entry = Some(name.clone());
                        st.current_time = 0;
                        st.session_time = 0;
                        st.last_entry = Some(name.clone());
                    }
                }
                if already_exists {
                    ui.set_dialog_error("Entry already exists.".into());
                    return;
                }

                if let Ok(mut file) = File::create("settings.json") {
                    let json = serde_json::json!({ "lastEntry": name });
                    let _ = file.write_all(json.to_string().as_bytes());
                }

                ui.set_current_entry_name(name.into());
                ui.set_main_timer_str("0:00:00".into());
                ui.set_session_timer_str("Session: 0:00:00".into());
                ui.set_has_selected_entry(true);
                ui.set_dialog_visible(false);

                save_ok();
                update_list_ok();
            }

            // ---- Mode 1: Edit time (int seconds) ----
            1 => {
                let Some(secs) = parse_int_seconds(&input) else {
                    ui.set_dialog_error("Please enter a valid whole number of seconds.".into());
                    return;
                };
                if secs < 0 {
                    ui.set_dialog_error("Time cannot be negative.".into());
                    return;
                }
                let new_time = {
                    let mut st = state_ok.borrow_mut();
                    if let Some(name) = st.current_entry.clone() {
                        st.entries.insert(name, secs);
                        st.current_time = secs;
                        Some(secs)
                    } else {
                        None
                    }
                };
                let Some(nt) = new_time else { return; };
                ui.set_main_timer_str(format_time(nt).into());
                ui.set_dialog_visible(false);
                save_ok();
                update_list_ok();
            }

            // ---- Mode 2: +/- minutes (float, supports . and ,) ----
            2 => {
                let Some(mins) = parse_float(&input) else {
                    ui.set_dialog_error("Please enter a valid number (e.g. -5.5 or -5,5).".into());
                    return;
                };
                let seconds_to_add = (mins * 60.0) as i32;
                let new_time = {
                    let mut st = state_ok.borrow_mut();
                    if let Some(name) = st.current_entry.clone() {
                        st.current_time = (st.current_time + seconds_to_add).max(0);
                        let ct = st.current_time;
                        st.entries.insert(name, ct);
                        Some(ct)
                    } else {
                        None
                    }
                };
                let Some(nt) = new_time else { return; };
                ui.set_main_timer_str(format_time(nt).into());
                ui.set_dialog_visible(false);
                save_ok();
                update_list_ok();
            }

            _ => {}
        }
    });

    // Flush pending changes when the window is closed
    let save_on_close = save_entries_fn.clone();
    ui.window().on_close_requested(move || {
        save_on_close();
        slint::CloseRequestResponse::HideWindow
    });

    ui.run()
}