use std::ffi::c_void;
use std::io::Cursor;
use std::sync::Mutex;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use image::{DynamicImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State};
use windows::Win32::Graphics::Gdi::{
  BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
  ReleaseDC, SelectObject, BITMAPINFO, BI_RGB, DIB_RGB_COLORS, HGDIOBJ, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
  GetSystemMetrics, SetForegroundWindow, ShowWindow, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
  SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_SHOW,
};

#[derive(Clone)]
struct CapturedScreen {
  png: Vec<u8>,
  data_url: String,
  width: u32,
  height: u32,
  origin_x: i32,
  origin_y: i32,
}

#[derive(Default)]
pub struct CaptureState {
  captured: Mutex<Option<CapturedScreen>>,
  panel_was_visible: Mutex<bool>,
  ball_was_visible: Mutex<bool>,
}

#[derive(Deserialize)]
pub struct SelectionRect {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

#[derive(Serialize)]
pub struct CaptureImagePayload {
  #[serde(rename = "dataUrl")]
  data_url: String,
  width: u32,
  height: u32,
}

#[derive(Clone, Serialize)]
struct ImageActionPayload {
  action: String,
  #[serde(rename = "dataUrl")]
  data_url: String,
}

fn virtual_screen() -> (i32, i32, i32, i32) {
  let x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
  let y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
  let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) }.max(1);
  let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) }.max(1);
  (x, y, width, height)
}

fn capture_screen() -> Result<CapturedScreen, String> {
  let (origin_x, origin_y, width, height) = virtual_screen();
  let width_u32 = width as u32;
  let height_u32 = height as u32;
  let byte_len = width as usize * height as usize * 4;
  let mut buffer = vec![0u8; byte_len];

  unsafe {
    let screen_dc = GetDC(None);
    if screen_dc.is_invalid() {
      return Err("screen capture unavailable".to_string());
    }
    let memory_dc = CreateCompatibleDC(Some(screen_dc));
    if memory_dc.is_invalid() {
      ReleaseDC(None, screen_dc);
      return Err("screen capture unavailable".to_string());
    }
    let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
    if bitmap.is_invalid() {
      let _ = DeleteDC(memory_dc);
      ReleaseDC(None, screen_dc);
      return Err("screen capture unavailable".to_string());
    }
    let previous = SelectObject(memory_dc, bitmap.into());
    let copied = BitBlt(
      memory_dc,
      0,
      0,
      width,
      height,
      Some(screen_dc),
      origin_x,
      origin_y,
      SRCCOPY,
    );
    if copied.is_err() {
      SelectObject(memory_dc, previous);
      let _ = DeleteObject(HGDIOBJ(bitmap.0));
      let _ = DeleteDC(memory_dc);
      ReleaseDC(None, screen_dc);
      return Err("screen capture failed".to_string());
    }

    let mut info = BITMAPINFO::default();
    info.bmiHeader.biSize = std::mem::size_of_val(&info.bmiHeader) as u32;
    info.bmiHeader.biWidth = width;
    info.bmiHeader.biHeight = -height;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB.0;
    let lines = GetDIBits(
      memory_dc,
      bitmap,
      0,
      height_u32,
      Some(buffer.as_mut_ptr() as *mut c_void),
      &mut info,
      DIB_RGB_COLORS,
    );
    SelectObject(memory_dc, previous);
    let _ = DeleteObject(HGDIOBJ(bitmap.0));
    let _ = DeleteDC(memory_dc);
    ReleaseDC(None, screen_dc);
    if lines != height {
      return Err("screen capture failed".to_string());
    }
  }

  for pixel in buffer.chunks_exact_mut(4) {
    pixel.swap(0, 2);
    pixel[3] = 255;
  }
  let image = RgbaImage::from_raw(width_u32, height_u32, buffer)
    .ok_or_else(|| "screen capture failed".to_string())?;
  let mut png = Vec::new();
  DynamicImage::ImageRgba8(image)
    .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
    .map_err(|error| error.to_string())?;
  let data_url = format!("data:image/png;base64,{}", BASE64.encode(&png));
  Ok(CapturedScreen {
    png,
    data_url,
    width: width_u32,
    height: height_u32,
    origin_x,
    origin_y,
  })
}

fn hide_capture_window(app: &AppHandle) {
  if let Some(window) = app.get_webview_window("capture") {
    let _ = window.hide();
  }
}

fn restore_app_windows(app: &AppHandle, state: &CaptureState) {
  let panel_visible = state
    .panel_was_visible
    .lock()
    .map(|value| *value)
    .unwrap_or(false);
  let ball_visible = state
    .ball_was_visible
    .lock()
    .map(|value| *value)
    .unwrap_or(false);
  if panel_visible {
    if let Some(panel) = app.get_webview_window("panel") {
      let _ = panel.show();
      let _ = panel.set_focus();
      if let Some(ball) = app.get_webview_window("ball") {
        let _ = ball.hide();
      }
      return;
    }
  }
  if ball_visible {
    if let Some(ball) = app.get_webview_window("ball") {
      let _ = ball.show();
    }
  }
}

