use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[cfg(target_os = "windows")]
fn make_frameless(window: &tauri::WebviewWindow) {
  use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_CAPTION, WS_EX_APPWINDOW,
    WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
  };

  if let Ok(hwnd) = window.hwnd() {
    unsafe {
      let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
      let strip = (WS_CAPTION.0
        | WS_SYSMENU.0
        | WS_THICKFRAME.0
        | WS_MINIMIZEBOX.0
        | WS_MAXIMIZEBOX.0) as isize;
      SetWindowLongPtrW(hwnd, GWL_STYLE, (style & !strip) | WS_POPUP.0 as isize);

      let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
      let new_ex = (ex & !(WS_EX_APPWINDOW.0 as isize)) | WS_EX_TOOLWINDOW.0 as isize;
      SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex);

      let _ = SetWindowPos(
        hwnd,
        None,
        0,
        0,
        0,
        0,
        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
      );
    }
  }
}

#[cfg(target_os = "windows")]
fn remove_border(window: &tauri::WebviewWindow) {
  use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE,
  };

  if let Ok(hwnd) = window.hwnd() {
    let color: u32 = DWMWA_COLOR_NONE;
    unsafe {
      let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_BORDER_COLOR,
        &color as *const u32 as *const core::ffi::c_void,
        4,
      );
    }
  }
}

#[cfg(target_os = "windows")]
fn set_taskbar_visible(window: &tauri::WebviewWindow, visible: bool) {
  use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
  };

  if let Ok(hwnd) = window.hwnd() {
    unsafe {
      let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
      let add = if visible {
        WS_EX_APPWINDOW.0
      } else {
        WS_EX_TOOLWINDOW.0
      };
      let remove = if visible {
        WS_EX_TOOLWINDOW.0
      } else {
        WS_EX_APPWINDOW.0
      };
      let new_ex = (ex & !(remove as isize)) | add as isize;
      SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex);
      let _ = SetWindowPos(
        hwnd,
        None,
        0,
        0,
        0,
        0,
        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
      );
    }
  }
  let _ = window.set_skip_taskbar(!visible);
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct BallPoint {
  x: i32,
  y: i32,
}

#[derive(Default)]
struct BallRest(Mutex<Option<BallPoint>>);

#[derive(Clone, Copy, PartialEq)]
enum PanelMode {
  Mini,
  Max,
}

struct PanelState(Mutex<PanelMode>);

impl Default for PanelState {
  fn default() -> Self {
    Self(Mutex::new(PanelMode::Mini))
  }
}

fn clamp_ball_point(monitor: &tauri::Monitor, point: BallPoint, bw: i32, bh: i32) -> BallPoint {
  let pos = monitor.position();
  let size = monitor.size();
  let min_x = pos.x;
  let max_x = (pos.x + size.width as i32 - bw).max(min_x);
  let min_y = pos.y;
  let max_y = (pos.y + size.height as i32 - bh).max(min_y);
  BallPoint {
    x: point.x.clamp(min_x, max_x),
    y: point.y.clamp(min_y, max_y),
  }
}

fn default_ball_point(monitor: &tauri::Monitor, bw: i32) -> BallPoint {
  let scale = monitor.scale_factor();
  let size = monitor.size();
  let pos = monitor.position();
  let margin = (16.0 * scale) as i32;
  BallPoint {
    x: pos.x + size.width as i32 - bw - margin,
    y: pos.y + size.height as i32 / 2 - bw / 2,
  }
}

fn ball_position_path(app: &AppHandle) -> Option<std::path::PathBuf> {
  app
    .path()
    .app_data_dir()
    .ok()
    .map(|dir| dir.join("ball-position.json"))
}

fn read_ball_position(app: &AppHandle) -> Option<BallPoint> {
  let data = std::fs::read_to_string(ball_position_path(app)?).ok()?;
  serde_json::from_str(&data).ok()
}

fn write_ball_position(app: &AppHandle, point: BallPoint) {
  let Some(path) = ball_position_path(app) else {
    return;
  };
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if let Ok(data) = serde_json::to_string(&point) {
    let _ = std::fs::write(path, data);
  }
}

fn current_rest(app: &AppHandle) -> Option<BallPoint> {
  if let Some(rest) = app.state::<BallRest>().0.lock().ok().and_then(|slot| *slot) {
    return Some(rest);
  }
  read_ball_position(app)
}

fn remember_rest(app: &AppHandle, point: BallPoint) {
  if let Ok(mut slot) = app.state::<BallRest>().0.lock() {
    *slot = Some(point);
  }
  write_ball_position(app, point);
}

fn restore_ball(app: &AppHandle, ball: &tauri::WebviewWindow) {
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let scale = monitor.scale_factor();
  let bw = (64.0 * scale) as i32;
  let point = current_rest(app)
    .map(|point| clamp_ball_point(&monitor, point, bw, bw))
    .unwrap_or_else(|| default_ball_point(&monitor, bw));
  let _ = ball.set_size(tauri::LogicalSize::new(64.0, 64.0));
  let _ = ball.set_position(PhysicalPosition::new(point.x, point.y));
  let _ = ball.set_always_on_top(true);
  let _ = ball.show();
  let _ = ball.eval("document.body.classList.remove('tucked','tucked-left','tucked-right')");
  remember_rest(app, point);
}

