#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lexilla;
mod scintilla;
mod sql_formatter;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::scintilla::{register_window_class, ScintillaEdit};
// use lazy_static::lazy_static;
use nwd::NwgUi;
// use nwg::EmbedResource;
// use nwg::Event;
use anyhow::Result;
use nwg::NativeUi;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
// use tokio;
// use winapi::um::winuser::{WS_CHILD, WS_EX_CLIENTEDGE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_VISIBLE};

// lazy_static! {
//     static ref EMBED: EmbedResource = nwg::EmbedResource::load(None).unwrap();
// }

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
    db_pool: RefCell<Option<PgPool>>,

    #[nwg_resource]
    embed: nwg::EmbedResource,

    #[nwg_control(
        maximized: true,
        position: (0, 0),
        title: "Data Fox",
        flags: "MAIN_WINDOW|VISIBLE",
    )]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] , OnResize: [BasicApp::on_resize(SELF)], OnInit: [BasicApp::on_init(SELF)])]
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
    #[nwg_events(
        OnResize:[ScintillaEdit::on_resize(CTRL)],
        OnInit: [ScintillaEdit::setup_event(CTRL)],
        OnKeyPress: [ScintillaEdit::on_key_press(CTRL, EVT_DATA)]
    )]
    #[nwg_layout_item(layout: layout,
        margin: MARGIN_0,
        max_size: Size { width: D::Percent(1.0), height: D::Auto},
        size: Size { width: D::Percent(1.0), height: D::Auto }
    )]
    scintilla: ScintillaEdit,
}

impl BasicApp {
    fn on_init(&self) {
        // self.center_window();
        // let em = &self.embed;
        // let icon = em.icon_str("MAINICON", None);
        // if icon == None {
        //     println!("icon is None")
        // }

        // self.window.set_icon(em.icon_str("MAINICON", None).as_ref());
    }

    fn say_goodbye(&self) {
        // nwg::simple_message("Goodbye", &format!("Goodbye {}", "123"));
        nwg::stop_thread_dispatch();
    }

    fn on_resize(&self) {
        // self.scintilla.size(self.window.set_size(x, y)
        println!("window size = {:?}", self.window.size());
    }

    fn center_window(&self) {
        let screen_width = nwg::Monitor::width();
        let screen_height = nwg::Monitor::height();

        let (win_width, win_height) = self.window.size();

        let x = (screen_width - win_width as i32) / 2;
        let y = (screen_height - win_height as i32) / 2;

        self.window.set_position(x, y);
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI Emoji").expect("set global font family error!");
    register_window_class();

    let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");

    let db_path = "sqlite:my_database.db?mode=rwc";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await?;

    app.db_pool.borrow_mut().replace(pool);

    nwg::dispatch_thread_events();

    Ok(())
}
