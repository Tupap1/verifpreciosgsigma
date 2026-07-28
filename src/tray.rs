use tracing::info;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::HBRUSH;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW, GetCursorPos,
    GetMessageW, InsertMenuW, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    TrackPopupMenu, IDI_APPLICATION, MF_BYPOSITION, MF_STRING, MSG, TPM_BOTTOMALIGN,
    TPM_RIGHTALIGN, WM_COMMAND, WM_DESTROY, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const IDM_OPEN_BROWSER: usize = 1001;
const IDM_CHECK_UPDATE: usize = 1002;
const IDM_EXIT: usize = 1003;

pub fn start_system_tray(port: u16) {
    std::thread::spawn(move || unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class_name = encode_wide("GsigmaTrayClass");

        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: LoadIconW(0, IDI_APPLICATION),
            hCursor: 0,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            encode_wide("VerificadorGsigmaTray").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            0,
            0,
            instance,
            std::ptr::null(),
        );

        if hwnd == 0 {
            return;
        }

        // Add Tray Icon
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = LoadIconW(0, IDI_APPLICATION);

        let tip_str = format!("Verificador de Precios Gsigma (Online - Puerto {})", port);
        let tip_wide = encode_wide(&tip_str);
        let len = tip_wide.len().min(nid.szTip.len() - 1);
        nid.szTip[..len].copy_from_slice(&tip_wide[..len]);

        Shell_NotifyIconW(NIM_ADD, &nid);
        info!("Icono de bandeja de sistema (System Tray) activado correctamente junto al reloj");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &nid);
    });
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            if lparam as u32 == WM_RBUTTONUP {
                show_context_menu(hwnd);
            }
            0
        }
        WM_COMMAND => {
            let id = wparam as usize & 0xffff;
            match id {
                IDM_OPEN_BROWSER => {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "http://localhost:8080"])
                        .spawn();
                }
                IDM_CHECK_UPDATE => {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "https://github.com/Tupap1/verifpreciosgsigma/releases"])
                        .spawn();
                }
                IDM_EXIT => {
                    PostQuitMessage(0);
                    std::process::exit(0);
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    let mut pt = std::mem::zeroed();
    GetCursorPos(&mut pt);

    InsertMenuW(
        menu,
        0,
        MF_BYPOSITION | MF_STRING,
        IDM_OPEN_BROWSER,
        encode_wide("🌐 Abrir Verificador en Navegador").as_ptr(),
    );
    InsertMenuW(
        menu,
        1,
        MF_BYPOSITION | MF_STRING,
        IDM_CHECK_UPDATE,
        encode_wide("🔄 Buscar Actualizaciones en GitHub").as_ptr(),
    );
    InsertMenuW(
        menu,
        2,
        MF_BYPOSITION | MF_STRING,
        IDM_EXIT,
        encode_wide("❌ Salir del Servidor").as_ptr(),
    );

    SetForegroundWindow(hwnd);
    TrackPopupMenu(
        menu,
        TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    );
    DestroyMenu(menu);
}

fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
