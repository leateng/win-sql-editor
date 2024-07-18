extern crate native_windows_gui as nwg;
// use nwg::stretch::style::*;
// use nwg::NativeUi;
use nwg::{ControlBase, ControlHandle, NwgError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi;
use winapi::um::winuser::{
    BS_BITMAP, BS_ICON, BS_NOTIFY, WS_CHILD, WS_DISABLED, WS_TABSTOP, WS_VISIBLE,
};

const SCI_SETCARETLINEVISIBLE: u32 = 0x200A;
const SCI_STYLESETFONT: u32 = 0x2FF8;
const SCI_STYLESETSIZE: u32 = 0x2FFB;
const SCI_SETTEXT: u32 = 0x000C;
const STYLE_DEFAULT: usize = 32;

extern "C" {
    pub fn Scintilla_RegisterClasses(
        hInstance: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}

pub fn register_window_class() -> bool {
    unsafe {
        let instance = winapi::um::libloaderapi::GetModuleHandleW(null_mut());
        let status = Scintilla_RegisterClasses(instance as *mut std::ffi::c_void);
        return status != 0;
    }
}

pub struct WString {
    inner: Vec<u16>,
}

impl WString {
    pub fn from_str(s: &str) -> WString {
        let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
        WString { inner: wide }
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.inner.as_ptr()
    }
}

pub struct ScintillaEditBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    parent: Option<ControlHandle>,
}

impl<'a> ScintillaEditBuilder<'a> {
    pub fn size(mut self, size: (i32, i32)) -> ScintillaEditBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> ScintillaEditBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> ScintillaEditBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut ScintillaEdit) -> Result<(), NwgError> {
        // let flags = self.flags.map(|f| f.bits()).unwrap_or(out.flags());
        let flags = out.flags();

        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("Scintilla")),
        }?;

        // Drop the old object
        *out = ScintillaEdit::default();

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(flags)
            // .ex_flags(self.ex_flags)
            .size(self.size)
            .position(self.position)
            .text(self.text)
            .parent(Some(parent))
            .build()?;

        Ok(())
    }
}

#[derive(Default, Eq, PartialEq)]
pub struct ScintillaEdit {
    pub handle: ControlHandle,
}

impl ScintillaEdit {
    pub fn builder<'a>() -> ScintillaEditBuilder<'a> {
        ScintillaEditBuilder {
            text: "",
            size: (100, 25),
            position: (0, 0),
            parent: None,
        }
    }

    /// Winapi class name used during control creation
    pub fn class_name(&self) -> &'static str {
        "Scintilla"
    }

    /// Winapi base flags used during window creation
    pub fn flags(&self) -> u32 {
        WS_VISIBLE | WS_TABSTOP | BS_NOTIFY
    }

    /// Winapi flags required by the control
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }

    // pub fn create(parent: &nwg::Window) -> Self {
    //     let instance = unsafe { winapi::um::libloaderapi::GetModuleHandleW(null_mut()) };
    //     let class_name = WString::from_str("Scintilla");
    //
    //     let hwnd = unsafe {
    //         winapi::um::winuser::CreateWindowExW(
    //             0,
    //             class_name.as_ptr(),
    //             null_mut(),
    //             winapi::um::winuser::WS_CHILD | winapi::um::winuser::WS_VISIBLE,
    //             0,
    //             0,
    //             800,
    //             600,
    //             parent.handle.hwnd().unwrap(),
    //             null_mut(),
    //             instance,
    //             null_mut(),
    //         )
    //     };
    //
    //     assert!(!hwnd.is_null(), "Failed to create Scintilla control");
    //
    //     let hwnd = nwg::ControlHandle::Hwnd(hwnd);
    //
    //     // 设置字体为 "Segoe UI Emoji"
    //     let font = WString::from_str("Segoe UI Emoji");
    //     unsafe {
    //         winapi::um::winuser::SendMessageW(
    //             hwnd.hwnd().unwrap(),
    //             SCI_STYLESETFONT,
    //             STYLE_DEFAULT as usize,
    //             font.as_ptr() as isize,
    //         );
    //         winapi::um::winuser::SendMessageW(
    //             hwnd.hwnd().unwrap(),
    //             SCI_STYLESETSIZE,
    //             STYLE_DEFAULT as usize,
    //             16,
    //         );
    //     }
    //
    //     // Example: set some text with emoji
    //     let text = WString::from_str("Hello, world! 😊🌍");
    //     unsafe {
    //         winapi::um::winuser::SendMessageW(
    //             hwnd.hwnd().unwrap(),
    //             SCI_SETTEXT,
    //             0,
    //             text.as_ptr() as isize,
    //         );
    //     }
    //
    //     ScintillaEdit { handle: hwnd }
    // }
}

// #[derive(Default, NativeUi)]
// pub struct MyApp {
//     #[nwg_control(size: (800, 600), position: (300, 300), title: "Scintilla Example")]
//     #[nwg_events(OnWindowClose: [MyApp::exit])]
//     window: nwg::Window,
//
//     #[nwg_control(parent: window, text: None)]
//     layout: nwg::Flexbox,
//
//     scintilla: ScintillaControl,
// }

// impl MyApp {
//     fn exit(&self) {
//         nwg::stop_thread_dispatch();
//     }
//
//     fn run() {
//         nwg::init().expect("Failed to initialize NWG");
//         let app = MyApp::build_ui(Default::default()).expect("Failed to build UI");
//
//         // 创建并添加 Scintilla 控件到布局
//         let parent = &app.window;
//         let scintilla = ScintillaControl::create(parent);
//         let hwnd = scintilla.hwnd.hwnd().unwrap();
//         nwg::Flexbox::attach(
//             ScintillaControl::create(parent).hwnd,
//             &app.layout,
//             nwg::stretch::geometry::Rect {
//                 start: 10.into(),
//                 end: 10.into(),
//                 top: 10.into(),
//                 bottom: 10.into(),
//             },
//         );
//
//         nwg::dispatch_thread_events();
//     }
// }
//
// fn main() {
//     MyApp::run();
// }
