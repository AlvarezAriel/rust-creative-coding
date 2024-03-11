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
    let index: u32 = id.x;
    let phase: f32 = uniforms.time + f32(index) * uniforms.freq / f32(uniforms.oscillator_count);
    output.data[index] = sin(phase) * 0.5 + 0.5;

    let size = textureDimensions(inTexture, 0);
    let position = vec2<u32>(vec2(0.0));
    let color = textureLoad(inTexture, position, 0);
    textureStore(outTexture, position, color);

    return;
}