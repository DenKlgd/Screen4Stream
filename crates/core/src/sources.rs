//! Перечисление окон и мониторов, которые можно транслировать.

use std::ffi::c_void;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, bail};
use serde::Serialize;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GW_OWNER, GWL_EXSTYLE, GetWindow, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    MONITORINFOF_PRIMARY,
    WS_EX_TOOLWINDOW,
};
use windows::core::{BOOL, PWSTR};

/// Ссылка на источник захвата: окно (HWND) или монитор (HMONITOR).
///
/// В API передаётся строкой: `win-1a2b3c` или `mon-10001`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRef {
    Window(u64),
    Monitor(u64),
}

impl fmt::Display for SourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceRef::Window(h) => write!(f, "win-{h:x}"),
            SourceRef::Monitor(h) => write!(f, "mon-{h:x}"),
        }
    }
}

impl FromStr for SourceRef {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let (kind, hex) = s
            .split_once('-')
            .ok_or_else(|| anyhow!("неверный id источника: {s}"))?;
        let handle = u64::from_str_radix(hex, 16)?;
        match kind {
            "win" => Ok(SourceRef::Window(handle)),
            "mon" => Ok(SourceRef::Monitor(handle)),
            _ => bail!("неизвестный тип источника: {kind}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Window,
    Monitor,
}

/// Источник, как его видит пользователь в окне выбора.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub kind: SourceKind,
    /// Заголовок окна или «Экран N».
    pub title: String,
    /// Имя программы или разрешение монитора.
    pub subtitle: String,
    pub width: u32,
    pub height: u32,
    /// Свёрнутое окно захватить нельзя: Windows его не отрисовывает.
    pub minimized: bool,
}

/// Окна верхнего уровня — примерно тот же список, что в Alt+Tab.
pub fn windows() -> Vec<Source> {
    let mut handles: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect_window), LPARAM(&mut handles as *mut _ as isize));
    }
    let own_pid = std::process::id();
    handles
        .into_iter()
        .filter_map(|hwnd| describe_window(hwnd, own_pid))
        .collect()
}

/// Все подключённые мониторы.
pub fn monitors() -> Vec<Source> {
    let mut handles: Vec<HMONITOR> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect_monitor),
            LPARAM(&mut handles as *mut _ as isize),
        );
    }
    handles
        .into_iter()
        .enumerate()
        .filter_map(|(index, hmon)| {
            let info = monitor_info(hmon)?;
            let (width, height) = rect_size(&info.rcMonitor);
            let primary = info.dwFlags & MONITORINFOF_PRIMARY != 0;
            Some(Source {
                id: SourceRef::Monitor(hmon.0 as usize as u64).to_string(),
                kind: SourceKind::Monitor,
                title: format!("Экран {}", index + 1),
                subtitle: if primary {
                    format!("{width} × {height} · основной")
                } else {
                    format!("{width} × {height}")
                },
                width,
                height,
                minimized: false,
            })
        })
        .collect()
}

/// Прямоугольник монитора в координатах рабочего стола.
pub(crate) fn monitor_rect(hmon: HMONITOR) -> Option<RECT> {
    monitor_info(hmon).map(|info| info.rcMonitor)
}

unsafe extern "system" fn collect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let handles = unsafe { &mut *(lparam.0 as *mut Vec<HWND>) };
    handles.push(hwnd);
    true.into()
}

unsafe extern "system" fn collect_monitor(
    hmon: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let handles = unsafe { &mut *(lparam.0 as *mut Vec<HMONITOR>) };
    handles.push(hmon);
    true.into()
}

fn describe_window(hwnd: HWND, own_pid: u32) -> Option<Source> {
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return None;
        }
        // Диалоги (у них есть окно-владелец) и служебные окна в Alt+Tab не показываются.
        if GetWindow(hwnd, GW_OWNER).is_ok_and(|owner| !owner.is_invalid()) {
            return None;
        }
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if ex_style & WS_EX_TOOLWINDOW.0 != 0 || is_cloaked(hwnd) {
            return None;
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == own_pid {
            return None;
        }

        let title = window_title(hwnd)?;
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).ok()?;
        let (width, height) = rect_size(&rect);
        let minimized = IsIconic(hwnd).as_bool();
        if !minimized && (width == 0 || height == 0) {
            return None;
        }

        Some(Source {
            id: SourceRef::Window(hwnd.0 as usize as u64).to_string(),
            kind: SourceKind::Window,
            title,
            subtitle: process_name(pid).unwrap_or_default(),
            width,
            height,
            minimized,
        })
    }
}

/// «Спрятанные» окна: приложения UWP в фоне, окна на других виртуальных столах.
fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut c_void,
            size_of::<u32>() as u32,
        )
    };
    result.is_ok() && cloaked != 0
}

fn window_title(hwnd: HWND) -> Option<String> {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return None;
    }
    let mut buf = vec![0u16; len as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buf) };
    let title = String::from_utf16_lossy(&buf[..copied.max(0) as usize]);
    let title = title.trim();
    (!title.is_empty()).then(|| title.to_owned())
}

/// Имя exe-файла без расширения: `Code`, `firefox`, `Telegram`.
fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);
        result.ok()?;
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        Path::new(&path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
    }
}

fn monitor_info(hmon: HMONITOR) -> Option<MONITORINFO> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    unsafe { GetMonitorInfoW(hmon, &mut info) }
        .as_bool()
        .then_some(info)
}

pub(crate) fn rect_size(rect: &RECT) -> (u32, u32) {
    (
        (rect.right - rect.left).max(0) as u32,
        (rect.bottom - rect.top).max(0) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ref_round_trip() {
        for source in [SourceRef::Window(0x1a2b3c), SourceRef::Monitor(0x10001)] {
            assert_eq!(source.to_string().parse::<SourceRef>().unwrap(), source);
        }
        assert!("tab-12".parse::<SourceRef>().is_err());
        assert!("win-xyz".parse::<SourceRef>().is_err());
    }
}
