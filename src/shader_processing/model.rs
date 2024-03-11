use nannou::wgpu;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vert {
    pub position: [f32; 2],
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ConvolutionUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub convolution: [f32; 16],
}

pub struct ShaderModel {
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub convolution_uniform: ConvolutionUniform,
}

pub const QUAD: [Vert; 4] = [
    Vert { position: [-1.0, 1.0] },
    Vert { position: [-1.0, -1.0] },
    Vert { position: [1.0, 1.0] },
    Vert { position: [1.0, -1.0] },
];
