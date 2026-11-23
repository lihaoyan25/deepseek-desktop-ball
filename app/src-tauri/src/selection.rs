use std::process;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};
use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND, POINT};
use windows::Win32::System::Com::{
  CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::DataExchange::{
  CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
  SetClipboardData,
};
use windows::Win32::System::Memory::{
  GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Accessibility::{
  CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
  keybd_event, GetAsyncKeyState, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_CONTROL, VK_LBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
  GetAncestor, GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId,
  SetForegroundWindow, ShowWindow, WindowFromPoint, GA_ROOT, SW_SHOWNOACTIVATE,
};

const TOOLBAR_WIDTH: f64 = 360.0;
const TOOLBAR_HEIGHT: f64 = 48.0;
const COMPACT_WIDTH: f64 = 320.0;
const COMPACT_HEIGHT: f64 = 400.0;
const DRAG_THRESHOLD: i32 = 6;
const EXCLUDED_CLASSES: &[&str] = &[
  "ConsoleWindowClass",
  "WindowsTerminalWindow",
  "CASCADIA_HOSTING_WINDOW_CLASS",
  "mintty",
];

#[derive(Default)]
pub struct SelectionState {
  enabled: AtomicBool,
  visible: AtomicBool,
  target: Mutex<Option<isize>>,
  anchor: Mutex<Option<(i32, i32)>>,
  text: Mutex<Option<String>>,
}

impl SelectionState {
  fn set_enabled(&self, enabled: bool) {
    self.enabled.store(enabled, Ordering::Relaxed);
  }

  fn enabled(&self) -> bool {
    self.enabled.load(Ordering::Relaxed)
  }

  fn visible(&self) -> bool {
    self.visible.load(Ordering::Relaxed)
  }
}

#[derive(Clone, Serialize)]
struct SelectionPayload {
  action: String,
  text: String,
}

fn hwnd(value: isize) -> HWND {
  HWND(value as *mut core::ffi::c_void)
}

fn hwnd_value(value: HWND) -> isize {
  value.0 as isize
}

fn own_window(target: HWND) -> bool {
  if target.0.is_null() {
    return true;
  }
  let mut pid = 0u32;
  unsafe {
    GetWindowThreadProcessId(target, Some(&mut pid));
  }
  pid == process::id()
}

fn ignored_window(target: HWND) -> bool {
  if own_window(target) {
    return true;
  }
  let mut buffer = [0u16; 256];
  let length = unsafe { GetClassNameW(target, &mut buffer) };
  if length <= 0 {
    return false;
  }
  let class = String::from_utf16_lossy(&buffer[..length as usize]);
  EXCLUDED_CLASSES.contains(&class.as_str())
}

fn toolbar_window(app: &AppHandle) -> Option<HWND> {
  app
    .get_webview_window("selection")
    .and_then(|window| window.hwnd().ok())
}

fn point_in_toolbar(app: &AppHandle, point: POINT) -> bool {
  let Some(toolbar) = toolbar_window(app) else {
    return false;
  };
  unsafe {
    let under = WindowFromPoint(point);
    if under.0.is_null() {
      return false;
    }
    GetAncestor(under, GA_ROOT) == toolbar
  }
}

fn hide_toolbar(app: &AppHandle) {
  if let Some(window) = app.get_webview_window("selection") {
    let _ = window.hide();
  }
  let state = app.state::<SelectionState>();
  state.visible.store(false, Ordering::Relaxed);
  if let Ok(mut text) = state.text.lock() {
    *text = None;
  }
}

fn show_toolbar(app: &AppHandle, point: POINT, target: HWND) {
  let Some(window) = app.get_webview_window("selection") else {
    return;
  };
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let scale = monitor.scale_factor();
  let work = monitor.work_area();
  let width = (TOOLBAR_WIDTH * scale).round() as i32;
  let height = (TOOLBAR_HEIGHT * scale).round() as i32;
  let margin = (10.0 * scale).round() as i32;
  let min_x = work.position.x;
  let max_x = (work.position.x + work.size.width as i32 - width).max(min_x);
  let min_y = work.position.y;
  let max_y = (work.position.y + work.size.height as i32 - height).max(min_y);
  let x = (point.x + margin).clamp(min_x, max_x);
  let y = (point.y + margin).clamp(min_y, max_y);

  let state = app.state::<SelectionState>();
  *state.target.lock().unwrap() = Some(hwnd_value(target));
  *state.anchor.lock().unwrap() = Some((point.x, point.y));
  state.visible.store(true, Ordering::Relaxed);

  let _ = window.set_size(tauri::LogicalSize::new(TOOLBAR_WIDTH, TOOLBAR_HEIGHT));
  let _ = window.set_position(PhysicalPosition::new(x, y));
  let _ = window.show();
  unsafe {
    if let Ok(hwnd) = window.hwnd() {
      let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
  }
}

pub(crate) fn open_compact_panel(app: &AppHandle, anchor: (i32, i32)) -> Result<(), String> {
  let panel = app
    .get_webview_window("panel")
    .ok_or_else(|| "panel window unavailable".to_string())?;
  let monitor = app
    .primary_monitor()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "monitor unavailable".to_string())?;
  let scale = monitor.scale_factor();
  let work = monitor.work_area();
  let width = (COMPACT_WIDTH * scale).round() as i32;
  let height = (COMPACT_HEIGHT * scale).round() as i32;
  let gap = (18.0 * scale).round() as i32;
  let right = work.position.x + work.size.width as i32;
  let bottom = work.position.y + work.size.height as i32;
  let x = if anchor.0 + width + gap <= right {
    anchor.0 + gap
  } else {
    anchor.0 - width - gap
  }
  .clamp(work.position.x, (right - width).max(work.position.x));
  let y = if anchor.1 + height + gap <= bottom {
    anchor.1 + gap
  } else {
    anchor.1 - height - gap
  }
  .clamp(work.position.y, (bottom - height).max(work.position.y));

  let state = app.state::<crate::PanelState>();
  crate::begin_temporary_compact(app);
  if let Ok(mut mode) = state.mode.lock() {
    *mode = crate::PanelMode::Mini;
  }
  let _ = app.emit("panel-mode", "mini");
  let _ = panel.set_size(tauri::LogicalSize::new(COMPACT_WIDTH, COMPACT_HEIGHT));
  let _ = panel.set_position(PhysicalPosition::new(x, y));
  let _ = panel.set_skip_taskbar(true);
  let _ = panel.set_always_on_top(true);
  let _ = panel.show();
  let _ = panel.set_focus();
  if let Some(ball) = app.get_webview_window("ball") {
    let _ = ball.hide();
  }
  Ok(())
}

fn read_clipboard_unlocked() -> Option<String> {
  unsafe {
    let handle = GetClipboardData(CF_UNICODETEXT.0.into()).ok()?;
    let global = HGLOBAL(handle.0);
    let size = GlobalSize(global);
    if size < 2 {
      return None;
    }
    let pointer = GlobalLock(global) as *const u16;
    if pointer.is_null() {
      return None;
    }
    let length = size / 2;
    let slice = std::slice::from_raw_parts(pointer, length);
    let end = slice.iter().position(|value| *value == 0).unwrap_or(length);
    let text = String::from_utf16_lossy(&slice[..end]);
    let _ = GlobalUnlock(global);
    Some(text)
  }
}

fn read_clipboard_text() -> Option<String> {
  unsafe {
    if OpenClipboard(None).is_err() {
      return None;
    }
    let text = read_clipboard_unlocked();
    let _ = CloseClipboard();
    text
  }
}

fn set_clipboard_text(text: &str) -> Result<(), String> {
  unsafe {
    if OpenClipboard(None).is_err() {
      return Err("clipboard unavailable".to_string());
    }
    if EmptyClipboard().is_err() {
      let _ = CloseClipboard();
      return Err("clipboard unavailable".to_string());
    }
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = wide.len() * std::mem::size_of::<u16>();
    let global = GlobalAlloc(GMEM_MOVEABLE, bytes).map_err(|error| error.to_string());
    let global = match global {
      Ok(global) => global,
      Err(error) => {
        let _ = CloseClipboard();
        return Err(error);
      }
    };
    let pointer = GlobalLock(global);
    if pointer.is_null() {
      let _ = CloseClipboard();
      return Err("clipboard unavailable".to_string());
    }
    std::ptr::copy_nonoverlapping(wide.as_ptr(), pointer as *mut u16, wide.len());
    let _ = GlobalUnlock(global);
    let _ = SetClipboardData(CF_UNICODETEXT.0.into(), Some(HANDLE(global.0)));
    let _ = CloseClipboard();
  }
  Ok(())
}

fn read_uia_selection() -> Option<Option<String>> {
  unsafe {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let result = (|| {
      let automation: IUIAutomation =
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
      let element = automation.GetFocusedElement().ok()?;
      let pattern = element
        .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
        .ok()?;
      let ranges = pattern.GetSelection().ok()?;
      let count = ranges.Length().ok()?;
      let mut text = String::new();
      for index in 0..count {
        let range = ranges.GetElement(index).ok()?;
        let value = range.GetText(-1).ok()?;
        text.push_str(&value.to_string());
      }
      Some(text)
    })();
    CoUninitialize();
    result.map(Some)
  }
}

fn selected_text_from_uia() -> Option<Option<String>> {
  let (sender, receiver) = mpsc::channel();
  thread::spawn(move || {
    let _ = sender.send(read_uia_selection());
  });
  receiver.recv_timeout(Duration::from_millis(320)).ok().flatten()
}

fn send_copy() {
  unsafe {
    keybd_event(VK_CONTROL.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
    keybd_event(b'C', 0, KEYBD_EVENT_FLAGS(0), 0);
    keybd_event(b'C', 0, KEYEVENTF_KEYUP, 0);
    keybd_event(VK_CONTROL.0 as u8, 0, KEYEVENTF_KEYUP, 0);
  }
}

fn resolve_selected_text(target: HWND) -> Result<String, String> {
  if target.0.is_null() || own_window(target) {
    return Err("no source window".to_string());
  }
  let old = read_clipboard_text().ok_or_else(|| "clipboard has no text".to_string())?;
  let before = unsafe { GetClipboardSequenceNumber() };
  unsafe {
    let _ = SetForegroundWindow(target);
  }
  thread::sleep(Duration::from_millis(60));
  send_copy();

  let mut copied = false;
  for _ in 0..24 {
    thread::sleep(Duration::from_millis(25));
    if unsafe { GetClipboardSequenceNumber() } != before {
      copied = true;
      break;
    }
  }
  let after = unsafe { GetClipboardSequenceNumber() };
  let text = if copied {
    let text = read_clipboard_text();
    if unsafe { GetClipboardSequenceNumber() } == after {
      let _ = set_clipboard_text(&old);
    }
    text
  } else {
    log::warn!("selection copy did not change clipboard");
    None
  };
  text
    .filter(|text| !text.trim().is_empty())
    .ok_or_else(|| "no selected text".to_string())
}

#[tauri::command]
pub fn set_selection_assistant_enabled(state: State<'_, SelectionState>, enabled: bool) {
  state.set_enabled(enabled);
}

#[tauri::command]
pub fn selection_action(
  app: AppHandle,
  state: State<'_, SelectionState>,
  action: String,
) -> Result<(), String> {
  let target = state
    .target
    .lock()
    .ok()
    .and_then(|target| *target)
    .map(hwnd)
    .ok_or_else(|| "no source window".to_string())?;
  let uia_text = state
    .text
    .lock()
    .ok()
    .and_then(|text| text.clone())
    .filter(|text| !text.trim().is_empty());
  let anchor = state
    .anchor
    .lock()
    .ok()
    .and_then(|anchor| *anchor)
    .unwrap_or((0, 0));
  hide_toolbar(&app);
  let text = match uia_text {
    Some(text) => text,
    None => resolve_selected_text(target)?,
  };

  if action == "copy" {
    set_clipboard_text(&text)?;
    return Ok(());
  }

  if let Err(error) = open_compact_panel(&app, anchor) {
    log::warn!("selection_action open compact failed: {error}");
    return Err(error);
  }
  log::info!("selection_action opening panel for {action}");
  let _ = app.emit_to(
    "panel",
    "selection-action",
    SelectionPayload {
      action,
      text,
    },
  );
  Ok(())
}

pub fn start(app: AppHandle) {
  thread::spawn(move || {
    let mut down = false;
    let mut moved = false;
    let mut start = POINT::default();
    let mut target = HWND::default();

    loop {
      thread::sleep(Duration::from_millis(22));
      let state = app.state::<SelectionState>();
      if !state.enabled() {
        if state.visible() {
          hide_toolbar(&app);
        }
        down = false;
        continue;
      }

      if state.visible() {
        let foreground = unsafe { GetForegroundWindow() };
        let source = state.target.lock().ok().and_then(|target| *target).map(hwnd);
        let toolbar = toolbar_window(&app);
        if source.is_some() && foreground != source.unwrap() && Some(foreground) != toolbar {
          hide_toolbar(&app);
        }
      }

      let pressed = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
      let mut point = POINT::default();
      if unsafe { GetCursorPos(&mut point) }.is_err() {
        continue;
      }

      if !down && pressed {
        down = true;
        moved = false;
        start = point;
        target = unsafe { GetForegroundWindow() };
        if state.visible() && !point_in_toolbar(&app, point) {
          hide_toolbar(&app);
        }
        continue;
      }

      if down && pressed {
        if (point.x - start.x).abs() > DRAG_THRESHOLD
          || (point.y - start.y).abs() > DRAG_THRESHOLD
        {
          moved = true;
        }
        continue;
      }

      if down && !pressed {
        down = false;
        let in_toolbar = state.visible() && point_in_toolbar(&app, point);

        if !in_toolbar && !ignored_window(target) && moved {
          let uia = selected_text_from_uia();
          let fallback = uia.is_none();
          let text = match uia {
            Some(Some(text)) if !text.trim().is_empty() => Some(text),
            _ => None,
          };
          if text.is_some() || fallback {
            if let Ok(mut stored) = state.text.lock() {
              *stored = text;
            }
            show_toolbar(&app, point, target);
          } else if let Ok(mut stored) = state.text.lock() {
            *stored = None;
          }
        }
      }
    }
  });
}
