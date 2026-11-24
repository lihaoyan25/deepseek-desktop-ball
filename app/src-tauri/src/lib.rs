use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};
use tauri_plugin_global_shortcut::{
  Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

mod capture;
mod selection;

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
fn make_noactivate(window: &tauri::WebviewWindow) {
  use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
  };

  if let Ok(hwnd) = window.hwnd() {
    unsafe {
      let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
      let new_ex = ex | WS_EX_NOACTIVATE.0 as isize | WS_EX_TOOLWINDOW.0 as isize;
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct PanelSize {
  width: f64,
  height: f64,
  #[serde(default)]
  x: Option<i32>,
  #[serde(default)]
  y: Option<i32>,
}

#[derive(Default, Serialize, Deserialize)]
struct PanelSizes {
  mini: Option<PanelSize>,
  max: Option<PanelSize>,
}

const MIN_PANEL_WIDTH: f64 = 320.0;
const MIN_PANEL_HEIGHT: f64 = 400.0;
const DEFAULT_MINI_WIDTH: f64 = 400.0;
const DEFAULT_MINI_HEIGHT: f64 = 640.0;

struct PanelState {
  mode: Mutex<PanelMode>,
  sizes: Mutex<PanelSizes>,
  last_size_save: Mutex<Instant>,
  temporary_compact: AtomicBool,
  resume_mode: Mutex<Option<PanelMode>>,
  animating: AtomicBool,
  animation_generation: AtomicU64,
}

impl Default for PanelState {
  fn default() -> Self {
    Self {
      mode: Mutex::new(PanelMode::Mini),
      sizes: Mutex::new(PanelSizes::default()),
      last_size_save: Mutex::new(Instant::now()),
      temporary_compact: AtomicBool::new(false),
      resume_mode: Mutex::new(None),
      animating: AtomicBool::new(false),
      animation_generation: AtomicU64::new(0),
    }
  }
}

#[derive(Clone, Copy, Debug)]
struct WindowRect {
  x: i32,
  y: i32,
  width: u32,
  height: u32,
}

const PANEL_ANIMATION_MS: u64 = 200;
const PANEL_OPEN_SCALE: f64 = 0.96;

fn panel_rect(window: &tauri::WebviewWindow) -> Option<WindowRect> {
  let position = window.outer_position().ok()?;
  let size = window.outer_size().ok()?;
  Some(WindowRect {
    x: position.x,
    y: position.y,
    width: size.width,
    height: size.height,
  })
}

fn set_panel_rect(window: &tauri::WebviewWindow, rect: WindowRect) {
  #[cfg(target_os = "windows")]
  {
    use windows::Win32::UI::WindowsAndMessaging::{
      SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
    };
    if let Ok(hwnd) = window.hwnd() {
      unsafe {
        let _ = SetWindowPos(
          hwnd,
          None,
          rect.x,
          rect.y,
          rect.width as i32,
          rect.height as i32,
          SWP_NOACTIVATE | SWP_NOZORDER,
        );
      }
      return;
    }
  }
  let _ = window.set_size(tauri::PhysicalSize::new(rect.width, rect.height));
  let _ = window.set_position(PhysicalPosition::new(rect.x, rect.y));
}

fn scaled_rect(rect: WindowRect, factor: f64) -> WindowRect {
  let width = (rect.width as f64 * factor).round().max(1.0) as u32;
  let height = (rect.height as f64 * factor).round().max(1.0) as u32;
  WindowRect {
    x: rect.x + ((rect.width - width) / 2) as i32,
    y: rect.y + ((rect.height - height) / 2) as i32,
    width,
    height,
  }
}

fn lerp_rect(start: WindowRect, end: WindowRect, t: f64) -> WindowRect {
  let value = |a: i32, b: i32| (a as f64 + (b - a) as f64 * t).round() as i32;
  WindowRect {
    x: value(start.x, end.x),
    y: value(start.y, end.y),
    width: value(start.width as i32, end.width as i32).max(1) as u32,
    height: value(start.height as i32, end.height as i32).max(1) as u32,
  }
}

fn bezier(a: f64, b: f64, t: f64) -> f64 {
  let u = 1.0 - t;
  3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
}

/** 与会话搜索相同的 cubic-bezier(0.2, 0, 0, 1)。 */
fn panel_ease(t: f64) -> f64 {
  let t = t.clamp(0.0, 1.0);
  let mut low = 0.0;
  let mut high = 1.0;
  for _ in 0..10 {
    let mid = (low + high) / 2.0;
    if bezier(0.2, 0.0, mid) < t {
      low = mid;
    } else {
      high = mid;
    }
  }
  bezier(0.0, 1.0, (low + high) / 2.0)
}

fn set_panel_animation_class(app: &AppHandle, active: bool) {
  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  let script = if active {
    "document.body.classList.add('window-animating')"
  } else {
    "document.body.classList.remove('window-animating')"
  };
  let _ = panel.eval(script);
}

fn begin_panel_animation(app: &AppHandle) -> u64 {
  let state = app.state::<PanelState>();
  set_panel_animation_class(app, true);
  state.animating.store(true, Ordering::Relaxed);
  state.animation_generation.fetch_add(1, Ordering::Relaxed) + 1
}

fn finish_panel_animation(app: &AppHandle, generation: u64) {
  let state = app.state::<PanelState>();
  if state.animation_generation.load(Ordering::Relaxed) == generation {
    state.animating.store(false, Ordering::Relaxed);
    set_panel_animation_class(app, false);
  }
}

fn spawn_panel_animation(
  app: AppHandle,
  panel: tauri::WebviewWindow,
  start: WindowRect,
  target: WindowRect,
  generation: u64,
) {
  thread::spawn(move || {
    let began = Instant::now();
    let duration = Duration::from_millis(PANEL_ANIMATION_MS);
    #[cfg(target_os = "windows")]
    use windows::Win32::Graphics::Dwm::DwmFlush;

    loop {
      if !panel.is_visible().unwrap_or(false) {
        finish_panel_animation(&app, generation);
        return;
      }
      let state = app.state::<PanelState>();
      if state.animation_generation.load(Ordering::Relaxed) != generation {
        return;
      }
      let frame_started = Instant::now();
      let t = (frame_started.duration_since(began).as_secs_f64() / duration.as_secs_f64()).min(1.0);
      set_panel_rect(&panel, lerp_rect(start, target, panel_ease(t)));
      if t >= 1.0 {
        break;
      }
      #[cfg(target_os = "windows")]
      {
        // Let DWM present this frame before scheduling the next one.
        let _ = unsafe { DwmFlush() };
      }
      #[cfg(not(target_os = "windows"))]
      {
        thread::sleep(Duration::from_millis(16));
      }
    }
    set_panel_rect(&panel, target);
    finish_panel_animation(&app, generation);
  });
}

fn animate_panel_open(
  app: &AppHandle,
  panel: &tauri::WebviewWindow,
  target: WindowRect,
) -> u64 {
  let start = scaled_rect(target, PANEL_OPEN_SCALE);
  let generation = begin_panel_animation(app);
  set_panel_rect(panel, start);
  generation
}

/** 划词紧凑窗口只借用当前主窗口，不覆盖用户原有的尺寸、位置和模式。 */
pub(crate) fn begin_temporary_compact(app: &AppHandle) {
  let state = app.state::<PanelState>();
  if state.temporary_compact.swap(true, Ordering::Relaxed) {
    return;
  }
  if let (Ok(mode), Ok(mut resume)) = (state.mode.lock(), state.resume_mode.lock()) {
    *resume = Some(*mode);
  }
}

pub(crate) fn end_temporary_compact(app: &AppHandle) {
  let state = app.state::<PanelState>();
  if !state.temporary_compact.swap(false, Ordering::Relaxed) {
    return;
  }
  let resume = state.resume_mode.lock().ok().and_then(|mut mode| mode.take());
  if let Some(mode) = resume {
    if let Ok(mut current) = state.mode.lock() {
      *current = mode;
    }
    emit_panel_mode(app, mode);
  }
}

fn panel_sizes_path(app: &AppHandle) -> Option<std::path::PathBuf> {
  app
    .path()
    .app_data_dir()
    .ok()
    .map(|dir| dir.join("panel-sizes.json"))
}

fn read_panel_sizes(app: &AppHandle) -> PanelSizes {
  panel_sizes_path(app)
    .and_then(|path| std::fs::read_to_string(path).ok())
    .and_then(|data| serde_json::from_str(&data).ok())
    .unwrap_or_default()
}

fn write_panel_sizes(app: &AppHandle, sizes: &PanelSizes) {
  let Some(path) = panel_sizes_path(app) else {
    return;
  };
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  if let Ok(data) = serde_json::to_string(sizes) {
    let _ = std::fs::write(path, data);
  }
}

fn clamp_panel_size(size: PanelSize, monitor: &tauri::Monitor) -> PanelSize {
  let scale = monitor.scale_factor();
  let work = monitor.work_area();
  let max_width = work.size.width as f64 / scale;
  let max_height = work.size.height as f64 / scale;
  PanelSize {
    width: size
      .width
      .clamp(MIN_PANEL_WIDTH, max_width.max(MIN_PANEL_WIDTH)),
    height: size
      .height
      .clamp(MIN_PANEL_HEIGHT, max_height.max(MIN_PANEL_HEIGHT)),
    x: size.x,
    y: size.y,
  }
}

fn clamp_panel_point(monitor: &tauri::Monitor, point: BallPoint, pw: i32, ph: i32) -> BallPoint {
  let work = monitor.work_area();
  let min_x = work.position.x;
  let max_x = (work.position.x + work.size.width as i32 - pw).max(min_x);
  let min_y = work.position.y;
  let max_y = (work.position.y + work.size.height as i32 - ph).max(min_y);
  BallPoint {
    x: point.x.clamp(min_x, max_x),
    y: point.y.clamp(min_y, max_y),
  }
}

fn default_max_panel_size(monitor: &tauri::Monitor) -> PanelSize {
  let scale = monitor.scale_factor();
  let work = monitor.work_area();
  PanelSize {
    width: (work.size.width as f64 / scale * 0.9).round(),
    height: (work.size.height as f64 / scale * 0.9).round(),
    x: None,
    y: None,
  }
}

fn saved_panel_size(state: &PanelState, mode: PanelMode) -> Option<PanelSize> {
  let sizes = state.sizes.lock().ok()?;
  match mode {
    PanelMode::Mini => sizes.mini,
    PanelMode::Max => sizes.max,
  }
}

fn emit_panel_mode(app: &AppHandle, mode: PanelMode) {
  let value = match mode {
    PanelMode::Mini => "mini",
    PanelMode::Max => "max",
  };
  let _ = app.emit("panel-mode", value);
}

fn remember_panel_size(app: &AppHandle, size: PanelSize, force: bool) {
  let state = app.state::<PanelState>();
  if state.temporary_compact.load(Ordering::Relaxed)
    || state.animating.load(Ordering::Relaxed)
  {
    return;
  }
  let mode = match state.mode.lock() {
    Ok(mode) => *mode,
    Err(_) => return,
  };
  let mut sizes = match state.sizes.lock() {
    Ok(sizes) => sizes,
    Err(_) => return,
  };
  let current = match mode {
    PanelMode::Mini => sizes.mini,
    PanelMode::Max => sizes.max,
  };
  let mut next = size;
  if next.x.is_none() || next.y.is_none() {
    next.x = next.x.or(current.and_then(|size| size.x));
    next.y = next.y.or(current.and_then(|size| size.y));
  }
  match mode {
    PanelMode::Mini => sizes.mini = Some(next),
    PanelMode::Max => sizes.max = Some(next),
  }
  let should_write = force
    || state
      .last_size_save
      .lock()
      .map(|mut last| {
        if last.elapsed() < Duration::from_millis(200) {
          false
        } else {
          *last = Instant::now();
          true
        }
      })
      .unwrap_or(true);
  if should_write {
    write_panel_sizes(app, &sizes);
  }
}

fn remember_current_panel_size(app: &AppHandle, force: bool) {
  let state = app.state::<PanelState>();
  if state.temporary_compact.load(Ordering::Relaxed)
    || state.animating.load(Ordering::Relaxed)
  {
    return;
  }
  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  if !panel.is_visible().unwrap_or(false) {
    return;
  }
  let Ok(size) = panel.inner_size() else {
    return;
  };
  let Ok(scale) = panel.scale_factor() else {
    return;
  };
  let logical = size.to_logical::<f64>(scale);
  let position = panel.outer_position().ok();
  remember_panel_size(
    app,
    PanelSize {
      width: logical.width,
      height: logical.height,
      x: position.map(|point| point.x),
      y: position.map(|point| point.y),
    },
    force,
  );
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
  let _ = ball.eval(
    "document.body.classList.remove('tucked','tucked-left','tucked-right','ball-leaving')",
  );
  remember_rest(app, point);
}

fn show_tucked_ball(app: &AppHandle, ball: &tauri::WebviewWindow) {
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let scale = monitor.scale_factor();
  let bw = (64.0 * scale) as i32;
  let work = monitor.work_area();
  let rest = current_rest(app)
    .map(|point| clamp_ball_point(&monitor, point, bw, bw))
    .unwrap_or_else(|| default_ball_point(&monitor, bw));
  let ball_on_left = rest.x + bw / 2 < work.position.x + work.size.width as i32 / 2;
  let tuck = (32.0 * scale) as i32;
  let tuck_x = if ball_on_left {
    work.position.x - tuck
  } else {
    work.position.x + work.size.width as i32 - tuck
  };
  let tuck_y = (rest.y + bw / 2 - tuck).clamp(
    work.position.y,
    (work.position.y + work.size.height as i32 - bw).max(work.position.y),
  );
  let _ = ball.set_size(tauri::LogicalSize::new(64.0, 64.0));
  let _ = ball.set_position(PhysicalPosition::new(tuck_x, tuck_y));
  let _ = ball.set_always_on_top(true);
  let _ = ball.show();
  let side_class = if ball_on_left {
    "tucked-left"
  } else {
    "tucked-right"
  };
  let _ = ball.eval(format!(
    "document.body.classList.remove('tucked','tucked-left','tucked-right','ball-leaving'); document.body.classList.add('{side_class}')"
  ));
}

fn hide_panel_and_restore(app: &AppHandle) {
  if let Some(panel) = app.get_webview_window("panel") {
    #[cfg(target_os = "windows")]
    set_taskbar_visible(&panel, false);
    let _ = panel.hide();
  }
  end_temporary_compact(app);
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
    remember_current_panel_size(app, true);
    hide_panel_and_restore(app);
    return;
  }

  let state = app.state::<PanelState>();
  if let Ok(mut mode) = state.mode.lock() {
    *mode = PanelMode::Mini;
  }
  emit_panel_mode(app, PanelMode::Mini);
  let saved = saved_panel_size(&state, PanelMode::Mini);
  let saved_position = saved.and_then(|size| size.x.zip(size.y));
  let size = clamp_panel_size(
    saved.unwrap_or(PanelSize {
      width: DEFAULT_MINI_WIDTH,
      height: DEFAULT_MINI_HEIGHT,
      x: None,
      y: None,
    }),
    &monitor,
  );

  let scale = monitor.scale_factor();
  let size_px = monitor.size();
  let pos = monitor.position();
  let right = pos.x + size_px.width as i32;
  let bottom = pos.y + size_px.height as i32;
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

  let pw = (size.width * scale).round() as i32;
  let ph = (size.height * scale).round() as i32;
  let margin = (16.0 * scale) as i32;
  let ball_center_x = rest.x + bw / 2;
  let ball_center_y = rest.y + bw / 2;
  let ball_on_left = ball_center_x < pos.x + size_px.width as i32 / 2;

  let candidate_x = if ball_on_left {
    rest.x + bw + margin
  } else {
    rest.x - margin - pw
  };
  let min_x = pos.x + margin;
  let max_x = (right - pw - margin).max(min_x);
  let min_y = pos.y + margin;
  let max_y = (bottom - ph - margin).max(min_y);
  let (x, y) = if let Some(point) = saved_position {
    let point = clamp_panel_point(&monitor, BallPoint { x: point.0, y: point.1 }, pw, ph);
    (point.x, point.y)
  } else {
    let x = candidate_x.clamp(min_x, max_x);
    let y = (ball_center_y - ph / 2).clamp(min_y, max_y);
    (x, y)
  };

  let target = WindowRect {
    x,
    y,
    width: pw as u32,
    height: ph as u32,
  };
  let start = scaled_rect(target, PANEL_OPEN_SCALE);
  let generation = animate_panel_open(app, &panel, target);
  #[cfg(target_os = "windows")]
  remove_border(&panel);
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, false);
  let _ = panel.set_always_on_top(true);
  let _ = panel.show();
  let _ = panel.set_focus();
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, false);
  spawn_panel_animation(app.clone(), panel.clone(), start, target, generation);

  let tuck = (32.0 * scale) as i32;  let tuck_x = if ball_on_left { pos.x - tuck } else { right - tuck };
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
    if let Ok(mut mode) = state.mode.lock() {
      *mode = PanelMode::Mini;
    }
    emit_panel_mode(&app, PanelMode::Mini);
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
    remember_current_panel_size(&app, true);
    hide_panel_and_restore(&app);
    return;
  }
  let mode = match state.mode.lock() {
    Ok(mode) => *mode,
    Err(_) => PanelMode::Mini,
  };
  match mode {
    PanelMode::Max => maximize_panel(app, state),
    PanelMode::Mini => {
      if let Ok(mut mode) = state.mode.lock() {
        *mode = PanelMode::Mini;
      }
      emit_panel_mode(&app, PanelMode::Mini);
      toggle(&app);
    }
  }
}

