struct Buffer {
    data: array<f32>,
};

struct Uniforms {
    time: f32,
    freq: f32,
    oscillator_count: u32,
};

@group(0) @binding(0)
var<storage, read_write> output: Buffer;

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

@group(0) @binding(2)
var inTexture: texture_2d<f32>;

@group(0) @binding(3)
var outTexture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var GAUSSIAN_BLUR_KERNEL: array<f32, 25> = array<f32, 25>(
        2., 4., 5., 4., 2.,
        4., 9., 12., 9., 4.,
        5., 12., 15., 12., 5.,
        4., 9., 12., 9., 4.,
        2., 4., 5., 4., 2.,
    );

    var GAUSSIAN_BLUR_KERNEL_TYPE_2: array<f32, 25> = array<f32, 25>(
        1.,   4.,  6.,  4.,  1., 
        4.,  16., 24., 16.,  4., 
        6.,  24., 36., 24.,  6.,
        4.,  16., 24., 16.,  4.,
        1.,   4.,  6.,  4.,  1.,
    );


    var GAUSSIAN_BLUR_STEPS: array<vec2<f32>, 25>  = array<vec2<f32>, 25>(
        vec2(-2., -2.), vec2(-1., -2.), vec2(0., -2.), vec2(1., -2.), vec2(2., -2.),
        vec2(-2., -1.), vec2(-1., -1.), vec2(0., -1.), vec2(1., -1.), vec2(2., -1.),
        vec2(-2., 0.0), vec2(-1., 0.0), vec2(0., 0.0), vec2(1., 0.0), vec2(2., 0.0),
        vec2(-2., 1.0), vec2(-1., 1.0), vec2(0., 1.0), vec2(1., 1.0), vec2(2., 1.0),
        vec2(-2., 2.0), vec2(-1., 2.0), vec2(0., 2.0), vec2(1., 2.0), vec2(2., 2.0)
    );


    let index: u32 = id.x;
    let indey: u32 = id.y;
    let phase: f32 = uniforms.time + f32(index) * uniforms.freq / f32(uniforms.oscillator_count);
    output.data[index] = sin(phase) * 0.5 + 0.5;

    var colorA = 0.0;
    var accumA = 0.0;

    var colorB = 0.0;
    var accumB = 0.0;

    let lum = vec4(0.375, 0.5, 0.125, 0.);

    for (var i = 0u; i < 25u; i = i + 1u) {
        let pixel = textureLoad(inTexture, vec2<u32>(vec2<f32>(id.xy) + GAUSSIAN_BLUR_STEPS[i]), 0);

        let cA = pixel * GAUSSIAN_BLUR_KERNEL[i];
        colorA += dot(cA, lum);
        accumA += GAUSSIAN_BLUR_KERNEL[i];

        let cB = pixel * GAUSSIAN_BLUR_KERNEL_TYPE_2[i];
        colorB += dot(cB, lum);
        accumB += GAUSSIAN_BLUR_KERNEL_TYPE_2[i];
    }

    let gA =  colorA / accumA;
    let gB =  colorB / accumB;
    let diff = gB - gA;
    let distance = diff * 10.;

    textureStore(outTexture, id.xy,  vec4(vec3(distance), 1.0));

    return;
}