//! Wiring: adapters in, `assess` in the middle, tray and popover out. No business rules here.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{NaiveDate, Utc};
use nibble_adapters::{ClaudeCode, JsonCatalog};
use nibble_core::{CatalogSource, Combined, Inputs, ProviderSource, SourceError, assess};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_notification::NotificationExt;

use crate::settings::{Settings, StoredDismissal};
use crate::tray;
use crate::view::{self, Prompts, Status, View};

/// How often nibble rereads the logs while idle.
const REFRESH_EVERY: Duration = Duration::from_secs(60);
/// The event the popover listens to for new views.
const VIEW_EVENT: &str = "view";

/// Everything the shell keeps between refreshes.
struct Shell {
    source: ClaudeCode,
    settings: Settings,
    settings_path: PathBuf,
    view: View,
    subscription: Option<String>,
    cycle: Option<NaiveDate>,
}

/// The shell, shared between the refresh thread and command handlers.
struct AppState(Mutex<Shell>);

/// Starts the app.
pub(crate) fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let handle = app.handle().clone();
            app.manage(AppState(Mutex::new(new_shell(&handle)?)));
            tray::build(&handle)?;
            thread::spawn(move || {
                loop {
                    refresh(&handle);
                    thread::sleep(REFRESH_EVERY);
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(false) = event {
                let _hidden = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_view,
            commands::dismiss,
            commands::celebrated,
            commands::answer_autostart,
            commands::set_notifications
        ])
        .run(tauri::generate_context!())
}

fn new_shell<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Shell> {
    let paths = app.path();
    // Claude Code honours CLAUDE_CONFIG_DIR, which moves both its logs and its config file.
    let claude_dir = std::env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from);
    let (projects, config) = if let Some(dir) = claude_dir {
        (dir.join("projects"), dir.join(".claude.json"))
    } else {
        let home = paths.home_dir()?;
        (
            home.join(".claude").join("projects"),
            home.join(".claude.json"),
        )
    };
    let settings_path = paths.app_config_dir()?.join("settings.json");
    Ok(Shell {
        source: ClaudeCode::new(projects, config, paths.app_data_dir()?.join("history.json")),
        settings: Settings::load(&settings_path),
        settings_path,
        view: View::default(),
        subscription: None,
        cycle: None,
    })
}

/// Refreshes on a background thread, so the UI never waits on disk.
pub(crate) fn refresh_soon<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    thread::spawn(move || refresh(&app));
}

/// Rereads everything, reassesses, and pushes the result to the tray and popover.
fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let Ok(mut shell) = state.0.lock() else {
        return;
    };
    let view = match reassess(&mut shell) {
        Ok((view, combined)) => {
            let _drawn = combined.map(|combined| tray::show(app, &combined));
            notify_paid_off(app, &mut shell, &view);
            view
        }
        Err(error) => View {
            error: Some(error.message),
            ..shell.view.clone()
        },
    };
    shell.view = view.clone();
    drop(shell);
    let _sent = app.emit(VIEW_EVENT, view);
}

fn reassess(shell: &mut Shell) -> Result<(View, Option<Combined>), SourceError> {
    let now = Utc::now();
    let snapshot = shell.source.refresh(now)?;
    let catalog = JsonCatalog::bundled().catalog()?;
    let Some(detected) = snapshot.subscription else {
        return Ok((
            View {
                status: Status::SignedOut,
                ..View::default()
            },
            None,
        ));
    };
    let subscription = shell.settings.apply(detected);
    let inputs = Inputs {
        now,
        subscriptions: vec![subscription.clone()],
        usage: snapshot.usage,
        limits: snapshot.limits,
        catalog,
        dismissals: shell.settings.dismissals(),
        history_start: snapshot.history_start,
    };
    let report = assess(&inputs);
    let Some(assessment) = report.subscriptions.first() else {
        return Ok((
            View {
                status: Status::SignedOut,
                ..View::default()
            },
            None,
        ));
    };
    shell.subscription = Some(subscription.id.as_str().to_owned());
    shell.cycle = Some(assessment.cycle.start);
    let mut view = view::of(assessment, subscription.plan.as_str());
    view.prompts = Prompts {
        celebrate: assessment.multiplier.is_some()
            && shell.settings.celebrated_cycle != shell.cycle,
        ask_autostart: !shell.settings.autostart_asked,
    };
    view.notifications_on = !shell.settings.notifications_off;
    Ok((view, Some(report.combined)))
}

/// Sends the Paid off notification once per cycle.
fn notify_paid_off<R: Runtime>(app: &AppHandle<R>, shell: &mut Shell, view: &View) {
    if view.multiplier.is_none() || shell.settings.notified_cycle == shell.cycle {
        return;
    }
    if !shell.settings.notifications_off {
        let plan = view.plan.clone().unwrap_or_else(|| "plan".into());
        let _shown = app
            .notification()
            .builder()
            .title("nibble ate the whole wedge")
            .body(format!("Your {plan} has paid for itself this cycle."))
            .show();
    }
    shell.settings.notified_cycle = shell.cycle;
    save(shell);
}

fn save(shell: &Shell) {
    // Losing a setting is better than crashing a menu bar app; the next save retries.
    let _saved = shell.settings.save(&shell.settings_path);
}

/// Runs `change` on the shell, saves the settings and refreshes.
fn update<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<'_, AppState>,
    change: impl FnOnce(&mut Shell),
) {
    if let Ok(mut shell) = state.0.lock() {
        change(&mut shell);
        save(&shell);
    }
    refresh_soon(app);
}

/// What the popover can ask of the shell.
mod commands {
    #![expect(
        clippy::needless_pass_by_value,
        reason = "Tauri hands command arguments over by value"
    )]

    use chrono::Utc;
    use tauri::{AppHandle, Runtime, State};
    use tauri_plugin_autostart::ManagerExt;

    use super::{AppState, StoredDismissal, View, update};

    #[tauri::command]
    pub(super) fn get_view(state: State<'_, AppState>) -> View {
        state
            .0
            .lock()
            .map(|shell| shell.view.clone())
            .unwrap_or_default()
    }

    #[tauri::command]
    pub(super) fn dismiss<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) {
        update(&app, &state, |shell| {
            if let Some(subscription) = shell.subscription.clone() {
                shell.settings.dismissals.push(StoredDismissal {
                    subscription,
                    at: Utc::now(),
                });
            }
        });
    }

    #[tauri::command]
    pub(super) fn celebrated<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) {
        update(&app, &state, |shell| {
            shell.settings.celebrated_cycle = shell.cycle;
        });
    }

    #[tauri::command]
    pub(super) fn answer_autostart<R: Runtime>(
        app: AppHandle<R>,
        state: State<'_, AppState>,
        enable: bool,
    ) {
        let launcher = app.autolaunch();
        let _changed = if enable {
            launcher.enable()
        } else {
            launcher.disable()
        };
        update(&app, &state, |shell| {
            shell.settings.autostart_asked = true;
        });
    }

    #[tauri::command]
    pub(super) fn set_notifications<R: Runtime>(
        app: AppHandle<R>,
        state: State<'_, AppState>,
        on: bool,
    ) {
        update(&app, &state, |shell| {
            shell.settings.notifications_off = !on;
        });
    }
}
