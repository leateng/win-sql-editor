mod bindings;
pub use bindings::*;

extern crate native_windows_gui as nwg;
use crate::lexilla::{
    create_lexer, ILexer5, SCE_SQL_CHARACTER, SCE_SQL_COMMENTDOC, SCE_SQL_COMMENTDOCKEYWORD,
    SCE_SQL_COMMENTDOCKEYWORDERROR, SCE_SQL_DEFAULT, SCE_SQL_IDENTIFIER, SCE_SQL_NUMBER,
    SCE_SQL_OPERATOR, SCE_SQL_SQLPLUS, SCE_SQL_SQLPLUS_PROMPT, SCE_SQL_STRING, SCE_SQL_USER1,
    SCE_SQL_USER2, SCE_SQL_USER3, SCE_SQL_USER4, SCE_SQL_WORD, SCE_SQL_WORD2,
};
use crate::lexilla::{SCE_SQL_COMMENT, SCE_SQL_COMMENTLINE};
// use nwg::TabsContainer;
// use nwg::{bind_raw_event_handler, Event, EventData, RawEventHandler};
use nwg::{ControlBase, ControlHandle, NwgError};
use std::ffi::{CStr, CString};
// use std::cell::RefCell;
// use std::ffi::OsStr;
// use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::{isize, mem};
// use winapi;
use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_VISIBLE};

static mut SCI_FN_DIRECT: SciFnDirect = None;

