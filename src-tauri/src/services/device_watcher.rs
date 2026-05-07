use crate::commands::device::detect_device_inner;
use nusb::hotplug::HotplugEvent;
use nusb::{DeviceId, MaybeFuture};
use std::collections::HashSet;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const APPLE_VID: u16 = 0x05ac;
const DEBOUNCE: Duration = Duration::from_millis(400);
const SETTLE: Duration = Duration::from_millis(250);

pub async fn run(app: AppHandle) {
    use futures_lite::StreamExt;

    let mut watch = match nusb::watch_devices() {
        Ok(w) => w,
        Err(e) => {
            log::error!(
                "USB hotplug watcher failed to start ({e}); falling back to interval polling only"
            );
            return;
        }
    };

    let mut apple_ids: HashSet<DeviceId> = HashSet::new();
    if let Ok(devices) = nusb::list_devices().wait() {
        for d in devices {
            if d.vendor_id() == APPLE_VID {
                apple_ids.insert(d.id());
            }
        }
    }

    emit_detection(&app, !apple_ids.is_empty());

    loop {
        let Some(first) = watch.next().await else {
            break;
        };
        let mut interesting = handle_event(&first, &mut apple_ids);

        loop {
            match tokio::time::timeout(DEBOUNCE, watch.next()).await {
                Ok(Some(ev)) => interesting |= handle_event(&ev, &mut apple_ids),
                Ok(None) => return,
                Err(_) => break,
            }
        }

        if interesting {
            tokio::time::sleep(SETTLE).await;
            emit_detection(&app, !apple_ids.is_empty());
        }
    }
}

fn handle_event(event: &HotplugEvent, apple_ids: &mut HashSet<DeviceId>) -> bool {
    match event {
        HotplugEvent::Connected(d) => {
            if d.vendor_id() == APPLE_VID {
                apple_ids.insert(d.id());
                true
            } else {
                false
            }
        }
        HotplugEvent::Disconnected(id) => apple_ids.remove(id),
    }
}

fn emit_detection(app: &AppHandle, expect_connected: bool) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || match detect_device_inner(&app) {
        Ok(info) => {
            // If the USB layer still sees an Apple device but ideviceinfo/irecovery
            // haven't caught up (mid-reset, usbmuxd not ready), suppress the
            // transient "disconnected" — the next hotplug burst or the 15 s
            // safety poll will resolve it. Prevents flicker on connect/pwn.
            if !info.connected && expect_connected {
                return;
            }
            let _ = app.emit("device-state-changed", info);
        }
        Err(e) => {
            log::warn!("device-state-changed: detection error: {e}");
        }
    });
}
