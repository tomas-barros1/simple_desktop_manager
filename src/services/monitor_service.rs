use gtk4::gio::{Cancellable, File, FileMonitor, FileMonitorEvent, FileMonitorFlags};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use tracing::debug;

/// Set up GIO file monitors for the given directories with a debounce delay.
/// When file changes occur in any of the watched directories, `on_change` is invoked.
pub fn watch_directories<F>(paths: &[PathBuf], debounce_ms: u64, on_change: F) -> Vec<FileMonitor>
where
    F: Fn() + 'static,
{
    let on_change_rc = Rc::new(on_change);
    let debounce_flag = Rc::new(RefCell::new(false));
    let mut monitors = Vec::new();

    for dir in paths {
        if !dir.is_dir() {
            continue;
        }

        let gfile = File::for_path(dir);
        let Ok(monitor) = gfile.monitor_directory(FileMonitorFlags::NONE, Cancellable::NONE) else {
            continue;
        };

        let on_change_cb = on_change_rc.clone();
        let debounce_flag_cb = debounce_flag.clone();
        let debounce_duration = Duration::from_millis(debounce_ms);

        monitor.connect_changed(move |_mon, file, _other, event| {
            match event {
                FileMonitorEvent::Created
                | FileMonitorEvent::Deleted
                | FileMonitorEvent::ChangesDoneHint => {
                    if *debounce_flag_cb.borrow() {
                        return;
                    }
                    *debounce_flag_cb.borrow_mut() = true;

                    debug!(file = ?file.path(), event = ?event, "filesystem change detected");

                    let on_change_fire = on_change_cb.clone();
                    let debounce_reset = debounce_flag_cb.clone();

                    gtk4::glib::timeout_add_local_once(debounce_duration, move || {
                        *debounce_reset.borrow_mut() = false;
                        on_change_fire();
                    });
                }
                _ => {}
            }
        });

        monitors.push(monitor);
    }

    monitors
}
