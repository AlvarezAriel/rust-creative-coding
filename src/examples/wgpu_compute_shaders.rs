//! A small GPU compute shader demonstration.
//!
//! Here we use a compute shader to calculate the amplitude of `OSCILLATOR_COUNT` number of
//! oscillators. The oscillator amplitudes are then laid out across the screen using rectangles
//! with a gray value equal to the amplitude. Real-time interaction is demonstrated by providing
//! access to time, frequency (mouse `x`) and the number of oscillators via uniform data.

use std::cell::Ref;

use nannou::image;
use nannou::image::DynamicImage;
use nannou::prelude::*;
use nannou::wgpu::{BufferInitDescriptor, Device};
use nannou_egui::{Egui, egui};
use nannou_egui::egui_wgpu::wgpu::TextureView;

use lib::shader_processing::model::{QUAD, Vert};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    compute: Compute,
    render: Render,
    gui: Gui,
}

struct Gui {
    egui: Egui,
    settings: Settings,
}

struct Settings {
    resolution: u32,
    scale: f32,
    accentuate: f32,
    color: Srgb<u8>,
    position: Vec2,
}

struct Compute {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
}

struct Render {
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniforms {
    time: f32,
    accentuate: f32,
}

fn model(app: &App) -> Model {
    let w_id = app.new_window()
        .size(1024, 1024)
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    
    let window = app.window(w_id).unwrap();
    let device = window.device();

    // This texture is the input to our whole workflow, it will be processed in the compute shader
    // and the result from that
    let texture_path_buffer = app.assets_path().unwrap().join("imagen.jpg");
    let image = image::open(texture_path_buffer).unwrap();
    let texture = wgpu::Texture::from_image(&window, &image);
    let texture_view = texture.view().build();

    // This texture will be the compute shader's output and the fragment shader's input,
    // allowing us to render the compute shader's result onto the Window.
    let storage_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: texture.width(),
            height: texture.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let storage_texture_view = storage_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let compute = build_compute_pipeline(app, &window, device, &texture_view, &storage_texture_view);
    let render = build_render_pipeline(&window, device, &storage_texture_view);
    let gui = build_gui_state(&window);
    
    Model {
        compute,
        render,
        gui,
    }
}

fn build_gui_state(window: &Ref<Window>) -> Gui {
    let egui = Egui::from_window(&window);
    let gui = Gui {
        egui,
        settings: Settings {
            resolution: 10,
            scale: 200.0,
            accentuate: 0.0,
            color: WHITE,
            position: vec2(0.0, 0.0),
        }
    };
    gui
}

fn build_compute_pipeline(app: &App, window: &Ref<Window>, device: &Device, texture_view: &TextureView, storage_texture_view: &TextureView) -> Compute {
// Create the compute shader module.
    let cs_desc = wgpu::include_wgsl!("shaders/cs.wgsl");
    let cs_mod = device.create_shader_module(cs_desc);
    
    // Create the buffer that will store time.
    let uniforms = create_uniforms(app.time, 1f32);
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniform-buffer"),
        contents: uniforms_bytes,
        usage,
    });

    // Create the bind group and pipeline.
    let uniform_dynamic = false;
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
        .uniform_buffer(wgpu::ShaderStages::COMPUTE, uniform_dynamic)
        .texture(
            wgpu::ShaderStages::COMPUTE,
            false,
            wgpu::TextureViewDimension::D2,
            wgpu::TextureSampleType::Float { filterable: true },
        )
        .storage_texture(
            wgpu::ShaderStages::COMPUTE,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureViewDimension::D2,
            wgpu::StorageTextureAccess::WriteOnly,
        )
        .build(device);

    let bind_group = wgpu::BindGroupBuilder::new()
        .buffer::<Uniforms>(&uniform_buffer, 0..1)
        .texture_view(&texture_view) // <- Input texture
        .texture_view(&storage_texture_view)// <- Output texture
        .build(device, &bind_group_layout);

    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let pipeline = create_compute_pipeline(device, &pipeline_layout, &cs_mod);

    let compute = Compute {
        uniform_buffer,
        bind_group,
        pipeline,
    };
    compute
}

