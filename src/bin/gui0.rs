use fl::prelude::{WidgetBase, WidgetExt};
use fltk as fl;
use word_library::store::gui;

fn main() {
    // let app = fl::app::App::default().with_scheme(fl::app::AppScheme::Gtk);
    let x = 0;
    let y = 0;
    let width = 1000;
    let height = 800;
    // let mut win = fl::window::Window::new(x, y, width, height, "message");
    let myapp = gui::APP::new(x, y, width, height, "word_library");
    // win.show();
    // app.run().unwrap();
}
