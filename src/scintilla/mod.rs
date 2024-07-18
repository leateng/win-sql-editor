extern crate native_windows_gui as nwg;
// use nwg::stretch::style::*;
// use nwg::NativeUi;
use nwg::{ControlBase, ControlHandle, NwgError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi;
use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_VISIBLE};

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
    ex_flags: u32,
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
            .ex_flags(self.ex_flags)
            .size(self.size)
            .position(self.position)
            .text(self.text)
            .parent(Some(parent))
            .build()?;

        // 设置字体为 "Segoe UI Emoji"
        let font = WString::from_str("Segoe UI Emoji");
        unsafe {
            winapi::um::winuser::SendMessageW(
                out.handle.hwnd().unwrap(),
                SCI_STYLESETFONT,
                STYLE_DEFAULT as usize,
                font.as_ptr() as isize,
            );
            winapi::um::winuser::SendMessageW(
                out.handle.hwnd().unwrap(),
                SCI_STYLESETSIZE,
                STYLE_DEFAULT as usize,
                20,
            );
        }

        // // Example: set some text with emoji
        // let text = WString::from_str("Hello, world! 😊🌍");
        // unsafe {
        //     winapi::um::winuser::SendMessageW(
        //         out.handle.hwnd().unwrap(),
        //         SCI_SETTEXT,
        //         0,
        //         text.as_ptr() as isize,
        //     );
        // }

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
            ex_flags: 0,
            parent: None,
        }
    }

    /// Winapi class name used during control creation
    pub fn class_name(&self) -> &'static str {
        "Scintilla"
    }

    /// Winapi base flags used during window creation
    pub fn ex_flags(&self) -> u32 {
        WS_EX_CLIENTEDGE
    }

    /// Winapi base flags used during window creation
    pub fn flags(&self) -> u32 {
        WS_VISIBLE
    }

    /// Winapi flags required by the control
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD
    }
}