#[tauri::command]
fn maximize_panel(app: AppHandle, state: State<'_, PanelState>) {
  remember_current_panel_size(&app, true);
  if let Ok(mut mode) = state.mode.lock() {
    *mode = PanelMode::Max;
  }
  emit_panel_mode(&app, PanelMode::Max);
  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let scale = monitor.scale_factor();
  let work = monitor.work_area();
  let saved = saved_panel_size(&state, PanelMode::Max);
  let saved_position = saved.and_then(|size| size.x.zip(size.y));
  let size = clamp_panel_size(
    saved.unwrap_or_else(|| default_max_panel_size(&monitor)),
    &monitor,
  );
  let width = (size.width * scale).round() as u32;
  let height = (size.height * scale).round() as u32;
  let (x, y) = if let Some(point) = saved_position {
    let point = clamp_panel_point(
      &monitor,
      BallPoint {
        x: point.0,
        y: point.1,
      },
      width as i32,
      height as i32,
    );
    (point.x, point.y)
  } else {
    (
      work.position.x + ((work.size.width - width) / 2) as i32,
      work.position.y + ((work.size.height - height) / 2) as i32,
    )
  };
  let target = WindowRect {
    x,
    y,
    width,
    height,
  };
  let generation = begin_panel_animation(&app);
  let start = if panel.is_visible().unwrap_or(false) {
    panel_rect(&panel).unwrap_or(target)
  } else {
    let start = scaled_rect(target, PANEL_OPEN_SCALE);
    set_panel_rect(&panel, start);
    start
  };
  #[cfg(target_os = "windows")]
  remove_border(&panel);
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, true);
  let _ = panel.set_always_on_top(false);
  let _ = panel.show();
  let _ = panel.set_focus();
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, true);
  spawn_panel_animation(app.clone(), panel.clone(), start, target, generation);
  if let Some(ball) = app.get_webview_window("ball") {
    let _ = ball.hide();
  }
}

