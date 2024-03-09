use nannou::image;
use nannou::image::GenericImageView;
use nannou::prelude::*;

use lib::shader_processing::model::ShaderModel;
use lib::shader_processing::pipeline::{init_shader, wgpu_render_pass};

fn main() {
    nannou::app(initialize).run();
}

struct Model {
    shader_model: ShaderModel,
}

fn initialize(app: &App) -> Model {
    // Load the image.
    let logo_path = app.assets_path().unwrap().join("prado.jpg");
    let image = image::open(logo_path).unwrap();
    let (img_w, img_h) = image.dimensions();

    let w_id = app
        .new_window()
        .size(img_w, img_h)
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();

    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let shader_model = init_shader(&image, &window, fs_desc);

    Model {
        shader_model
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    wgpu_render_pass(frame, &model.shader_model);
}