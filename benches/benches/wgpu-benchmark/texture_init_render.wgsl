@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4f {
    let positions = array(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4f(positions[index], 0.0, 1.0);
}

@fragment
fn solid() -> @location(0) vec4f {
    return vec4f(64.0, 128.0, 192.0, 255.0) / 255.0;
}

@group(0) @binding(0) var a: texture_2d<f32>;
@group(0) @binding(1) var b: texture_2d<f32>;

@fragment
fn sample_pair() -> @location(0) vec4f {
    return (textureLoad(a, vec2i(0), 0) + textureLoad(a, vec2i(0), 1)
          + textureLoad(b, vec2i(0), 0) + textureLoad(b, vec2i(0), 1)) / 4.0;
}
