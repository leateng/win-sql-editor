mod bindings;
pub use bindings::*;
use winapi::ctypes::c_uchar;

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
use nwg::{ControlBase, ControlHandle, EventData, NwgError};
use std::ffi::{CStr, CString};
// use std::cell::RefCell;
// use std::ffi::OsStr;
// use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::str::FromStr;
use std::{isize, mem};
// use winapi;
use winapi::um::winuser::{NMHDR, WM_KEYDOWN, WM_NOTIFY};
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

        out.set_code_page(SC_CP_UTF8);
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
        out.set_key_words(
            1,
            vec![
                "select", "from", "where", "create", "delete", "update", "drop", "table", "in",
                "and", "or", "left", "right", "join", "inner", "alter", "default", "order", "by",
                "desc", "asc", "group", "having", "not", "null", "limit", "add", "column",
            ],
        );
        out.setup_color_scheme();
        out.setup_caret(2, 0xFFE75C27);

        // events, observe scintilla events on it's parent control
        let hwnd = out.handle.hwnd().unwrap();
        let edit_control = out.clone();
        let _ = nwg::bind_raw_event_handler(&parent, 0xFFFF + 100, move |_hwnd, msg, _w, l| {
            if msg == WM_NOTIFY {
                let nmhdr: &NMHDR = unsafe { &*(l as *const NMHDR) };
                let scn: &SCNotification = unsafe { &*(l as *const SCNotification) };

                // println!("nmhdr.hwndFrom==hwnd: {:?}", nmhdr.hwndFrom == hwnd);
                // println!("nmhdr.code==SCN_KEY: {:?}", nmhdr.code == SCN_KEY);

                // handle the message send from current scintilla control
                if nmhdr.hwndFrom == hwnd {
                    match nmhdr.code {
                        SCN_MODIFIED => {
                            edit_control.sci_call(SCI_COLOURISE, 0, -1);

                            if scn.linesAdded != 0 {
                                edit_control.update_line_number();
                            }
                        }

                        SCN_ZOOM => {
                            edit_control.update_line_number();
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

    pub fn sci_call(
        &self,
        i_message: ::std::os::raw::c_uint,
        w_param: uptr_t,
        l_param: sptr_t,
    ) -> isize {
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
                        fp(self.sci_direct_ptr, i_message, w_param, l_param)
                    } else {
                        0
                    }
                }
                Some(fp) => fp(self.sci_direct_ptr, i_message, w_param, l_param),
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

    pub fn set_element_colour(&self, element: u32, color: isize) {
        self.sci_call(SCI_SETELEMENTCOLOUR, element as usize, color);
    }

    pub fn style_set_fore(&self, element: u32, color: u32) {
        self.sci_call(SCI_STYLESETFORE, element as usize, color as isize);
    }

    pub fn style_set_back(&self, element: u32, color: u32) {
        self.sci_call(SCI_STYLESETBACK, element as usize, color as isize);
    }

    pub fn set_lexer_elem_color(&self, elem: u32, fore: u32, back: u32) {
        self.style_set_fore(elem, fore);
        self.style_set_back(elem, back);
    }

    pub fn set_key_words(&self, key_word_set: usize, key_words: Vec<&str>) {
        let combined_string: String = key_words.join(" ");
        let kws = CString::new(combined_string).unwrap();
        self.sci_call(SCI_SETKEYWORDS, key_word_set, kws.as_ptr() as isize);
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

        // line number
        self.set_lexer_elem_color(STYLE_LINENUMBER, comment_fg, default_bg);

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

        self.set_margins(2);
        self.set_margin_type_n(0, SC_MARGIN_NUMBER);
        self.update_line_number();
    }

    pub fn set_caret_width(&self, width: u32) {
        self.sci_call(SCI_SETCARETWIDTH, width as usize, 0_isize);
    }

    pub fn setup_caret(&self, width: u32, color: isize) {
        self.set_caret_width(width);
        self.set_element_colour(SC_ELEMENT_CARET, color);
        self.set_element_colour(SC_ELEMENT_CARET_LINE_BACK, 0xFF3E342F);
    }

    // margin functions
    pub fn set_margins(&self, margins: u32) {
        self.sci_call(SCI_SETMARGINS, margins as usize, 0_isize);
    }

    pub fn set_margin_type_n(&self, margin: u32, margin_type: u32) {
        self.sci_call(SCI_SETMARGINTYPEN, margin as usize, margin_type as isize);
    }

    pub fn set_margin_width_n(&self, margin: u32, width: u32) {
        self.sci_call(SCI_SETMARGINWIDTHN, margin as usize, width as isize);
    }

    pub fn set_margin_back_n(&self, margin: u32, colour: u32) {
        self.sci_call(SCI_SETMARGINBACKN, margin as usize, colour as isize);
    }

    pub fn get_line_count(&self) -> usize {
        self.sci_call(SCI_GETLINECOUNT, 0 as usize, 0 as isize) as usize
    }

    pub fn set_code_page(&self, code_page: u32) {
        self.sci_call(SCI_SETCODEPAGE, code_page as usize, 0 as isize);
    }

    pub fn get_selection_start(&self) -> usize {
        self.sci_call(SCI_GETSELECTIONSTART, 0 as usize, 0 as isize) as usize
    }

    pub fn get_selection_end(&self) -> usize {
        self.sci_call(SCI_GETSELECTIONEND, 0 as usize, 0 as isize) as usize
    }

    pub fn get_sel_text(&self) -> Option<String> {
        let n = self.get_selection_end() - self.get_selection_start();
        if n <= 0 {
            return None;
        }

        let mut buffer: Vec<u8> = vec![0; n + 1];

        self.sci_call(SCI_GETSELTEXT, 0 as usize, buffer.as_mut_ptr() as isize);
        let str = CStr::from_bytes_with_nul(&buffer)
            .unwrap()
            .to_str()
            .unwrap();
        Some(str.to_owned())
    }

    pub fn text_width(&self, style: u32, text: &str) -> u32 {
        let c_string = CString::new(text).expect("CString::new failed");
        self.sci_call(
            SCI_TEXTWIDTH,
            style as usize,
            c_string.as_c_str().as_ptr() as isize,
        ) as u32
    }

    pub fn update_line_number(&self) {
        let n = self.get_line_count();
        let result = format!("00{}", n);
        let width = self.text_width(STYLE_LINENUMBER, result.as_str());
        self.set_margin_width_n(0, width);
    }

    // others
    pub fn on_resize(&self) {
        println!("resize scintilla");
    }

    pub fn on_key_press(&self, event_data: &EventData) {
        match event_data {
            EventData::OnKey(key_code) => {
                if *key_code == nwg::keys::F5 {
                    let start = self.get_selection_start();
                    let end = self.get_selection_end();

                    println!("format selection [start, end]=[{:?}, {:?}]", start, end);

                    if let Some(text) = self.get_sel_text() {
                        println!("text={:?}", text);
                    }
                }
            }
            _ => (),
        }
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
