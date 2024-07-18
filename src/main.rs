// #![windows_subsystem = "windows"]
mod scintilla;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::scintilla::ScintillaEdit;
use nwd::NwgUi;
use nwg::NativeUi;
use tokio;

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (300, 115), position: (300, 300), title: "Basic example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] )]
    window: nwg::Window,

    #[nwg_control(text: "Heisenberg", size: (280, 25), position: (10, 10))]
    name_edit: nwg::TextInput,

    // #[nwg_control(text: "Say my name", size: (280, 60), position: (10, 40))]
    // #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    // hello_button: nwg::Button,

    // #[nwg_control(parent: window)]
    // layout: nwg::FlexboxLayout,

    // #[nwg_control(parent: layout)]
    #[nwg_control(size: (280, 60), position: (10, 40))]
    scintilla: ScintillaEdit,
}

impl BasicApp {
    fn say_hello(&self) {
        nwg::simple_message("Hello😀", &format!("Hello😀 {}", self.name_edit.text()));
    }

    fn say_goodbye(&self) {
        nwg::simple_message("Goodbye", &format!("Goodbye {}", self.name_edit.text()));
        nwg::stop_thread_dispatch();
    }
}

#[tokio::main]
async fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI Emoji").expect("set global font family error!");
    ScintillaEdit::regist();

    let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
