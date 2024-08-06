mod bindings;
mod constants;
pub use bindings::*;
pub use constants::*;

use std::ffi::{CStr, CString};

pub fn create_lexer(name: &str) -> Option<*mut ILexer5> {
    let c_string = CString::new(name).expect("CString::new failed");
    let c_str: &CStr = c_string.as_c_str();
    let ilexer = unsafe { CreateLexer(c_str.as_ptr()) };
    if ilexer.is_null() {
        None
    } else {
        Some(ilexer)
    }
}
