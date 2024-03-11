use std::cell::Ref;
use nannou::{Frame, wgpu};
use nannou::image::DynamicImage;
use nannou::prelude::{BufferInitDescriptor, DeviceExt, Window};
use nannou::wgpu::ShaderModuleDescriptor;
use crate::shader_processing::model::{ConvolutionUniform, QUAD, ShaderModel, Vert};

pub fn init_shader(image: &DynamicImage, window: &Ref<Window>, fs_desc: ShaderModuleDescriptor, convolution: [f32; 16]) -> ShaderModel {
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();

    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");

    let vs_mod = device.create_shader_module(vs_desc);
    let fs_mod = device.create_shader_module(fs_desc);

    // Load the image as a texture.
    let texture = wgpu::Texture::from_image(window, &image);
    let texture_view = texture.view().build();

    // Create the sampler for sampling from the source texture.
    let sampler_desc = wgpu::SamplerBuilder::new().into_descriptor();
    let sampler_filtering = wgpu::sampler_filtering(&sampler_desc);
    let sampler = device.create_sampler(&sampler_desc);

    let bind_group_layout =
        wgpu::BindGroupLayoutBuilder::new()
            .texture(
                wgpu::ShaderStages::FRAGMENT,
                false,
                wgpu::TextureViewDimension::D2,
                texture_view.sample_type(),
            )
            .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
            .build(device);

    let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("uniform_bind_group_layout"),
    });

    let convolution_uniform = ConvolutionUniform {
        convolution
    };

    let convolution_uniform_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Convolution Matrix Buffer"),
            contents: bytemuck::cast_slice(&[convolution_uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        }
    );

    let bind_group = wgpu::BindGroupBuilder::new()
        .texture_view(&texture_view)
        .sampler(&sampler)
        .build(device, &bind_group_layout);

    let uniform_bind_group = wgpu::BindGroupBuilder::new()
        .binding(wgpu::BindingResource::Buffer(convolution_uniform_buffer.as_entire_buffer_binding()))
        .build(device, &uniform_bind_group_layout);

    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
        push_constant_ranges: &[],
    };
    let pipeline_layout = device.create_pipeline_layout(&desc);

    let render_pipeline = wgpu::RenderPipelineBuilder::from_layout(&pipeline_layout, &vs_mod)
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

    ShaderModel {
        bind_group,
        uniform_bind_group,
        vertex_buffer,
        render_pipeline,
        convolution_uniform,
    }
}

pub fn wgpu_render_pass(frame: Frame, shader_model: &ShaderModel) {
    let mut encoder = frame.command_encoder();
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &shader_model.bind_group, &[]);
    render_pass.set_bind_group(1, &shader_model.uniform_bind_group, &[]);
    render_pass.set_pipeline(&shader_model.render_pipeline);
    render_pass.set_vertex_buffer(0, shader_model.vertex_buffer.slice(..));
    let vertex_range = 0..QUAD.len() as u32;
    let instance_range = 0..1;
    render_pass.draw(vertex_range, instance_range);
}

// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vert]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}