fn hide_panel_and_restore(app: &AppHandle) {
  if let Some(panel) = app.get_webview_window("panel") {
    #[cfg(target_os = "windows")]
    set_taskbar_visible(&panel, false);
    let _ = panel.hide();
  }
  if let Some(ball) = app.get_webview_window("ball") {
    restore_ball(app, &ball);
  }
}

fn toggle(app: &AppHandle) {
  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  let Some(ball) = app.get_webview_window("ball") else {
    return;
  };
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };

  if panel.is_visible().unwrap_or(false) {
    hide_panel_and_restore(app);
    return;
  }

  let scale = monitor.scale_factor();
  let size = monitor.size();
  let pos = monitor.position();
  let right = pos.x + size.width as i32;
  let bottom = pos.y + size.height as i32;
  let bw = (64.0 * scale) as i32;
  let rest = ball
    .outer_position()
    .map(|point| BallPoint {
      x: point.x,
      y: point.y,
    })
    .unwrap_or_else(|_| default_ball_point(&monitor, bw));
  let rest = clamp_ball_point(&monitor, rest, bw, bw);
  remember_rest(app, rest);

  let pw = (400.0 * scale) as i32;
  let ph = (640.0 * scale) as i32;
  let margin = (16.0 * scale) as i32;
  let ball_center_x = rest.x + bw / 2;
  let ball_center_y = rest.y + bw / 2;
  let ball_on_left = ball_center_x < pos.x + size.width as i32 / 2;

  // 面板贴在球的一侧，垂直居中对齐；空间不足时向屏幕内收回。
  let candidate_x = if ball_on_left {
    rest.x + bw + margin
  } else {
    rest.x - margin - pw
  };
  let min_x = pos.x + margin;
  let max_x = (right - pw - margin).max(min_x);
  let x = candidate_x.clamp(min_x, max_x);
  let min_y = pos.y + margin;
  let max_y = (bottom - ph - margin).max(min_y);
  let y = (ball_center_y - ph / 2).clamp(min_y, max_y);

  let _ = panel.set_size(tauri::LogicalSize::new(400.0, 640.0));
  let _ = panel.set_position(PhysicalPosition::new(x, y));
  #[cfg(target_os = "windows")]
  remove_border(&panel);
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, false);
  let _ = panel.set_always_on_top(true);
  let _ = panel.show();
  let _ = panel.set_focus();
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, false);

  // 球收进离球最近的一侧屏幕边缘，保持当前 y；左侧镜像尾巴。
  let tuck = (32.0 * scale) as i32;
  let tuck_x = if ball_on_left { pos.x - tuck } else { right - tuck };
  let min_tuck_y = pos.y;
  let max_tuck_y = (bottom - bw).max(min_tuck_y);
  let tuck_y = (ball_center_y - tuck).clamp(min_tuck_y, max_tuck_y);
  let _ = ball.set_size(tauri::LogicalSize::new(64.0, 64.0));
  let _ = ball.set_position(PhysicalPosition::new(tuck_x, tuck_y));
  let _ = ball.set_always_on_top(true);
  let _ = ball.show();
  let _ = ball.eval("document.body.classList.remove('tucked','tucked-left','tucked-right')");
  let side_class = if ball_on_left {
    "tucked-left"
  } else {
    "tucked-right"
  };
  let _ = ball.eval(format!("document.body.classList.add('{side_class}')"));
}

#[tauri::command]
fn toggle_panel(app: AppHandle, state: State<'_, PanelState>) {
  let panel_visible = app
    .get_webview_window("panel")
    .map(|panel| panel.is_visible().unwrap_or(false))
    .unwrap_or(false);
  if !panel_visible {
    *state.0.lock().unwrap() = PanelMode::Mini;
  }
  toggle(&app);
}

#[tauri::command]
fn toggle_from_ball(app: AppHandle, state: State<'_, PanelState>) {
  let panel_visible = app
    .get_webview_window("panel")
    .map(|panel| panel.is_visible().unwrap_or(false))
    .unwrap_or(false);
  if panel_visible {
    hide_panel_and_restore(&app);
    return;
  }
  let mode = *state.0.lock().unwrap();
  match mode {
    PanelMode::Max => maximize_panel(app, state),
    PanelMode::Mini => {
      *state.0.lock().unwrap() = PanelMode::Mini;
      toggle(&app);
    }
  }
}

#[tauri::command]
fn maximize_panel(app: AppHandle, state: State<'_, PanelState>) {
  *state.0.lock().unwrap() = PanelMode::Max;
  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let work = monitor.work_area();
  let width = ((work.size.width as f64) * 0.9).round() as u32;
  let height = ((work.size.height as f64) * 0.9).round() as u32;
  let x = work.position.x + ((work.size.width - width) / 2) as i32;
  let y = work.position.y + ((work.size.height - height) / 2) as i32;
  let _ = panel.set_size(PhysicalSize::new(width, height));
  let _ = panel.set_position(PhysicalPosition::new(x, y));
  #[cfg(target_os = "windows")]
  remove_border(&panel);
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, true);
  let _ = panel.set_always_on_top(false);
  let _ = panel.show();
  let _ = panel.set_focus();
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, true);
  if let Some(ball) = app.get_webview_window("ball") {
    let _ = ball.hide();
  }
}

