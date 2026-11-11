//! Wiring: adapters in, `assess` in the middle, tray and popover out. No business rules here.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use nibble_adapters::{ClaudeCode, JsonCatalog};
use nibble_core::{CatalogSource, Combined, Inputs, ProviderSource, SourceError, assess};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::settings::{Settings, StoredDismissal};
use crate::tray;
use crate::updates;
use crate::view::{self, Prompts, Status, Switches, UpdateView, View};

/// How often nibble rereads the logs while idle.
const REFRESH_EVERY: Duration = Duration::from_secs(60);
/// How often nibble asks whether an update check is due.
const UPDATE_TICK: Duration = Duration::from_hours(1);
/// The event the popover listens to for new views.
const VIEW_EVENT: &str = "view";

/// Everything the shell keeps between refreshes.
struct Shell {
    source: ClaudeCode,
    settings: Settings,
    settings_path: PathBuf,
    view: View,
    subscription: Option<String>,
    /// The start of the current Week.
    week: Option<DateTime<Utc>>,
    /// When nibble last asked GitHub for a new version, this run.
    last_update_check: Option<DateTime<Utc>>,
    /// The newer version found this run, if any.
    pending: Option<Pending>,
}

/// A newer version, kept to install it, and how far installing it got.
struct Pending {
    update: Update,
    stage: Stage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Found,
    Installing,
    Failed,
}

impl Pending {
    fn found(update: Update) -> Self {
        Self {
            update,
            stage: Stage::Found,
        }
    }

    fn installing(&self) -> bool {
        self.stage == Stage::Installing
    }

    fn view(&self) -> UpdateView {
        match self.stage {
            Stage::Found => UpdateView::Available {
                version: self.update.version.clone(),
            },
            Stage::Installing => UpdateView::Installing,
            Stage::Failed => UpdateView::Failed,
        }
    }
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let handle = app.handle().clone();
            app.manage(AppState(Mutex::new(new_shell(&handle)?)));
            tray::build(&handle)?;
            let refresher = handle.clone();
            thread::spawn(move || {
                loop {
                    refresh(&refresher);
                    thread::sleep(REFRESH_EVERY);
                }
            });
            // Development builds have no release to update to.
            if !cfg!(debug_assertions) {
                thread::spawn(move || {
                    loop {
                        check_for_update(&handle);
                        thread::sleep(UPDATE_TICK);
                    }
                });
            }
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
            commands::set_notifications,
            commands::set_update_checks,
            commands::install_update,
            commands::quit
        ])
        .run(tauri::generate_context!())
}