pub fn start_capture(app: &AppHandle) {
  let state = app.state::<CaptureState>();
  if state
    .captured
    .lock()
    .map(|value| value.is_some())
    .unwrap_or(false)
  {
    return;
  }
  let Some(capture_window) = app.get_webview_window("capture") else {
    return;
  };

  let panel = app.get_webview_window("panel");
  let ball = app.get_webview_window("ball");
  let panel_visible = panel
    .as_ref()
    .map(|window| window.is_visible().unwrap_or(false))
    .unwrap_or(false);
  let ball_visible = ball
    .as_ref()
    .map(|window| window.is_visible().unwrap_or(false))
    .unwrap_or(false);
  if let Ok(mut value) = state.panel_was_visible.lock() {
    *value = panel_visible;
  }
  if let Ok(mut value) = state.ball_was_visible.lock() {
    *value = ball_visible;
  }

  if let Some(window) = app.get_webview_window("selection") {
    let _ = window.hide();
  }
  if let Some(window) = &panel {
    let _ = window.hide();
  }
  if let Some(window) = &ball {
    let _ = window.hide();
  }

  // Let the hidden windows settle before taking the snapshot.
  std::thread::sleep(Duration::from_millis(110));
  let captured = match capture_screen() {
    Ok(captured) => captured,
    Err(error) => {
      log::warn!("capture failed: {error}");
      restore_app_windows(app, &state);
      return;
    }
  };
  if let Ok(mut value) = state.captured.lock() {
    *value = Some(captured);
  }

  let (x, y, width, height) = virtual_screen();
  let _ = capture_window.set_position(PhysicalPosition::new(x, y));
  let _ = capture_window.set_size(PhysicalSize::new(width as u32, height as u32));
  let _ = capture_window.show();
  let _ = capture_window.set_focus();
  unsafe {
    if let Ok(hwnd) = capture_window.hwnd() {
      let _ = SetForegroundWindow(hwnd);
      let _ = ShowWindow(hwnd, SW_SHOW);
    }
  }
}

#[tauri::command]
pub fn capture_image(state: State<'_, CaptureState>) -> Result<CaptureImagePayload, String> {
  let captured = state
    .captured
    .lock()
    .map_err(|_| "capture unavailable".to_string())?
    .clone()
    .ok_or_else(|| "capture unavailable".to_string())?;
  Ok(CaptureImagePayload {
    data_url: captured.data_url,
    width: captured.width,
    height: captured.height,
  })
}

fn crop_capture(captured: &CapturedScreen, rect: &SelectionRect) -> Result<RgbaImage, String> {
  let image = image::load_from_memory(&captured.png)
    .map_err(|error| error.to_string())?
    .to_rgba8();
  let x = (rect.x.clamp(0.0, 1.0) * captured.width as f64).round() as u32;
  let y = (rect.y.clamp(0.0, 1.0) * captured.height as f64).round() as u32;
  let width = (rect.width.clamp(0.0, 1.0) * captured.width as f64).round() as u32;
  let height = (rect.height.clamp(0.0, 1.0) * captured.height as f64).round() as u32;
  let width = width.min(captured.width.saturating_sub(x));
  let height = height.min(captured.height.saturating_sub(y));
  if width < 2 || height < 2 {
    return Err("selection too small".to_string());
  }
  Ok(image::imageops::crop_imm(&image, x, y, width, height).to_image())
}

fn encode_png(image: RgbaImage) -> Result<(Vec<u8>, String), String> {
  let mut png = Vec::new();
  DynamicImage::ImageRgba8(image)
    .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
    .map_err(|error| error.to_string())?;
  let data_url = format!("data:image/png;base64,{}", BASE64.encode(&png));
  Ok((png, data_url))
}

fn copy_image_to_clipboard(image: &RgbaImage) -> Result<(), String> {
  let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
  clipboard
    .set_image(arboard::ImageData {
      width: image.width() as usize,
      height: image.height() as usize,
      bytes: std::borrow::Cow::Owned(image.clone().into_raw()),
    })
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn capture_action(
  app: AppHandle,
  state: State<'_, CaptureState>,
  action: String,
  rect: SelectionRect,
) -> Result<(), String> {
  let captured = state
    .captured
    .lock()
    .map_err(|_| "capture unavailable".to_string())?
    .take()
    .ok_or_else(|| "capture unavailable".to_string())?;
  let cropped = crop_capture(&captured, &rect)?;
  let (_, data_url) = encode_png(cropped.clone())?;
  hide_capture_window(&app);

  if action == "copy" {
    let result = copy_image_to_clipboard(&cropped);
    restore_app_windows(&app, &state);
    return result;
  }

  let anchor = (
    captured.origin_x
      + ((rect.x.clamp(0.0, 1.0) + rect.width.clamp(0.0, 1.0) / 2.0) * captured.width as f64)
        as i32,
    captured.origin_y
      + ((rect.y.clamp(0.0, 1.0) + rect.height.clamp(0.0, 1.0) / 2.0) * captured.height as f64)
        as i32,
  );
  if let Err(error) = crate::selection::open_compact_panel(&app, anchor) {
    restore_app_windows(&app, &state);
    return Err(error);
  }
  let _ = app.emit_to(
    "panel",
    "selection-image-action",
    ImageActionPayload {
      action,
      data_url,
    },
  );
  Ok(())
}

#[tauri::command]
pub fn capture_cancel(app: AppHandle, state: State<'_, CaptureState>) {
  hide_capture_window(&app);
  if let Ok(mut captured) = state.captured.lock() {
    *captured = None;
  }
  restore_app_windows(&app, &state);
}
