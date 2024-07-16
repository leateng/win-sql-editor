#![windows_subsystem = "windows"]

use sqlx::postgres::PgPoolOptions;
use tokio;
use w::co::WS;
use winsafe::{self as w, gui, prelude::*, HINSTANCE};

mod scintilla;

#[derive(Clone)]
struct MyWindow {
    wnd: gui::WindowMain,
    btn_hello: gui::Button,
    edit: scintilla::ScintillaEdit,
}

impl MyWindow {
    fn new() -> Self {
        let wnd = gui::WindowMain::new(gui::WindowMainOpts {
            title: "Win SQL".into(),
            size: (800, 650),
            style: gui::WindowMainOpts::default().style
                | WS::MINIMIZEBOX
                | WS::MAXIMIZEBOX
                | WS::SIZEBOX, // add a minimize button
            ..Default::default()
        });

        let btn_hello = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "&Click me".into(),
                position: (20, 20),
                ..Default::default()
            },
        );

        let edit = scintilla::ScintillaEdit::new(&wnd, (20, 50), (500, 500));

        let new_self = Self {
            wnd,
            btn_hello,
            edit,
        };
        new_self.events();
        new_self
    }

    fn events(&self) {
        let self2 = self.clone();
        self2.btn_hello.on().bn_clicked(move || {
            println!("button click");

            let h_instance = HINSTANCE::GetModuleHandle(None).unwrap();
            println!("register_classes hInstance ={:?}", h_instance);

            Ok(())
        });

        let self2 = self.clone();
    }
}

#[tokio::main]
async fn main() {
    let my_window = MyWindow::new();
    scintilla::register_classes();
    if let Err(e) = my_window.wnd.run_main(None) {
        eprintln!("{}", e);
    }
}
