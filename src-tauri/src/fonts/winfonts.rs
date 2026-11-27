//! Parte específica de Windows para instalar fuentes SOLO para el usuario actual (sin administrador).
//! Windows 10 (1809+) y Windows 11 admiten fuentes por usuario: archivo en
//! %LOCALAPPDATA%\Microsoft\Windows\Fonts + valor en HKCU\...\CurrentVersion\Fonts.
use std::path::{Path, PathBuf};

pub fn user_fonts_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|l| {
        PathBuf::from(l)
            .join("Microsoft")
            .join("Windows")
            .join("Fonts")
    })
}

/// Registra la fuente ya copiada y avisa a las aplicaciones abiertas.
#[cfg(windows)]
pub fn register_user_font(dest: &Path, value_name: &str) -> Result<(), String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows NT\CurrentVersion\Fonts")
        .map_err(|e| e.to_string())?;
    // Las fuentes por usuario se registran con la ruta absoluta.
    key.set_value(value_name, &dest.to_string_lossy().to_string())
        .map_err(|e| e.to_string())?;
    notify_font_change(dest);
    Ok(())
}

#[cfg(windows)]
fn notify_font_change(path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Gdi::AddFontResourceW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_FONTCHANGE,
    };
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: `wide` termina en NUL y vive durante la llamada; el resto son constantes del sistema.
    unsafe {
        AddFontResourceW(wide.as_ptr());
        let mut result: usize = 0;
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_FONTCHANGE,
            0,
            0,
            SMTO_ABORTIFHUNG,
            1000,
            &mut result,
        );
    }
}

#[cfg(not(windows))]
pub fn register_user_font(_dest: &Path, _value_name: &str) -> Result<(), String> {
    Err("Installing fonts is only supported on Windows".into())
}