#[tauri::command]
fn restore_panel(app: AppHandle, state: State<'_, PanelState>) {
  remember_current_panel_size(&app, true);
  if let Ok(mut mode) = state.mode.lock() {
    *mode = PanelMode::Mini;
  }
  emit_panel_mode(&app, PanelMode::Mini);
  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let scale = monitor.scale_factor();
  let work = monitor.work_area();
  let saved = saved_panel_size(&state, PanelMode::Mini);
  let saved_position = saved.and_then(|size| size.x.zip(size.y));
  let size = clamp_panel_size(
    saved.unwrap_or(PanelSize {
      width: DEFAULT_MINI_WIDTH,
      height: DEFAULT_MINI_HEIGHT,
      x: None,
      y: None,
    }),
    &monitor,
  );
  let width = (size.width * scale).round() as u32;
  let height = (size.height * scale).round() as u32;
  let (x, y) = if let Some(point) = saved_position {
    let point = clamp_panel_point(
      &monitor,
      BallPoint {
        x: point.0,
        y: point.1,
      },
      width as i32,
      height as i32,
    );
    (point.x, point.y)
  } else {
    let current = panel.outer_position().unwrap_or(PhysicalPosition::new(
      work.position.x,
      work.position.y,
    ));
    let x = current
      .x
      .clamp(work.position.x, (work.position.x + work.size.width as i32 - width as i32).max(work.position.x));
    let y = current
      .y
      .clamp(work.position.y, (work.position.y + work.size.height as i32 - height as i32).max(work.position.y));
    (x, y)
  };
  let target = WindowRect {
    x,
    y,
    width,
    height,
  };
  let generation = begin_panel_animation(&app);
  let start = panel_rect(&panel).unwrap_or(target);
  #[cfg(target_os = "windows")]
  set_taskbar_visible(&panel, false);
  let _ = panel.set_always_on_top(true);
  let _ = panel.show();
  let _ = panel.set_focus();
  spawn_panel_animation(app.clone(), panel.clone(), start, target, generation);

  if let Some(ball) = app.get_webview_window("ball") {
    let app_handle = app.clone();
    thread::spawn(move || {
      thread::sleep(Duration::from_millis(PANEL_ANIMATION_MS + 50));
      if let Some(panel) = app_handle.get_webview_window("panel") {
        if panel.is_visible().unwrap_or(false) {
          show_tucked_ball(&app_handle, &ball);
        }
      }
    });
  }
}

