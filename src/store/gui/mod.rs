use crate::db::sqlite::store as dbstore;
use crate::store;
use fl::dialog;
use fltk as fl;
use fltk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct APP {
    pub lib: Rc<RefCell<store::WordLibrary<store::World, sqlite::Error, dbstore::Tansaction>>>,
    // pub output: fl::output::MultilineOutput,
    pub tabs: fl::group::Pack,
}

struct Current {
    but: Option<fl::button::Button>,
    index: fl::browser::SelectBrowser,
    // lib: Rc<RefCell<store::WordLibrary<store::World, sqlite::Error, dbstore::Tansaction>>>,
    store_key: String,
}

impl Current {
    fn set_button(&mut self, but: fl::button::Button) {
        self.but = Some(but)
    }

    fn set_store_key(&mut self, key: String) {
        self.store_key = key
    }
}

impl APP {
    fn meun(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        tabs: fl::group::Pack,
        // output: fl::output::MultilineOutput,
        current: Rc<RefCell<Current>>,
        lib: Rc<RefCell<store::WordLibrary<store::World, sqlite::Error, dbstore::Tansaction>>>,
    ) -> fl::menu::SysMenuBar {
        let mut menu = fltk::menu::SysMenuBar::new(x, y, width, height, None);

        let cur = current;

        let mut fc =
            fl::dialog::NativeFileChooser::new(fl::dialog::NativeFileChooserType::BrowseMultiFile);
        menu.add(
            "Store/Open",
            fl::enums::Shortcut::Ctrl | fl::enums::Shortcut::from_char('t'),
            fl::menu::MenuFlag::Normal,
            {
                let lib = lib.clone();
                let mut tabs = tabs.clone();
                // let output = output.clone();
                tabs.set_align(fl::enums::Align::Left);

                move |_| {
                    fc.show();
                    let mut fs = fc.filenames();
                    while let Some(i) = fs.pop() {
                        let key = i.clone().into_os_string().into_string().unwrap();
                        println!("open {}", key);
                        match dbstore::Sqlite::new(key.clone()) {
                            Ok(v) => {
                                let index = &mut lib.borrow_mut().stores;
                                index.insert(key.clone(), Box::new(v));
                            }
                            Err(err) => {
                                // TODO add alter window
                                println!("warn0 {}", err);
                                return;
                            }
                        }

                        // let but = APP::tab_buttom(&key, output.clone(), cur.clone(), lib.clone());
                        let mut but = APP::tab_buttom(&key, cur.clone(), lib.clone());
                        tabs.add(&but);
                        but.do_callback();
                    }

                    println!("redraw tab");
                    // tab redraw is useless
                    tabs.parent().unwrap().redraw();
                }
            },
        );
        menu
    }

    fn file_tab(x: i32, y: i32, width: i32, height: i32) -> fl::group::Pack {
        let mut tabs = fl::group::Pack::new(x, y, width, height, "tabs");
        tabs.set_spacing(3);
        tabs.set_type(fl::group::PackType::Horizontal);
        tabs.end();
        tabs.show();
        tabs
    }

    fn browser() -> fl::browser::SelectBrowser {
        fl::browser::SelectBrowser::new(0, 0, 0, 0, None)
    }

    fn tab_buttom(
        key: &str,
        // mut output: fl::output::MultilineOutput,
        current: Rc<RefCell<Current>>,
        lib: Rc<RefCell<store::WordLibrary<store::World, sqlite::Error, dbstore::Tansaction>>>,
    ) -> fl::button::Button {
        // let key_width = key.len() as i32;
        let mut but = fl::button::Button::new(0, 0, 0, 0, None);
        // fl::enums::Font::

        but.set_label(&key);
        but.set_align(fl::enums::Align::Left | fl::enums::Align::Inside);
        let (width, height) = but.measure_label();
        but.resize(0, 0, width + 7, height);
        but.set_frame(fl::enums::FrameType::PlasticRoundUpBox);
        but.set_color(fl::enums::Color::White);

        but.clear_visible_focus();
        but.set_callback({
            // let mut output = output.clone();
            let mut but = but.clone();
            move |b| {
                let key = b.label();
                match lib.borrow().list(0, &key) {
                    Ok(ws) => {
                        let mut cur = current.borrow_mut();
                        if let Some(mut cur) = cur.but.clone() {
                            cur.set_frame(fl::enums::FrameType::PlasticRoundUpBox);
                            cur.set_color(fl::enums::Color::White);
                            cur.redraw();
                        }

                        but.set_frame(fl::enums::FrameType::PlasticRoundDownBox);
                        but.set_color(fl::enums::Color::Blue);
                        // let t = current.as_ref().borrow().but;
                        // current.get_mut().but = Some(but);
                        cur.set_button(but.clone());
                        cur.set_store_key(key);

                        // output.set_value("");

                        let mut browser = cur.index.clone();
                        browser.clear();

                        for i in &ws {
                            let txt = format!("{}", i.name);
                            browser.add(&txt);

                            // if let Err(err) = output.append(&txt) {
                            //     println!("warn1 {}", err);
                            //     return;
                            // }
                        }
                        browser.show()
                    }
                    Err(err) => {
                        println!("warn2 {}", err);
                        return;
                    }
                }
            }
        });
        but
    }

