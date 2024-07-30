#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lexilla;
mod scintilla;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::scintilla::{register_window_class, ScintillaEdit};
use nwd::NwgUi;
// use nwg::Event;
use nwg::NativeUi;
// use tokio;
// use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_VISIBLE};

use nwg::stretch::{
    geometry::{Rect, Size},
    style::{Dimension as D, FlexDirection},
};

const PT_0: D = D::Points(0.0);
// const PT_2: D = D::Points(2.0);
// const PT_5: D = D::Points(5.0);
// const PT_10: D = D::Points(10.0);
//
// const FIFTY_PC: D = D::Percent(0.5);
//
// const PADDING: Rect<D> = Rect {
//     start: PT_10,
//     end: PT_10,
//     top: PT_10,
//     bottom: PT_10,
// };
const MARGIN_0: Rect<D> = Rect {
    start: PT_0,
    end: PT_0,
    top: PT_0,
    bottom: PT_0,
};

// const MARGIN_2: Rect<D> = Rect {
//     start: PT_2,
//     end: PT_2,
//     top: PT_2,
//     bottom: PT_2,
// };
//
// const MARGIN_5: Rect<D> = Rect {
//     start: PT_5,
//     end: PT_5,
//     top: PT_5,
//     bottom: PT_5,
// };

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (800, 600), position: (300, 300), title: "SQL Editor", flags: "MAIN_WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] , OnResize: [BasicApp::on_resize(SELF)])]
    window: nwg::Window,

    // #[nwg_layout(parent: window, spacing: 2, min_size: [150, 140])]
    // grid: nwg::GridLayout,
    // #[nwg_control(text: "Heisenberg", size: (280, 25), position: (10, 10))]
    // name_edit: nwg::TextInput,

    // #[nwg_control(text: "Say my name", size: (280, 60), position: (10, 40))]
    // #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    // hello_button: nwg::Button,
    #[nwg_layout(parent: window, flex_direction: FlexDirection::Row)]
    layout: nwg::FlexboxLayout,

    #[nwg_control()]
    #[nwg_events(OnResize:[ScintillaEdit::on_resize(CTRL)], OnInit: [ScintillaEdit::setup_event(CTRL)])]
    #[nwg_layout_item(layout: layout,
        margin: MARGIN_0,
        max_size: Size { width: D::Percent(1.0), height: D::Auto},
        size: Size { width: D::Percent(1.0), height: D::Auto }
    )]
    scintilla: ScintillaEdit,
}

impl BasicApp {
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