fn build_render_pipeline(window: &Ref<Window>, device: &Device, storage_texture_view: &TextureView) -> Render {
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/passtrough.wgsl");

    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    // Create the sampler for sampling from the source texture.
    let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);


    let render_bind_group_layout =
        wgpu::BindGroupLayoutBuilder::new()
            .texture(
                wgpu::ShaderStages::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
            .build(device);

    let render_bind_group = wgpu::BindGroupBuilder::new()
        .texture_view(&storage_texture_view)
        .sampler(&sampler)
        .build(device, &render_bind_group_layout);


    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&render_bind_group_layout],
        push_constant_ranges: &[],
    };
    let render_pipeline_layout = device.create_pipeline_layout(&desc);

    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&render_pipeline_layout, &vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(format)
        .add_vertex_buffer::<Vert>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(msaa_samples)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device);

    let vertices_bytes = vertices_as_bytes(&QUAD[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    let render = Render {
        bind_group: render_bind_group,
        render_pipeline,
        vertex_buffer,
    };
    render
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let egui = &mut model.gui.egui;
    let settings = &mut model.gui.settings;

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
        ui.label("Accentuate:");
        ui.add(egui::Slider::new(&mut settings.accentuate, 1.0..=20.0));

        // Random color button
        let clicked = ui.button("Random color").clicked();

        if clicked {
            settings.color = rgb(random(), random(), random());
        }
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.gui.egui.handle_raw_event(event);
}


fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(BLACK);
    {
        compute_pass(app, &model, &frame);
        render_pass(&model, &frame);
    }
    model.gui.egui.draw_to_frame(&frame).unwrap();
}

fn compute_pass(app: &App, model: &&Model, frame: &Frame) {
    let window = app.window(frame.window_id()).unwrap();
    let device = window.device();
    let win_rect = window.rect();
    let compute = &model.compute;

    // An update for the uniform buffer with the current time.
    let uniforms = create_uniforms(app.time, model.gui.settings.accentuate);
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::COPY_SRC;
    let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("uniform-data-transfer"),
        contents: uniforms_bytes,
        usage,
    });

    // The encoder we'll use to encode the compute pass.
    let desc = wgpu::CommandEncoderDescriptor {
        label: Some("convolution-compute"),
    };
    let mut encoder = device.create_command_encoder(&desc);
    encoder.copy_buffer_to_buffer(
        &new_uniform_buffer,
        0,
        &compute.uniform_buffer,
        0,
        uniforms_size,
    );
    {
        let pass_desc = wgpu::ComputePassDescriptor {
            label: Some("nannou-wgpu_compute_shader-compute_pass"),
        };
        let mut cpass = encoder.begin_compute_pass(&pass_desc);
        cpass.set_pipeline(&compute.pipeline);
        cpass.set_bind_group(0, &compute.bind_group, &[]);
        cpass.dispatch_workgroups(1024u32, 1024u32, 1);
    }

    // Submit the compute pass to the device's queue.
    window.queue().submit(Some(encoder.finish()));
}

fn render_pass(model: &&Model, frame: &Frame) {
    let shader_model = &model.render;
    //draw.to_frame(app, &frame).unwrap();
    let mut encoder = frame.command_encoder();
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &shader_model.bind_group, &[]);
    render_pass.set_pipeline(&shader_model.render_pipeline);
    render_pass.set_vertex_buffer(0, shader_model.vertex_buffer.slice(..));
    let vertex_range = 0..QUAD.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}

fn create_uniforms(time: f32, accentuate: f32) -> Uniforms {

    Uniforms {
        time,
        accentuate,
    }
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nannou"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    })
}

fn create_compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    cs_mod: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let desc = wgpu::ComputePipelineDescriptor {
        label: Some("nannou"),
        layout: Some(layout),
        module: &cs_mod,
        entry_point: "main",
    };
    device.create_compute_pipeline(&desc)
}

// See `nannou::wgpu::bytes` docs for why these are necessary.

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}

fn vertices_as_bytes(data: &[Vert]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}