    pub fn new<T: Into<Option<&'static str>>>(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        title: T,
    ) -> APP {
        let mut app = fl::app::App::default().with_scheme(fl::app::AppScheme::Gtk);

        let menu_height = 40;
        let mut win = fl::window::Window::new(x, y, width, height, title);

        let lib = Rc::new(RefCell::new(store::WordLibrary {
            stores: HashMap::new(),
        }));

        let tab_height = 40;
        let tabs = APP::file_tab(x, y + menu_height, width, tab_height);

        let index_width = 100;
        let mut output = fl::text::TextEditor::new(
            x + index_width,
            y + tab_height + menu_height,
            width - index_width,
            height - menu_height - tab_height,
            None,
        );

        // output.visible_focus(false);
        // output.set_readonly(true);
        let mut buffer = fl::text::TextBuffer::default();
        output.set_buffer(buffer.clone());
        output.show();

        let mut index = fl::browser::SelectBrowser::new(
            x,
            y + tab_height + menu_height,
            index_width,
            height - menu_height - tab_height,
            None,
        );
        index.show();
        // index.set_frame(fl::enums::FrameType::DownFrame);

        let cur = Rc::new(RefCell::new(Current {
            but: None,
            index: index.clone(),
            // lib: lib.clone(),
            store_key: String::from(""),
        }));

        index.set_selection_color(fl::enums::Color::Blue);

        index.set_visible_focus();
        // index.set_frame(fl::enums::FrameType::UpFrame);
        index.show();

        index.set_callback({
            let lib = lib.clone();
            let cur = cur.clone();
            let mut output = output.clone();
            let mut win = win.clone();
            let mut buffer = buffer.clone();
            move |i| {
                if let Some(word) = i.selected_text() {
                    let word = word.trim_end();
                    match lib.borrow().get(word, &cur.borrow().store_key) {
                        Ok(word) => {
                            // output.set_value(&word.message);
                            // output.clear();
                            // output.add(&word.message);

                            buffer.set_text(&word.message);
                            // i.set_frame(fl::enums::FrameType::GtkDownFrame);
                            // win.redraw();
                        }
                        Err(err) => {
                            println!("get word for output fail, word: {}, err msg: {}", word, err);
                        }
                    }
                }
            }
        });

        let mut menu = APP::meun(
            x,
            y,
            width,
            menu_height,
            tabs.clone(),
            // output.clone(),
            cur.clone(),
            lib.clone(),
        );

        menu.add(
            "Store/Save",
            fl::enums::Shortcut::Ctrl | fl::enums::Shortcut::from_char('s'),
            fl::menu::MenuFlag::Normal,
            {
                let buffer = buffer.clone();
                let lib = lib.clone();
                let cur = cur.clone();
                let index = index.clone();
                move |_| {
                    if let Some(name) = index.selected_text() {
                        let word = store::World {
                            message: buffer.text(),
                            name: name,
                        };

                        if let Err(err) = lib.borrow().update(&word, &cur.borrow().store_key) {
                            match err {
                                store::InternalError::NotFound => {
                                    if let Err(err) =
                                        lib.borrow().insert(&word, &cur.borrow().store_key)
                                    {
                                        println!("save error: {}", err);
                                        return;
                                    }
                                }
                                err => {
                                    println!("save error: {}", err);
                                    return;
                                }
                            }
                        }
                    }
                }
            },
        );

        menu.add(
            "Store/NewDoc",
            fl::enums::Shortcut::Ctrl | fl::enums::Shortcut::from_char('n'),
            fl::menu::MenuFlag::Normal,
            {
                let mut buffer = buffer.clone();
                let lib = lib.clone();
                let cur = cur.clone();
                move |_| {
                    if let Some(name) = fl::dialog::input_default("word key", "") {
                        let word = store::World {
                            message: String::from(""),
                            name: name,
                        };

                        if let Err(err) = lib.borrow().insert(&word, &cur.borrow().store_key) {
                            println!("insert word error: {}", err);
                            return;
                        }

                        buffer.set_text(&word.message);
                        index.add(&word.name);
                        index.select(index.size());
                        index.bottom_line(index.size());
                    }
                }
            },
        );

        // let output = output.below_of(&tabs, 6);
        let tabs = tabs.below_of(&menu, 1);

        win.end();
        win.show();
        app.run().unwrap();
        APP { lib, tabs }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_db() {
        let path = r#"d://test.sql"#;
        dbstore::Sqlite::new(path.to_owned()).unwrap();
    }
}