fn new_shell<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Shell> {
    let paths = app.path();
    // NIBBLE_DATA_DIR keeps settings and history elsewhere, e.g. for demo data.
    let data_dir = std::env::var_os("NIBBLE_DATA_DIR").map(PathBuf::from);
    let settings_path = match &data_dir {
        Some(dir) => dir.clone(),
        None => paths.app_config_dir()?,
    }
    .join("settings.json");
    let history_path = match data_dir {
        Some(dir) => dir,
        None => paths.app_data_dir()?,
    }
    .join("history.json");
    Ok(Shell {
        source: ClaudeCode::installed(&paths.home_dir()?, history_path),
        settings: Settings::load(&settings_path),
        settings_path,
        view: View::default(),
        subscription: None,
        week: None,
        last_update_check: None,
        pending: None,
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
    let mut view = match reassess(&mut shell) {
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
    view.switches = Switches {
        notifications: !shell.settings.notifications_off,
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
        update_checks: !shell.settings.update_checks_off,
    };
    // With checks off, a found version stops nagging, but an install under way still shows.
    view.update = shell.pending.as_ref().map(Pending::view).filter(|update| {
        view.switches.update_checks || !matches!(update, UpdateView::Available { .. })
    });
    shell.view = view.clone();
    drop(shell);
    let _sent = app.emit(VIEW_EVENT, view);
}

fn reassess(shell: &mut Shell) -> Result<(View, Option<Combined>), SourceError> {
    let now = Utc::now();
    let snapshot = shell.source.refresh(now)?;
    let catalog = JsonCatalog::bundled().catalog()?;
    let Some(detected) = snapshot.subscription else {
        return Ok((View::waiting(Status::SignedOut), None));
    };
    if snapshot.usage.is_empty() {
        return Ok((View::waiting(Status::NoUsage), None));
    }
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
        return Ok((View::waiting(Status::SignedOut), None));
    };
    shell.subscription = Some(subscription.id.as_str().to_owned());
    shell.week = Some(assessment.week.start);
    let mut view = view::of(assessment, &subscription, &inputs.catalog, now);
    view.prompts = Prompts {
        celebrate: assessment.multiplier.is_some() && shell.settings.celebrated_week != shell.week,
        ask_autostart: !shell.settings.autostart_asked,
    };
    Ok((view, Some(report.combined)))
}

/// Sends the Paid off notification once a Week.
fn notify_paid_off<R: Runtime>(app: &AppHandle<R>, shell: &mut Shell, view: &View) {
    if view.multiplier.is_none() || shell.settings.notified_week == shell.week {
        return;
    }
    if !shell.settings.notifications_off {
        let plan = view.plan.clone().unwrap_or_else(|| "plan".into());
        let _shown = app
            .notification()
            .builder()
            .title("nibble ate the whole wedge")
            .body(format!("Your {plan} has paid for itself this week."))
            .show();
    }
    shell.settings.notified_week = shell.week;
    save(shell);
}

/// Asks GitHub for a newer version, if a check is due. A failed check stays quiet and is
/// retried at the next hourly tick.
fn check_for_update<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let now = Utc::now();
    // Claim this check before the network call, so a second caller meanwhile skips it.
    let previous = {
        let Ok(mut shell) = state.0.lock() else {
            return;
        };
        let checks_on = !shell.settings.update_checks_off;
        if shell.pending.as_ref().is_some_and(Pending::installing)
            || !updates::due(shell.last_update_check, now, checks_on)
        {
            return;
        }
        shell.last_update_check.replace(now)
    };
    let found = app
        .updater()
        .map(|updater| tauri::async_runtime::block_on(updater.check()));
    let Ok(mut shell) = state.0.lock() else {
        return;
    };
    let Ok(Ok(found)) = found else {
        shell.last_update_check = previous;
        return;
    };
    if shell.pending.as_ref().is_some_and(Pending::installing) {
        return;
    }
    shell.pending = found.map(Pending::found);
    drop(shell);
    refresh_soon(app);
}

/// Downloads and installs the update found, then restarts into it.
fn install_update<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let update = {
        let Ok(mut shell) = state.0.lock() else {
            return;
        };
        let Some(pending) = shell.pending.as_mut().filter(|p| !p.installing()) else {
            return;
        };
        pending.stage = Stage::Installing;
        pending.update.clone()
    };
    refresh(app);
    let installed =
        tauri::async_runtime::block_on(update.download_and_install(|_chunk, _total| {}, || {}));
    if installed.is_ok() {
        app.restart();
    }
    if let Ok(mut shell) = state.0.lock()
        && let Some(pending) = shell.pending.as_mut()
    {
        pending.stage = Stage::Failed;
    }
    refresh(app);
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
            shell.settings.celebrated_week = shell.week;
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

    #[tauri::command]
    pub(super) fn set_update_checks<R: Runtime>(
        app: AppHandle<R>,
        state: State<'_, AppState>,
        on: bool,
    ) {
        update(&app, &state, |shell| {
            shell.settings.update_checks_off = !on;
        });
        // Turning checks back on looks right away, not at the next hourly tick.
        if on && !cfg!(debug_assertions) {
            std::thread::spawn(move || super::check_for_update(&app));
        }
    }

    #[tauri::command]
    pub(super) fn install_update<R: Runtime>(app: AppHandle<R>) {
        std::thread::spawn(move || super::install_update(&app));
    }

    #[tauri::command]
    pub(super) fn quit<R: Runtime>(app: AppHandle<R>) {
        app.exit(0);
    }
}