#[tauri::command]
fn restore_panel(app: AppHandle, state: State<'_, PanelState>) {
  *state.0.lock().unwrap() = PanelMode::Mini;
  if let Some(panel) = app.get_webview_window("panel") {
    #[cfg(target_os = "windows")]
    set_taskbar_visible(&panel, false);
    let _ = panel.hide();
  }
  if let Some(ball) = app.get_webview_window("ball") {
    restore_ball(&app, &ball);
  }
  toggle(&app);
}

#[tauri::command]
fn close_panel(app: AppHandle) {
  hide_panel_and_restore(&app);
}

#[tauri::command]
fn save_ball_position(app: AppHandle) {
  if let Some(panel) = app.get_webview_window("panel") {
    if panel.is_visible().unwrap_or(false) {
      return;
    }
  }
  let Some(ball) = app.get_webview_window("ball") else {
    return;
  };
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let scale = monitor.scale_factor();
  let bw = (64.0 * scale) as i32;
  let Ok(position) = ball.outer_position() else {
    return;
  };
  let point = clamp_ball_point(
    &monitor,
    BallPoint {
      x: position.x,
      y: position.y,
    },
    bw,
    bw,
  );
  let _ = ball.set_position(PhysicalPosition::new(point.x, point.y));
  remember_rest(&app, point);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      if let Some(panel) = app.get_webview_window("panel") {
        if !panel.is_visible().unwrap_or(false) {
          toggle(app);
        } else {
          let _ = panel.set_focus();
        }
      }
    }))
    .plugin(tauri_plugin_opener::init())
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
          if event.state == ShortcutState::Pressed {
            toggle(app);
          }
        })
        .build(),
    )
    .manage(BallRest::default())
    .manage(PanelState::default())
    .invoke_handler(tauri::generate_handler![
      toggle_panel,
      toggle_from_ball,
      close_panel,
      save_ball_position,
      maximize_panel,
      restore_panel
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let toggle_item = MenuItem::with_id(app, "toggle", "显示 / 隐藏", true, None::<&str>)?;
      let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;
      let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
          "toggle" => toggle(app),
          "quit" => app.exit(0),
          _ => {}
        });
      if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
      }
      let _ = tray.build(app);

      let _ = app.global_shortcut().register("Ctrl+Shift+Space");

      if let Some(panel) = app.get_webview_window("panel") {
        #[cfg(target_os = "windows")]
        {
          make_frameless(&panel);
          remove_border(&panel);
        }
        let _ = panel.set_size(tauri::LogicalSize::new(400.0, 640.0));
        let _ = panel.set_always_on_top(true);

        // 拖动放大/迷你窗口时始终保留在屏幕工作区内，避免标题栏和按钮被拖出屏幕。
        let app_handle = app.handle().clone();
        panel.on_window_event(move |event| {
          if let tauri::WindowEvent::Moved(position) = event {
            let Some(window) = app_handle.get_webview_window("panel") else {
              return;
            };
            if !window.is_visible().unwrap_or(false) {
              return;
            }
            if window.is_minimized().unwrap_or(false) {
              return;
            }
            let Ok(Some(monitor)) = app_handle.primary_monitor() else {
              return;
            };
            let Ok(size) = window.outer_size() else {
              return;
            };
            let work = monitor.work_area();
            let min_x = work.position.x;
            let max_x = (work.position.x + work.size.width as i32 - size.width as i32).max(min_x);
            let min_y = work.position.y;
            let max_y = (work.position.y + work.size.height as i32 - size.height as i32).max(min_y);
            let x = position.x.clamp(min_x, max_x);
            let y = position.y.clamp(min_y, max_y);
            if x != position.x || y != position.y {
              let _ = window.set_position(PhysicalPosition::new(x, y));
            }
          }
        });
      }

      if let Some(ball) = app.get_webview_window("ball") {
        #[cfg(target_os = "windows")]
        {
          make_frameless(&ball);
          remove_border(&ball);
        }
        let _ = ball.set_size(tauri::LogicalSize::new(64.0, 64.0));
        if let Ok(Some(monitor)) = app.primary_monitor() {
          let scale = monitor.scale_factor();
          let bw = (64.0 * scale) as i32;
          let point = read_ball_position(app.handle())
            .map(|point| clamp_ball_point(&monitor, point, bw, bw))
            .unwrap_or_else(|| default_ball_point(&monitor, bw));
          let _ = ball.set_position(PhysicalPosition::new(point.x, point.y));
          remember_rest(app.handle(), point);
        }
        let _ = ball.set_always_on_top(true);
        let _ = ball.show();
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while building tauri application");
}