macro_rules! scintilla_rgb_color {
    ($hex:expr) => {{
        let hex = $hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u32::from_str_radix(&hex[0..2], 16).expect("Invalid hex format");
            let g = u32::from_str_radix(&hex[2..4], 16).expect("Invalid hex format");
            let b = u32::from_str_radix(&hex[4..6], 16).expect("Invalid hex format");
            (b << 16) | (g << 8) | r
        } else {
            panic!("Invalid RGB format, expected #RRGGBB");
        }
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

        out.set_technology(SC_TECHNOLOGY_DIRECTWRITERETAIN as usize);
        out.set_font_quality(SC_EFF_QUALITY_ANTIALIASED as usize);
        out.set_wrap_mode(SC_WRAP_WORD as usize);
        out.set_wrap_visual_flags(SC_WRAPVISUALFLAG_START as usize);
        out.set_eol_mode(SC_EOL_CRLF as usize);
        out.set_tab_width(2 as usize);
        out.set_end_at_last_line(true);
        out.set_ime_interaction(SC_IME_INLINE as usize);
        out.style_set_font(STYLE_DEFAULT as usize, "Cascadia Code");
        out.set_font_size(12);

        let ilexer = create_lexer("sql").unwrap();
        out.set_ilexer(ilexer);
        out.set_sql_lexer_keywords();
        out.setup_color_scheme();
        out.setup_caret(2, 0xFFE75C27);

        // events, observe scintilla events on it's parent control
        let hwnd = out.handle.hwnd().unwrap();
        let edit2 = out.clone();
        let _ = nwg::bind_raw_event_handler(&parent, 0xFFFF + 100, move |_hwnd, msg, _w, l| {
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

    pub fn set_wrap_mode(&self, wrap_mode: usize) {
        self.sci_call(SCI_SETWRAPMODE, wrap_mode, 0);
    }

    pub fn set_wrap_visual_flags(&self, wrap_visual_flags: usize) {
        self.sci_call(SCI_SETWRAPVISUALFLAGS, wrap_visual_flags, 0);
    }

    pub fn set_eol_mode(&self, eol_mode: usize) {
        self.sci_call(SCI_SETEOLMODE, eol_mode, 0);
    }

    pub fn set_tab_width(&self, tab_width: usize) {
        self.sci_call(SCI_SETTABWIDTH, tab_width, 0);
    }

    pub fn set_ilexer(&self, ilexer: *mut ILexer5) {
        self.sci_call(SCI_SETILEXER, 0_usize, ilexer as isize);
    }

    pub fn style_set_font(&self, style: usize, font: &str) {
        let c_string = CString::new(font).expect("CString::new failed");
        let c_str: &CStr = c_string.as_c_str();
        self.sci_call(SCI_STYLESETFONT, style as usize, c_str.as_ptr() as isize);
    }

    pub fn set_ime_interaction(&self, ime_interaction: usize) {
        self.sci_call(SCI_SETIMEINTERACTION, ime_interaction as usize, 0);
    }

    pub fn set_end_at_last_line(&self, end_at_last_line: bool) {
        let mut end = 1;
        if end_at_last_line == true {
            end = 0;
        }
        self.sci_call(SCI_SETENDATLASTLINE, end, 0);
    }

    pub fn set_lexer_elem_color(&self, elem: u32, fore: u32, back: u32) {
        self.sci_call(SCI_STYLESETFORE, elem as usize, fore as isize);
        self.sci_call(SCI_STYLESETBACK, elem as usize, back as isize);
    }

    pub fn set_sql_lexer_keywords(&self) {
        let keywords = CString::new("select from where order group having").unwrap();
        self.sci_call(SCI_SETKEYWORDS, 1_usize, keywords.as_ptr() as isize);
    }

    pub fn setup_color_scheme(&self) {
        let default_bg = scintilla_rgb_color!("#282C34");
        let default_fg = scintilla_rgb_color!("#ABB2BF");
        let comment_fg = scintilla_rgb_color!("#7F848E");
        let number_fg = scintilla_rgb_color!("#D19A66");
        let keyword_fg = scintilla_rgb_color!("#C678DD");
        let string_fg = scintilla_rgb_color!("#98C379");

        // style defines
        // default
        self.set_lexer_elem_color(STYLE_DEFAULT, default_fg, default_bg);

        //sql default
        self.set_lexer_elem_color(SCE_SQL_DEFAULT, default_fg, default_bg);

        // comment
        self.set_lexer_elem_color(SCE_SQL_COMMENT, comment_fg, default_bg);

        // comment line
        self.set_lexer_elem_color(SCE_SQL_COMMENTLINE, comment_fg, default_bg);

        // comment doc
        self.set_lexer_elem_color(SCE_SQL_COMMENTDOC, comment_fg, default_bg);

        // number
        self.set_lexer_elem_color(SCE_SQL_NUMBER, number_fg, default_bg);

        // keyword
        self.set_lexer_elem_color(SCE_SQL_WORD, keyword_fg, default_bg);

        // double quote string
        self.set_lexer_elem_color(SCE_SQL_STRING, string_fg, default_bg);

        // character, single quote string
        self.set_lexer_elem_color(SCE_SQL_CHARACTER, string_fg, default_bg);

        // sql plus
        self.set_lexer_elem_color(SCE_SQL_SQLPLUS, 0xFF000, default_bg);

        // sql plus prompt
        self.set_lexer_elem_color(SCE_SQL_SQLPLUS_PROMPT, 0xFF000, default_bg);

        // operator
        self.set_lexer_elem_color(SCE_SQL_OPERATOR, default_fg, default_bg);

        // identifer
        self.set_lexer_elem_color(SCE_SQL_IDENTIFIER, default_fg, default_bg);

        // word2
        self.set_lexer_elem_color(SCE_SQL_WORD2, keyword_fg, default_bg);

        // comment doc word
        self.set_lexer_elem_color(SCE_SQL_COMMENTDOCKEYWORD, 0xFF0000, default_bg);

        // comment doc word error
        self.set_lexer_elem_color(SCE_SQL_COMMENTDOCKEYWORDERROR, 0xFF0000, default_bg);

        // user1
        self.set_lexer_elem_color(SCE_SQL_USER1, 0xFF0000, default_bg);

        // user2
        self.set_lexer_elem_color(SCE_SQL_USER2, 0xFF0000, default_bg);

        // user3
        self.set_lexer_elem_color(SCE_SQL_USER3, 0xFF0000, default_bg);

        // user4
        self.set_lexer_elem_color(SCE_SQL_USER4, 0xFF0000, default_bg);
    }

    pub fn setup_caret(&self, width: usize, color: isize) {
        self.sci_call(SCI_SETCARETWIDTH, width, 0_isize);
        self.sci_call(SCI_SETELEMENTCOLOUR, SC_ELEMENT_CARET as usize, color);
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
