//! Миниатюры окон и мониторов для окна выбора источника.
//!
//! Тот же подход, что в C++-версии: окно рисуем через `PrintWindow`,
//! монитор копируем с экрана через `BitBlt`.

use anyhow::{Context, Result, anyhow, bail};
use image::RgbImage;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC, HGDIOBJ, HMONITOR,
    ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

use crate::SourceRef;
use crate::sources::{monitor_rect, rect_size};

/// Просим окно отрисовать и DirectX-содержимое (браузеры, плееры, игры).
/// В windows-rs этой константы нет, значение из документации WinAPI.
const PW_RENDERFULLCONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(2);

/// Снимок источника, уменьшенный до `max_width` по ширине, в формате JPEG.
pub fn capture_jpeg(source: SourceRef, max_width: u32) -> Result<Vec<u8>> {
    let image = match source {
        SourceRef::Window(h) => capture_window(HWND(h as usize as *mut _))?,
        SourceRef::Monitor(h) => capture_monitor(HMONITOR(h as usize as *mut _))?,
    };
    let image = if image.width() > max_width {
        let height = (u64::from(image.height()) * u64::from(max_width) / u64::from(image.width()))
            .max(1) as u32;
        image::imageops::resize(&image, max_width, height, FilterType::Triangle)
    } else {
        image
    };

    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, 75).encode_image(&image)?;
    Ok(jpeg)
}

fn capture_window(hwnd: HWND) -> Result<RgbImage> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.context("окно не найдено")?;
    let (width, height) = checked_size(&rect)?;

    unsafe {
        let screen = GetDC(None);
        let canvas = Canvas::new(screen, width, height);
        let result = if PrintWindow(hwnd, canvas.dc, PW_RENDERFULLCONTENT).as_bool() {
            canvas.read(width, height)
        } else {
            Err(anyhow!("PrintWindow не смог отрисовать окно"))
        };
        drop(canvas);
        ReleaseDC(None, screen);
        result
    }
}

fn capture_monitor(hmon: HMONITOR) -> Result<RgbImage> {
    let rect = monitor_rect(hmon).context("монитор не найден")?;
    let (width, height) = checked_size(&rect)?;

    unsafe {
        let screen = GetDC(None);
        let canvas = Canvas::new(screen, width, height);
        let result = BitBlt(
            canvas.dc,
            0,
            0,
            width,
            height,
            Some(screen),
            rect.left,
            rect.top,
            SRCCOPY,
        )
        .context("BitBlt не смог скопировать экран")
        .and_then(|_| canvas.read(width, height));
        drop(canvas);
        ReleaseDC(None, screen);
        result
    }
}

fn checked_size(rect: &RECT) -> Result<(i32, i32)> {
    match rect_size(rect) {
        (0, _) | (_, 0) => bail!("у источника нулевой размер"),
        (w, h) => Ok((w as i32, h as i32)),
    }
}

/// Bitmap в памяти, в который рисуется снимок. GDI-объекты освобождаются в `Drop`.
struct Canvas {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
}

impl Canvas {
    unsafe fn new(screen: HDC, width: i32, height: i32) -> Self {
        unsafe {
            let dc = CreateCompatibleDC(Some(screen));
            let bitmap = CreateCompatibleBitmap(screen, width, height);
            let previous = SelectObject(dc, bitmap.into());
            Self { dc, bitmap, previous }
        }
    }

    unsafe fn read(&self, width: i32, height: i32) -> Result<RgbImage> {
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                // Отрицательная высота — строки сверху вниз, переворачивать не нужно.
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; width as usize * height as usize * 4];
        let lines = unsafe {
            // GetDIBits требует, чтобы bitmap не был выбран в DC.
            SelectObject(self.dc, self.previous);
            GetDIBits(
                self.dc,
                self.bitmap,
                0,
                height as u32,
                Some(bgra.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            )
        };
        if lines == 0 {
            bail!("GetDIBits не вернул ни одной строки");
        }
        let rgb = bgra
            .chunks_exact(4)
            .flat_map(|px| [px[2], px[1], px[0]])
            .collect();
        RgbImage::from_raw(width as u32, height as u32, rgb).context("неверный размер буфера")
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.dc);
        }
    }
}
