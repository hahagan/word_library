use crate::db::sqlite::store as dbstore;
use crate::store;
use fl::enums::Align;
use fltk as fl;
use fltk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct APP {
    pub lib: Rc<RefCell<store::WordLibrary<store::World, sqlite::Error, dbstore::Tansaction>>>,
    pub output: fl::output::MultilineOutput,
    pub tabs: fl::group::Pack,
}

impl APP {
    pub fn new<T: Into<Option<&'static str>>>(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        title: T,
    ) -> APP {
        let mut app = fl::app::App::default().with_scheme(fl::app::AppScheme::Gtk);

        let menu_height = 40;
        let mut win = fl::window::Window::new(x, y, width, height, "message");

        let mut menu = fltk::menu::SysMenuBar::new(x, y, width, menu_height, title);
        // menu.add("New/folder", , flag, cb);
        let lib = Rc::new(RefCell::new(store::WordLibrary {
            stores: HashMap::new(),
        }));

        let tab_height = 40;
        let mut tabs = fl::group::Pack::new(x, y + menu_height, width, tab_height, "tabs");
        tabs.set_spacing(3);
        tabs.set_type(fl::group::PackType::Horizontal);
        tabs.end();
        tabs.show();

        let tabs = tabs.below_of(&menu, 6);

        let output = fl::output::MultilineOutput::new(
            x,
            y + tab_height + menu_height,
            width,
            height - menu_height - tab_height,
            "words",
        );

        let mut output = output.below_of(&tabs, 6);
        output.visible_focus(false);
        output.set_readonly(true);
        output.show();

        let mut fc =
            fl::dialog::NativeFileChooser::new(fl::dialog::NativeFileChooserType::BrowseMultiFile);
        menu.add(
            "New/folder",
            fl::enums::Shortcut::None,
            fl::menu::MenuFlag::Normal,
            {
                let lib = lib.clone();
                let mut tabs = tabs.clone();
                let mut win = win.clone();
                let output = output.clone();
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

                        let key_width = key.len() as i32;
                        let mut but = fl::button::Button::new(0, 0, key_width * 10, 40, "");
                        but.set_label(&key);
                        but.set_align(fl::enums::Align::Left | fl::enums::Align::Inside);
                        but.set_frame(fl::enums::FrameType::UpFrame);
                        // but.clear_visible_focus();
                        but.set_callback({
                            let mut output = output.clone();
                            let lib = lib.clone();
                            move |b| {
                                let key = b.label();
                                match lib.borrow().list(0, &key) {
                                    Ok(ws) => {
                                        output.set_value("");
                                        for i in &ws {
                                            let txt = format!("{}-{}\n", i.name, i.message);

                                            if let Err(err) = output.append(&txt) {
                                                println!("warn1 {}", err);
                                                return;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!("warn2 {}", err);
                                        return;
                                    }
                                }
                            }
                        });
                        tabs.add(&but);
                        tabs.redraw();
                        tabs.show();
                        win.redraw();
                    }

                    println!("redraw tab");
                    tabs.redraw();
                    // tabs.show();
                    // win.redraw();
                    app.redraw();
                }
            },
        );

        win.end();
        win.show();
        app.run().unwrap();
        APP { lib, output, tabs }
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
