use nannou::wgpu;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vert {
    pub position: [f32; 2],
}

pub struct ShaderModel {
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

pub const QUAD: [Vert; 4] = [
    Vert { position: [-1.0, 1.0] },
    Vert { position: [-1.0, -1.0] },
    Vert { position: [1.0, 1.0] },
    Vert { position: [1.0, -1.0] },
];
