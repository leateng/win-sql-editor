#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod scintilla;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::scintilla::{register_window_class, ScintillaEdit};
use nwd::NwgUi;
use nwg::NativeUi;
// use tokio;
// use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_VISIBLE};

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (800, 600), position: (300, 300), title: "SQL Editor", flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] , OnResize: [BasicApp::on_resize])]
    window: nwg::Window,

    // #[nwg_control(text: "Heisenberg", size: (280, 25), position: (10, 10))]
    // name_edit: nwg::TextInput,

    // #[nwg_control(text: "Say my name", size: (280, 60), position: (10, 40))]
    // #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    // hello_button: nwg::Button,

    // #[nwg_control(parent: window)]
    // layout: nwg::FlexboxLayout,

    // #[nwg_control(parent: layout)]
    #[nwg_control(size: (800, 600), position: (0, 0))]
    scintilla: ScintillaEdit,
}

impl BasicApp {
    fn say_hello(&self) {
        nwg::simple_message("Hello😀", &format!("Hello😀 {}", "123"));
    }

    fn say_goodbye(&self) {
        nwg::simple_message("Goodbye", &format!("Goodbye {}", "123"));
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&self) {
        // self.scintilla.size(self.window.set_size(x, y)
        println!("window size = {:?}", self.window.size());
    }
}

#[tokio::main]
async fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI Emoji").expect("set global font family error!");
    register_window_class();

    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
