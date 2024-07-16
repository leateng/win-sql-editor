use std::any::Any;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::sync::Arc;

use winsafe::co;
use winsafe::decl::*;
use winsafe::gui::{events::*, iterators::*, privs::*, *};
use winsafe::msg::*;
use winsafe::prelude::*;

struct Obj {
    // actual fields of Edit
    base: BaseNativeControl,
    events: EditEvents,
    _pin: PhantomPinned,
}

//------------------------------------------------------------------------------

/// Native
/// [edit](https://learn.microsoft.com/en-us/windows/win32/controls/about-edit-controls)
/// control.
#[derive(Clone)]
pub struct Edit(Pin<Arc<Obj>>);

unsafe impl Send for Edit {}

impl AsRef<BaseNativeControl> for Edit {
    fn as_ref(&self) -> &BaseNativeControl {
        &self.0.base
    }
}

impl GuiWindow for Edit {
    fn hwnd(&self) -> &HWND {
        self.0.base.hwnd()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GuiWindowText for Edit {}

impl GuiChild for Edit {
    fn ctrl_id(&self) -> u16 {
        self.0.base.ctrl_id()
    }
}

impl GuiChildFocus for Edit {}

impl GuiNativeControl for Edit {}

impl GuiNativeControlEvents<EditEvents> for Edit {
    fn on(&self) -> &EditEvents {
        if *self.hwnd() != HWND::NULL {
            panic!("Cannot add events after the control creation.");
        } else if *self.0.base.parent().hwnd() != HWND::NULL {
            panic!("Cannot add events after the parent window creation.");
        }
        &self.0.events
    }
}

impl Edit {
    /// Instantiates a new `Edit` object, to be created on the parent window
    /// with
    /// [`HWND::CreateWindowEx`](crate::prelude::user_Hwnd::CreateWindowEx).
    ///
    /// # Panics
    ///
    /// Panics if the parent window was already created – that is, you cannot
    /// dynamically create an `Edit` in an event closure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use winsafe::{self as w, prelude::*, gui};
    ///
    /// let wnd: gui::WindowMain; // initialized somewhere
    /// # let wnd = gui::WindowMain::new(gui::WindowMainOpts::default());
    ///
    /// let txt = gui::Edit::new(
    ///     &wnd,
    ///     gui::EditOpts {
    ///         position: (10, 10),
    ///         width: 120,
    ///         ..Default::default()
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn new(parent: &impl GuiParent, opts: EditOpts) -> Self {
        let opts = auto_ctrl_id_if_zero(opts);
        let ctrl_id = opts.ctrl_id;

        let new_self = Self(Arc::pin(Obj {
            base: BaseNativeControl::new(parent, ctrl_id),
            events: EditEvents::new(parent, ctrl_id),
            _pin: PhantomPinned,
        }));

        let self2 = new_self.clone();
        parent
            .as_ref()
            .before_user_on()
            .wm_create_or_initdialog(move |_, _| {
                self2.create(OptsResz::Wnd(&opts))?;
                Ok(WmRet::NotHandled)
            });

        new_self
    }

    /// Instantiates a new `Edit` object, to be loaded from a dialog resource
    /// with [`HWND::GetDlgItem`](crate::prelude::user_Hwnd::GetDlgItem).
    ///
    /// # Panics
    ///
    /// Panics if the parent dialog was already created – that is, you cannot
    /// dynamically create an `Edit` in an event closure.
    #[must_use]
    pub fn new_dlg(parent: &impl GuiParent, ctrl_id: u16, resize_behavior: (Horz, Vert)) -> Self {
        let new_self = Self(Arc::pin(Obj {
            base: BaseNativeControl::new(parent, ctrl_id),
            events: EditEvents::new(parent, ctrl_id),
            _pin: PhantomPinned,
        }));

        let self2 = new_self.clone();
        parent
            .as_ref()
            .before_user_on()
            .wm(co::WM::INITDIALOG, move |_, _| {
                self2.create(OptsResz::Dlg(resize_behavior))?;
                Ok(WmRet::NotHandled)
            });

        new_self
    }

    fn create(&self, opts_resz: OptsResz<&EditOpts>) -> SysResult<()> {
        match opts_resz {
            OptsResz::Wnd(opts) => {
                let mut pos = POINT::new(opts.position.0, opts.position.1);
                let mut sz = SIZE::new(opts.width as _, opts.height as _);
                multiply_dpi_or_dtu(self.0.base.parent(), Some(&mut pos), Some(&mut sz))?;

                self.0.base.create_window(
                    "Scintilla",
                    Some(&opts.text),
                    pos,
                    sz,
                    opts.window_ex_style,
                    opts.window_style | opts.edit_style.into(),
                )?;

                unsafe {
                    self.hwnd().SendMessage(wm::SetFont {
                        hfont: ui_font(),
                        redraw: true,
                    });
                }
            }
            OptsResz::Dlg(_) => self.0.base.create_dlg()?,
        }

        self.0
            .base
            .parent()
            .add_to_layout_arranger(self.hwnd(), opts_resz.resize_behavior())
    }

    /// Hides any balloon tip by sending an
    /// [`em::HideBalloonTip`](crate::msg::em::HideBalloonTip) message.
    pub fn hide_balloon_tip(&self) {
        unsafe { self.hwnd().SendMessage(em::HideBalloonTip {}) }.unwrap();
    }

    /// Returns an iterator over the lines in the Edit.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use winsafe::{self as w, prelude::*, gui};
    ///
    /// let my_edit: gui::Edit; // initialized somewhere
    /// # let wnd = gui::WindowMain::new(gui::WindowMainOpts::default());
    /// # let my_edit = gui::Edit::new(&wnd, gui::EditOpts::default());
    ///
    /// for line in my_edit.iter_lines() {
    ///     println!("{}", line);
    /// }
    /// # w::SysResult::Ok(())
    /// ```
    #[must_use]
    pub fn iter_lines<'a>(&'a self) -> impl Iterator<Item = String> + 'a {
        EditLineIter::new(self)
    }

