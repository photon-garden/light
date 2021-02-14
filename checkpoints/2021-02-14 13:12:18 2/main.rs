mod light;
#[macro_use]
mod checkpoint;
mod helpers;
use helpers::Helpers;

use nannou::prelude::*;

fn main() {
    nannou::app(start).simple_window(view).run();
}

struct Model {}

fn start(app: &App) -> Model {
    app.set_loop_mode(LoopMode::loop_ntimes(3));

    Model {}
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let mut current_checkpoint = checkpoint::save(frame.nth());
    let helpers = Helpers::new(app);

    let image_path = current_checkpoint.image_path();
    app.main_window().capture_frame(image_path);

    light::draw(app, &frame, &mut current_checkpoint.rand, &helpers);
}