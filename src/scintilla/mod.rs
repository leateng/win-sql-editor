mod bindings;
pub use bindings::*;

extern crate native_windows_gui as nwg;
use crate::lexilla::{
    self, SCE_SQL_CHARACTER, SCE_SQL_COMMENTDOC, SCE_SQL_DEFAULT, SCE_SQL_NUMBER, SCE_SQL_SQLPLUS,
    SCE_SQL_STRING, SCE_SQL_WORD, SCE_SQL_WORD2,
};
use crate::lexilla::{CreateLexer, SCE_SQL_COMMENT, SCE_SQL_COMMENTLINE};
// use nwg::TabsContainer;
// use nwg::{bind_raw_event_handler, Event, EventData, RawEventHandler};
use nwg::{ControlBase, ControlHandle, NwgError};
// use std::cell::RefCell;
// use std::ffi::OsStr;
// use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::{isize, mem};
// use winapi;
use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_VISIBLE};

static mut SCI_FN_DIRECT: SciFnDirect = None;

macro_rules! rgb_to_bgr {
    ($rgb:expr) => {{
        let rgb = $rgb;
        let r = (rgb >> 16) & 0xFF;
        let g = (rgb >> 8) & 0xFF;
        let b = rgb & 0xFF;
        (b << 16) | (g << 8) | r
    }};
}

pub struct ScintillaEditBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    ex_flags: u32,
    parent: Option<ControlHandle>,
}

#[derive(Default, PartialEq, Clone)]
pub struct ScintillaEdit {
    pub handle: ControlHandle,
    sci_direct_ptr: sptr_t,
    font: String,
    lexer: String,
}

impl PartialEq<ScintillaEdit> for ControlHandle {
    fn eq(&self, other: &ScintillaEdit) -> bool {
        println!("PartialEq");
        *self == other.handle
    }
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
        status != 0
    }
}

// pub struct WString {
//     inner: Vec<u16>,
// }

// impl WString {
//     pub fn from_str(s: &str) -> WString {
//         let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
//         WString { inner: wide }
//     }
//
//     pub fn as_ptr(&self) -> *const u16 {
//         self.inner.as_ptr()
//     }
// }

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
                0_usize,
                0_isize,
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

        // static LEX_NAME: &str = "sql";
        let ilexer = unsafe { CreateLexer("sql".as_ptr() as *const i8) };
        // let ilexer = unsafe { CreateLexer(out.lexer.as_ptr() as *const std::ffi::c_char) };
        println!("ilexer = {:?}", ilexer);
        out.sci_call(SCI_SETILEXER, 0_usize, ilexer as isize);

        // style defines
        // default
        out.sci_call(
            SCI_STYLESETFORE,
            STYLE_DEFAULT as usize,
            rgb_to_bgr!(0xABB2BF),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            STYLE_DEFAULT as usize,
            rgb_to_bgr!(0x282c34),
        );

        //sql default
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_DEFAULT as usize,
            rgb_to_bgr!(0xABB2BF),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_DEFAULT as usize,
            rgb_to_bgr!(0x282c34),
        );

        // comment
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_COMMENT as usize,
            rgb_to_bgr!(0x7F848E),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_COMMENT as usize,
            rgb_to_bgr!(0x282c34),
        );

        // comment line
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_COMMENTLINE as usize,
            rgb_to_bgr!(0x7F848E),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_COMMENTLINE as usize,
            rgb_to_bgr!(0x282c34),
        );

        // comment doc
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_COMMENTDOC as usize,
            rgb_to_bgr!(0x7F848E),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_COMMENTDOC as usize,
            rgb_to_bgr!(0x282c34),
        );

        // number
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_NUMBER as usize,
            rgb_to_bgr!(0xD19A66),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_NUMBER as usize,
            rgb_to_bgr!(0x282c34),
        );

        // keyword
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_WORD as usize,
            rgb_to_bgr!(0xC678dd),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_WORD as usize,
            rgb_to_bgr!(0x282c34),
        );

        // double quote string
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_STRING as usize,
            rgb_to_bgr!(0x98C379),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_STRING as usize,
            rgb_to_bgr!(0x282c34),
        );

        // character
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_CHARACTER as usize,
            rgb_to_bgr!(0xFF000),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_CHARACTER as usize,
            rgb_to_bgr!(0x282c34),
        );

        // sql plus
        out.sci_call(
            SCI_STYLESETFORE,
            SCE_SQL_SQLPLUS as usize,
            rgb_to_bgr!(0xFF000),
        );
        out.sci_call(
            SCI_STYLESETBACK,
            SCE_SQL_SQLPLUS as usize,
            rgb_to_bgr!(0x282c34),
        );

        // out.sci_call(SCI_STYLESETFORE, SCE_SQL_WORD2 as usize, 0xDD78C6);
        // out.sci_call(
        //     SCI_STYLESETBACK,
        //     SCE_SQL_WORD2 as usize,
        //     rgb_to_bgr!(0x282c34),
        // );

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

        // events, observe scintilla events on it's parent control
        let hwnd = out.handle.hwnd().unwrap();
        let edit2 = out.clone();
        let _ = nwg::bind_raw_event_handler(&parent, 0xFFFF + 100, move |_hwnd, msg, w, l| {
            // use winapi::shared::minwindef::{HIWORD, LOWORD};
            use winapi::um::winuser::{NMHDR, WM_NOTIFY};

            if msg == WM_NOTIFY {
                let nmhdr: &NMHDR = unsafe { &*(l as *const NMHDR) };

                // handle the message send from current scintilla control
                if nmhdr.hwndFrom == hwnd {
                    match nmhdr.code {
                        SCN_MODIFIED => {
                            edit2.sci_call(SCI_COLOURISE, 0, -1);
                            // 例如 SCN_MODIFIED 事件
                            // println!("Text modified!");
                            // let scn: &SCNotification = unsafe { &*(w as *const SCNotification) };
                            // println!("scn = {:?}", scn);
                        }
                        _ => {}
                    }
                }
            }
            None
        });

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
                        0_usize,
                        0_isize,
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

    pub fn on_resize(&self) {
        println!("resize scintilla");
    }
    pub fn setup_event(&self) {
        println!("setup scintilla event");
    }
}

impl From<&ScintillaEdit> for ControlHandle {
    fn from(val: &ScintillaEdit) -> Self {
        val.handle
    }
}