    /// Limits the number of characters that can be type by sending an
    /// [`em::SetLimitText`](crate::msg::em::SetLimitText) message.
    pub fn limit_text(&self, max_chars: Option<u32>) {
        unsafe {
            self.hwnd().SendMessage(em::SetLimitText { max_chars });
        }
    }

    /// Returns the number of lines by sending an
    /// [`em::GetLineCount`](crate::msg::em::GetLineCount) message.
    #[must_use]
    pub fn line_count(&self) -> u32 {
        unsafe { self.hwnd().SendMessage(em::GetLineCount {}) }
    }

    /// Sets the font to the `Edit` by sending an
    /// [`wm::SetFont`](crate::msg::wm::SetFont) message.
    ///
    /// Note that the font must remain alive while being used in the control.
    pub fn set_font(&self, font: &HFONT) {
        unsafe {
            self.hwnd().SendMessage(wm::SetFont {
                hfont: font.raw_copy(),
                redraw: true,
            });
        }
    }

    /// Sets the selection range of the text by sending an
    /// [`em::SetSel`](crate::msg::em::SetSel) message.
    ///
    /// # Examples
    ///
    /// Selecting all text in the control:
    ///
    /// ```no_run
    /// use winsafe::{self as w, prelude::*, gui};
    ///
    /// let my_edit: gui::Edit; // initialized somewhere
    /// # let wnd = gui::WindowMain::new(gui::WindowMainOpts::default());
    /// # let my_edit = gui::Edit::new(&wnd, gui::EditOpts::default());
    ///
    /// my_edit.set_selection(0, -1);
    /// ```
    ///
    /// Clearing the selection:
    ///
    /// ```no_run
    /// use winsafe::gui;
    ///
    /// let my_edit: gui::Edit; // initialized somewhere
    /// # let wnd = gui::WindowMain::new(gui::WindowMainOpts::default());
    /// # let my_edit = gui::Edit::new(&wnd, gui::EditOpts::default());
    ///
    /// my_edit.set_selection(-1, -1);
    /// ```
    pub fn set_selection(&self, start: i32, end: i32) {
        unsafe {
            self.hwnd().SendMessage(em::SetSel { start, end });
        }
    }