#[tauri::command]
fn close_panel(app: AppHandle) {
  remember_current_panel_size(&app, true);
  hide_panel_and_restore(&app);
}

#[tauri::command]
fn panel_mode(state: State<'_, PanelState>) -> String {
  let mode = match state.mode.lock() {
    Ok(mode) => *mode,
    Err(_) => PanelMode::Mini,
  };
  match mode {
    PanelMode::Mini => "mini".to_string(),
    PanelMode::Max => "max".to_string(),
  }
}

#[tauri::command]
fn reset_panel_size(app: AppHandle, state: State<'_, PanelState>) {
  let Ok(Some(monitor)) = app.primary_monitor() else {
    return;
  };
  let mode = match state.mode.lock() {
    Ok(mode) => *mode,
    Err(_) => PanelMode::Mini,
  };
  let max_default = default_max_panel_size(&monitor);
  let mut sizes = match state.sizes.lock() {
    Ok(sizes) => sizes,
    Err(_) => return,
  };
  let mini_position = sizes.mini.and_then(|size| size.x.zip(size.y));
  let max_position = sizes.max.and_then(|size| size.x.zip(size.y));
  sizes.mini = Some(PanelSize {
    width: DEFAULT_MINI_WIDTH,
    height: DEFAULT_MINI_HEIGHT,
    x: mini_position.map(|point| point.0),
    y: mini_position.map(|point| point.1),
  });
  sizes.max = Some(PanelSize {
    width: max_default.width,
    height: max_default.height,
    x: max_position.map(|point| point.0),
    y: max_position.map(|point| point.1),
  });
  write_panel_sizes(&app, &sizes);
  let size = match mode {
    PanelMode::Mini => sizes.mini,
    PanelMode::Max => sizes.max,
  };
  drop(sizes);

  let Some(panel) = app.get_webview_window("panel") else {
    return;
  };
  let Some(size) = size else {
    return;
  };
  let scale = monitor.scale_factor();
  let _ = panel.set_size(tauri::LogicalSize::new(size.width, size.height));
  if let (Some(x), Some(y)) = (size.x, size.y) {
    let point = clamp_panel_point(
      &monitor,
      BallPoint { x, y },
      (size.width * scale).round() as i32,
      (size.height * scale).round() as i32,
    );
    let _ = panel.set_position(PhysicalPosition::new(point.x, point.y));
  }
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn start_ball_drag(app: AppHandle) {
  use windows::Win32::Foundation::POINT;
  use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
  use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, SetWindowPos, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_NOZORDER,
  };

  let Some(ball) = app.get_webview_window("ball") else {
    return;
  };
  let Ok(hwnd) = ball.hwnd() else {
    return;
  };
  let hwnd_addr = hwnd.0 as isize;
  let Ok(start_pos) = ball.outer_position() else {
    return;
  };

  thread::spawn(move || {
    let hwnd = windows::Win32::Foundation::HWND(hwnd_addr as *mut core::ffi::c_void);
    let mut start_cursor = POINT::default();
    if unsafe { GetCursorPos(&mut start_cursor) }.is_err() {
      return;
    }

    // 自定义拖动：直接跟随物理光标位置移动窗口，避免 Windows 原生
    // startDragging 在关闭“拖动时显示窗口内容”时画出的线框。
    loop {
      let pressed = (unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } as u16 & 0x8000) != 0;
      if !pressed {
        break;
      }
      let mut cursor = POINT::default();
      if unsafe { GetCursorPos(&mut cursor) }.is_err() {
        break;
      }
      let x = start_pos.x + cursor.x - start_cursor.x;
      let y = start_pos.y + cursor.y - start_cursor.y;
      let _ = unsafe {
        SetWindowPos(
          hwnd,
          None,
          x,
          y,
          0,
          0,
          SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )
      };
      thread::sleep(Duration::from_millis(8));
    }

    save_ball_position(app);
  });
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
        .with_handler(|app, shortcut, event| {
          if event.state == ShortcutState::Pressed {
            if shortcut.matches(
              Modifiers::CONTROL | Modifiers::SHIFT,
              Code::KeyA,
            ) {
              capture::start_capture(app);
            } else {
              toggle(app);
            }
          }
        })
        .build(),
    )
    .manage(BallRest::default())
    .manage(PanelState::default())
    .manage(selection::SelectionState::default())
    .manage(capture::CaptureState::default())
    .invoke_handler(tauri::generate_handler![
      toggle_panel,
      toggle_from_ball,
      close_panel,
      save_ball_position,
      start_ball_drag,
      maximize_panel,
      restore_panel,
      panel_mode,
      reset_panel_size,
      selection::set_selection_assistant_enabled,
      selection::selection_action,
      capture::capture_image,
      capture::capture_action,
      capture::capture_cancel
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
      let screenshot_item = MenuItem::with_id(app, "screenshot", "截图问答", true, None::<&str>)?;
      let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&toggle_item, &screenshot_item, &quit_item])?;
      let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
          "toggle" => toggle(app),
          "screenshot" => capture::start_capture(app),
          "quit" => {
            remember_current_panel_size(app, true);
            app.exit(0);
          }
          _ => {}
        });
      if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
      }
      let _ = tray.build(app);

      let _ = app.global_shortcut().register("Ctrl+Shift+Space");
      let _ = app.global_shortcut().register(Shortcut::new(
        Some(Modifiers::CONTROL | Modifiers::SHIFT),
        Code::KeyA,
      ));

      if let Some(capture_window) = app.get_webview_window("capture") {
        #[cfg(target_os = "windows")]
        {
          make_frameless(&capture_window);
          remove_border(&capture_window);
        }
        let _ = capture_window.set_skip_taskbar(true);
        let _ = capture_window.set_always_on_top(true);
      }

      if let Some(selection_window) = app.get_webview_window("selection") {
        #[cfg(target_os = "windows")]
        {
          make_frameless(&selection_window);
          make_noactivate(&selection_window);
          remove_border(&selection_window);
        }
        let _ = selection_window.set_skip_taskbar(true);
        let _ = selection_window.set_always_on_top(true);
      }

      selection::start(app.handle().clone());

      if let Some(panel) = app.get_webview_window("panel") {
        #[cfg(target_os = "windows")]
        {
          make_frameless(&panel);
          remove_border(&panel);
        }

        let loaded_sizes = read_panel_sizes(app.handle());
        if let Ok(mut sizes) = app.state::<PanelState>().sizes.lock() {
          *sizes = loaded_sizes;
        }
        let initial = app
          .state::<PanelState>()
          .sizes
          .lock()
          .ok()
          .and_then(|sizes| sizes.mini)
          .unwrap_or(PanelSize {
            width: DEFAULT_MINI_WIDTH,
            height: DEFAULT_MINI_HEIGHT,
            x: None,
            y: None,
          });
        let _ = panel.set_size(tauri::LogicalSize::new(initial.width, initial.height));
        let _ = panel.set_always_on_top(true);

        let app_handle = app.handle().clone();
        panel.on_window_event(move |event| {
          match event {
            tauri::WindowEvent::Moved(position) => {
              if app_handle
                .state::<PanelState>()
                .animating
                .load(Ordering::Relaxed)
              {
                return;
              }
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
              remember_current_panel_size(&app_handle, false);
            }
            tauri::WindowEvent::Resized(size) => {
              if app_handle
                .state::<PanelState>()
                .animating
                .load(Ordering::Relaxed)
              {
                return;
              }
              let Some(window) = app_handle.get_webview_window("panel") else {
                return;
              };
              if !window.is_visible().unwrap_or(false) {
                return;
              }
              if window.is_minimized().unwrap_or(false) {
                return;
              }
              let Ok(scale) = window.scale_factor() else {
                return;
              };
              let Ok(Some(monitor)) = app_handle.primary_monitor() else {
                return;
              };
              let logical = size.to_logical::<f64>(scale);
              let clamped = clamp_panel_size(
                PanelSize {
                  width: logical.width,
                  height: logical.height,
                  x: None,
                  y: None,
                },
                &monitor,
              );
              if (clamped.width - logical.width).abs() > 0.5
                || (clamped.height - logical.height).abs() > 0.5
              {
                let _ = window.set_size(tauri::LogicalSize::new(clamped.width, clamped.height));
                return;
              }
              remember_panel_size(&app_handle, clamped, false);
            }
            _ => {}
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
