use nannou::prelude::*;
use nannou_egui::{Egui, egui};

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

struct Model {
    texture: wgpu::Texture,
    surface_size: [u32; 2],
    egui: Egui,
    settings: Settings,
}

struct Settings {
    resolution: u32,
    scale: f32,
    rotation: f32,
    color: Srgb<u8>,
    position: Vec2,
}

fn model(_app: &App) -> Model {
    let surface_size = [2048, 1024];
    let window_id = _app
        .new_window()
        .size(surface_size[0], surface_size[1])
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = _app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    let assets = _app.assets_path().unwrap();
    let img_path = assets.join("imagen.jpg");
    let texture = wgpu::Texture::from_path(_app, img_path).unwrap();
    Model {
        texture,
        surface_size,
        egui,
        settings: Settings {
            resolution: 10,
            scale: 200.0,
            rotation: 0.0,
            color: WHITE,
            position: vec2(0.0, 0.0),
        },
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let egui = &mut _model.egui;
    let settings = &mut _model.settings;

    egui.set_elapsed_time(_update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        // Resolution slider
        ui.label("Resolution:");
        ui.add(egui::Slider::new(&mut settings.resolution, 1..=40));

        // Scale slider
        ui.label("Scale:");
        ui.add(egui::Slider::new(&mut settings.scale, 0.0..=1000.0));

        // Rotation slider
        ui.label("Rotation:");
        ui.add(egui::Slider::new(&mut settings.rotation, 0.0..=360.0));

        // Random color button
        let clicked = ui.button("Random color").clicked();

        if clicked {
            settings.color = rgb(random(), random(), random());
        }
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn view(_app: &App, _model: &Model, frame: Frame) {

    let draw = _app.draw();
    draw.background().color(BLACK);

    let size = _model.texture.size();

    let scale_to_fit: [f32; 2] = if size[0] > size[1] {
        let new_ratio: f32 = _model.surface_size[0].to_f32().unwrap() / size[0].to_f32().unwrap();
        let new_height: f32 = new_ratio * size[1].to_f32().unwrap();
        [_model.surface_size[0].to_f32().unwrap(), new_height]
    } else {
        let new_ratio: f32 = _model.surface_size[1].to_f32().unwrap() / size[1].to_f32().unwrap();
        let new_width: f32 = new_ratio * size[1].to_f32().unwrap();
        [_model.surface_size[1].to_f32().unwrap(), new_width]
    };

    draw.texture(&_model.texture).wh(scale_to_fit.into());

    let settings = &_model.settings;
    let rotation_radians = deg_to_rad(settings.rotation);
    draw.ellipse()
        .resolution(settings.resolution as f32)
        .xy(settings.position)
        .color(settings.color)
        .rotate(-rotation_radians)
        .radius(settings.scale);


    draw.to_frame(_app, &frame).unwrap();
    _model.egui.draw_to_frame(&frame).unwrap();
}