    /// Displays a balloon tip by sending an
    /// [`em::ShowBalloonTip`](crate::msg::em::ShowBalloonTip) message.
    pub fn show_ballon_tip(&self, title: &str, text: &str, icon: co::TTI) {
        let mut title16 = WString::from_str(title);
        let mut text16 = WString::from_str(text);

        let mut info = EDITBALLOONTIP::default();
        info.set_pszTitle(Some(&mut title16));
        info.set_pszText(Some(&mut text16));
        info.ttiIcon = icon;

        unsafe { self.hwnd().SendMessage(em::ShowBalloonTip { info: &info }) }.unwrap();
    }
}

//------------------------------------------------------------------------------

/// Options to create an [`Edit`](crate::gui::Edit) programmatically with
/// [`Edit::new`](crate::gui::Edit::new).
pub struct EditOpts {
    /// Text of the control to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// Defaults to empty string.
    pub text: String,
    /// Left and top position coordinates of control within parent's client
    /// area, to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// If the parent window is a dialog, the values are in Dialog Template
    /// Units; otherwise in pixels, which will be multiplied to match current
    /// system DPI.
    ///
    /// Defaults to `(0, 0)`.
    pub position: (i32, i32),
    /// Control width to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// If the parent window is a dialog, the value is in Dialog Template Units;
    /// otherwise in pixels, which will be multiplied to match current system
    /// DPI.
    ///
    /// Defaults to `100`.
    pub width: u32,
    /// Control height to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// If the parent window is a dialog, the value is in Dialog Template Units;
    /// otherwise in pixels, which will be multiplied to match current system
    /// DPI.
    ///
    /// Defaults to `23`.
    ///
    /// **Note:** You should change the default height only in a multi-line
    /// edit, otherwise it will look off.
    pub height: u32,
    /// Edit styles to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// Defaults to `ES::AUTOHSCROLL | ES::NOHIDESEL`.
    ///
    /// Suggestions:
    /// * add `ES::PASSWORD` for a password input;
    /// * add `ES::NUMBER` to accept only numbers;
    /// * replace with `ES::MULTILINE | ES::WANTRETURN | ES::AUTOVSCROLL | ES::NOHIDESEL` for a multi-line edit.
    pub edit_style: co::ES,
    /// Window styles to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// Defaults to `WS::CHILD | WS::VISIBLE | WS::TABSTOP | WS::GROUP`.
    pub window_style: co::WS,
    /// Extended window styles to be
    /// [created](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw).
    ///
    /// Defaults to `WS_EX::LEFT | WS_EX::CLIENTEDGE`.
    pub window_ex_style: co::WS_EX,

    /// The control ID.
    ///
    /// Defaults to an auto-generated ID.
    pub ctrl_id: u16,
    /// Horizontal and vertical behavior of the control when the parent window
    /// is resized.
    ///
    /// **Note:** You should use `Vert::Resize` only in a multi-line edit.
    ///
    /// Defaults to `(gui::Horz::None, gui::Vert::None)`.
    pub resize_behavior: (Horz, Vert),
}

impl Default for EditOpts {
    fn default() -> Self {
        Self {
            text: "".to_owned(),
            position: (0, 0),
            width: 100,
            height: 23,
            edit_style: co::ES::AUTOHSCROLL | co::ES::NOHIDESEL,
            window_style: co::WS::CHILD | co::WS::VISIBLE | co::WS::TABSTOP | co::WS::GROUP,
            window_ex_style: co::WS_EX::LEFT | co::WS_EX::CLIENTEDGE,
            ctrl_id: 0,
            resize_behavior: (Horz::None, Vert::None),
        }
    }
}

impl ResizeBehavior for &EditOpts {
    fn resize_behavior(&self) -> (Horz, Vert) {
        self.resize_behavior
    }
}

impl AutoCtrlId for EditOpts {
    fn ctrl_id_mut(&mut self) -> &mut u16 {
        &mut self.ctrl_id
    }
}
