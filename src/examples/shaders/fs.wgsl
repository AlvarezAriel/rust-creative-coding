struct FragmentOutput {
    @location(0) f_color: vec4<f32>,
};

struct ConvolutionUniform {
    convolution: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> convolution_matrix: ConvolutionUniform;
@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0) @binding(1)
var tex_sampler: sampler;

@fragment
fn main(@location(0) tex_coords: vec2<f32>) -> FragmentOutput {

    let w = convolution_matrix.convolution;

    let out_color: vec4<f32> = textureSample(tex, tex_sampler, tex_coords);



    return FragmentOutput(out_color);
}