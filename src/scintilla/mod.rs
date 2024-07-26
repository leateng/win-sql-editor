mod bindings;
pub use bindings::*;

extern crate native_windows_gui as nwg;
use crate::lexilla::CreateLexer;
use nwg::{ControlBase, ControlHandle, NwgError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::{isize, mem};
use winapi;
use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_VISIBLE};

// static DEFAULT_FONT: String = "Courier New".into();
static mut SCI_FN_DIRECT: SciFnDirect = None;

pub struct WString {
    inner: Vec<u16>,
}

pub struct ScintillaEditBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    ex_flags: u32,
    parent: Option<ControlHandle>,
}

#[derive(Default, Eq, PartialEq)]
pub struct ScintillaEdit {
    pub handle: ControlHandle,
    sci_direct_ptr: sptr_t,
    font: String,
    lexer: String,
}

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

impl WString {
    pub fn from_str(s: &str) -> WString {
        let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
        WString { inner: wide }
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.inner.as_ptr()
    }
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

        out.sci_direct_ptr = unsafe {
            winapi::um::winuser::SendMessageW(
                out.handle.hwnd().unwrap(),
                SCI_GETDIRECTPOINTER,
                0 as usize,
                0 as isize,
            )
        };
        out.font = "Courier New".into();
        out.lexer = "sql".into();

        out.set_technology(SC_TECHNOLOGY_DIRECTWRITERETAIN as usize);
        out.set_font_quality(SC_EFF_QUALITY_ANTIALIASED as usize);
        out.sci_call(SCI_SETIMEINTERACTION, SC_IME_INLINE as usize, 0);
        out.sci_call(
            SCI_STYLESETFONT,
            STYLE_DEFAULT as usize,
            out.font.as_ptr() as isize,
        );
        out.set_font_size(12);

        let ilexer = unsafe { CreateLexer(out.lexer.as_ptr() as *const std::ffi::c_char) };

        println!("ilexer = {:?}", ilexer);
        out.sci_call(SCI_SETILEXER, 0 as usize, ilexer as isize);

        // Example: set some text with emoji
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

    pub fn sci_call(&self, i_message: ::std::os::raw::c_uint, w_param: uptr_t, l_param: sptr_t) {
        unsafe {
            match SCI_FN_DIRECT {
                None => {
                    let fp = winapi::um::winuser::SendMessageW(
                        self.handle.hwnd().unwrap(),
                        SCI_GETDIRECTFUNCTION,
                        0 as usize,
                        0 as isize,
                    );

                    SCI_FN_DIRECT = mem::transmute(fp);
                    if let Some(fp) = SCI_FN_DIRECT {
                        fp(self.sci_direct_ptr, i_message, w_param, l_param);
                    }
                }
                Some(fp) => {
                    fp(self.sci_direct_ptr, i_message, w_param, l_param);
                }
            }
        }
    }

    pub fn set_font_size(&self, font_size: isize) {
        self.sci_call(SCI_STYLESETSIZE, STYLE_DEFAULT as usize, font_size);
    }

    pub fn set_technology(&self, technology: usize) {
        self.sci_call(SCI_SETTECHNOLOGY, technology, 0);
    }

    pub fn set_font_quality(&self, font_quality: usize) {
        self.sci_call(SCI_SETFONTQUALITY, font_quality, 0);
    }
}

impl From<ControlHandle> for ScintillaEdit {
    fn from(handle: ControlHandle) -> ScintillaEdit {
        let sci_direct_ptr = unsafe {
            winapi::um::winuser::SendMessageW(
                handle.hwnd().unwrap(),
                SCI_GETDIRECTPOINTER,
                0 as usize,
                0 as isize,
            )
        };
        ScintillaEdit {
            handle,
            sci_direct_ptr,
            font: String::from("Courier New"),
            lexer: String::from("sql"),
        }
    }
}

impl Into<ControlHandle> for &ScintillaEdit {
    fn into(self) -> ControlHandle {
        self.handle
    }
}
