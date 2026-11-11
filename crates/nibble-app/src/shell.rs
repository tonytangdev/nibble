//! Wiring: adapters in, `assess` in the middle, tray and popover out. No business rules here.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Days, Local, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc};
use nibble_adapters::{ClaudeCode, JsonCatalog};
use nibble_core::{CatalogSource, Combined, Day, Inputs, ProviderSource, SourceError, assess};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::settings::{Settings, StoredDismissal};
use crate::tray;
use crate::updates;
use crate::view::{self, CheckView, Prompts, Status, Switches, UpdateView, View};

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
    /// The start of the current Day.
    day: Option<DateTime<Utc>>,
    /// When nibble last asked GitHub for a new version, this run.
    last_update_check: Option<DateTime<Utc>>,
    /// The newer version found this run, if any.
    pending: Option<Pending>,
    /// How the last check the user asked for went, this run.
    asked_check: Option<CheckView>,
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
            // A click on any of nibble's notifications opens the popover.
            let opener = handle.clone();
            app.notification().on_action(move |action| {
                if action.action_id() == "tap" {
                    tray::open_popover(&opener);
                }
            })?;
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
                        check_for_update(&handle, false);
                        thread::sleep(UPDATE_TICK);
                    }
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(false) = event {
                tray::hide_on_blur(window);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_view,
            commands::dismiss,
            commands::celebrated,
            commands::answer_autostart,
            commands::set_notifications,
            commands::set_update_checks,
            commands::check_for_update,
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
        day: None,
        last_update_check: None,
        pending: None,
        asked_check: None,
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
    // With checks off, a found version stops nagging unless the user asked for it, but an
    // install under way still shows.
    let asked = shell.asked_check == Some(CheckView::Found);
    view.update = shell.pending.as_ref().map(Pending::view).filter(|update| {
        updates::offer(view.switches.update_checks, asked)
            || !matches!(update, UpdateView::Available { .. })
    });
    view.update_check = shell.asked_check;
    view.version = app.package_info().version.to_string();
    tray::badge(app, view.update.is_some());
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
        today: local_day(now),
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
    shell.day = Some(assessment.day.start);
    let mut view = view::of(assessment, &subscription, &inputs.catalog, now);
    view.prompts = Prompts {
        celebrate: assessment.multiplier.is_some() && shell.settings.celebrated_day != shell.day,
        ask_autostart: !shell.settings.autostart_asked,
    };
    Ok((view, Some(report.combined)))
}

/// Today on this computer's clock, from local midnight to the next.
fn local_day(now: DateTime<Utc>) -> Day {
    let today = now.with_timezone(&Local).date_naive();
    let start = local_midnight(today).unwrap_or(now);
    let ends_at = today
        .checked_add_days(Days::new(1))
        .and_then(local_midnight)
        .unwrap_or_else(|| start + TimeDelta::days(1));
    Day { start, ends_at }
}

/// The first moment of `date` in local time. Where a clock change skips midnight, the
/// day starts an hour later.
fn local_midnight(date: NaiveDate) -> Option<DateTime<Utc>> {
    let at = |time: NaiveTime| Local.from_local_datetime(&date.and_time(time)).earliest();
    at(NaiveTime::MIN)
        .or_else(|| at(NaiveTime::from_hms_opt(1, 0, 0)?))
        .map(|start| start.with_timezone(&Utc))
}

/// Sends the Paid off notification once a Day.
fn notify_paid_off<R: Runtime>(app: &AppHandle<R>, shell: &mut Shell, view: &View) {
    if view.multiplier.is_none() || shell.settings.notified_day == shell.day {
        return;
    }
    if !shell.settings.notifications_off {
        let plan = view.plan.clone().unwrap_or_else(|| "plan".into());
        let _shown = app
            .notification()
            .builder()
            .title("nibble ate the whole wedge")
            .body(format!("Today alone paid for your whole month of {plan}."))
            .show();
    }
    shell.settings.notified_day = shell.day;
    save(shell);
}

/// Asks GitHub for a newer version: if a check is due, or right away when the user asked
/// (`asked`). A failed check the user did not ask for stays quiet and is retried at the
/// next hourly tick.
fn check_for_update<R: Runtime>(app: &AppHandle<R>, asked: bool) {
    let state = app.state::<AppState>();
    let now = Utc::now();
    // Claim this check before the network call, so a second caller meanwhile skips it.
    let previous = {
        let Ok(mut shell) = state.0.lock() else {
            return;
        };
        let checks_on = !shell.settings.update_checks_off;
        let wanted = if asked {
            shell.asked_check != Some(CheckView::Checking)
        } else {
            updates::due(shell.last_update_check, now, checks_on)
        };
        if shell.pending.as_ref().is_some_and(Pending::installing) || !wanted {
            return;
        }
        if asked {
            shell.asked_check = Some(CheckView::Checking);
        }
        shell.last_update_check.replace(now)
    };
    if asked {
        refresh(app);
    }
    let found = app
        .updater()
        .map(|updater| tauri::async_runtime::block_on(updater.check()));
    let Ok(mut shell) = state.0.lock() else {
        return;
    };
    let Ok(Ok(found)) = found else {
        shell.last_update_check = previous;
        if asked {
            shell.asked_check = Some(CheckView::Failed);
            drop(shell);
            refresh_soon(app);
        }
        return;
    };
    if shell.pending.as_ref().is_some_and(Pending::installing) {
        return;
    }
    if asked {
        shell.asked_check = Some(if found.is_some() {
            CheckView::Found
        } else {
            CheckView::UpToDate
        });
    }
    if let Some(update) = &found {
        // The user who asked is looking at the answer already; a notification would echo it.
        notify_update(app, &mut shell, &update.version, !asked);
    }
    shell.pending = found.map(Pending::found);
    drop(shell);
    refresh_soon(app);
}

/// Sends the update notification once per version. With `show` false, only marks the
/// version as announced, so a later check stays quiet about it.
fn notify_update<R: Runtime>(app: &AppHandle<R>, shell: &mut Shell, version: &str, show: bool) {
    if !updates::announce(version, shell.settings.announced_version.as_deref()) {
        return;
    }
    if show {
        notify_new_version(app, version);
    }
    shell.settings.announced_version = Some(version.to_owned());
    save(shell);
}

fn notify_new_version<R: Runtime>(app: &AppHandle<R>, version: &str) {
    let _shown = app
        .notification()
        .builder()
        .title("nibble grew a new tooth")
        .body(format!(
            "Version {version} is ready to install. Click to open nibble."
        ))
        .show();
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
            shell.settings.celebrated_day = shell.day;
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
            std::thread::spawn(move || super::check_for_update(&app, false));
        }
    }

    /// Looks for a new version now, even with the daily checks off.
    #[tauri::command]
    pub(super) fn check_for_update<R: Runtime>(app: AppHandle<R>) {
        std::thread::spawn(move || super::check_for_update(&app, true));
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

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, Utc};

    use super::local_day;

    #[test]
    fn today_holds_now_and_spans_one_local_day() {
        let now = Utc::now();
        let day = local_day(now);
        assert!(day.start <= now && now < day.ends_at);
        let length = day.ends_at - day.start;
        assert!(TimeDelta::hours(23) <= length && length <= TimeDelta::hours(25));
    }
}
