use std::cell::Ref;
use nannou::{Frame, wgpu};
use nannou::image::DynamicImage;
use nannou::prelude::{BufferInitDescriptor, DeviceExt, Window};
use nannou::wgpu::ShaderModuleDescriptor;
use crate::shader_processing::model::{QUAD, ShaderModel, Vert};

pub fn init_shader(image: &DynamicImage, window: &Ref<Window>, fs_desc: ShaderModuleDescriptor) -> ShaderModel {
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
        create_bind_group_layout(device, texture_view.sample_type(), sampler_filtering);
    let bind_group = create_bind_group(device, &bind_group_layout, &texture_view, &sampler);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        msaa_samples,
    );

    let vertices_bytes = vertices_as_bytes(&QUAD[..]);
    let usage = wgpu::BufferUsages::VERTEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage,
    });

    ShaderModel {
        bind_group,
        vertex_buffer,
        render_pipeline,
    }
}

pub fn wgpu_render_pass(frame: Frame, shader_model: &ShaderModel) {
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

fn create_bind_group_layout(
    device: &wgpu::Device,
    texture_sample_type: wgpu::TextureSampleType,
    sampler_filtering: bool,
) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .texture(
            wgpu::ShaderStages::FRAGMENT,
            false,
            wgpu::TextureViewDimension::D2,
            texture_sample_type,
        )
        .sampler(wgpu::ShaderStages::FRAGMENT, sampler_filtering)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .texture_view(texture)
        .sampler(sampler)
        .build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(fs_mod)
        .color_format(dst_format)
        .add_vertex_buffer::<Vert>(&wgpu::vertex_attr_array![0 => Float32x2])
        .sample_count(sample_count)
        .primitive_topology(wgpu::PrimitiveTopology::TriangleStrip)
        .build(device)
}


// See the `nannou::wgpu::bytes` documentation for why this is necessary.
fn vertices_as_bytes(data: &[Vert]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}