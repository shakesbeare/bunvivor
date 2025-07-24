#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> transparent_color: vec3<f32>;
@group(2) @binding(1) var base_texture: texture_2d<f32>;
@group(2) @binding(2) var base_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var sample = textureSample(base_texture, base_sampler, mesh.uv);
    if (sample.r == transparent_color.r && sample.g == transparent_color.g && sample.b == transparent_color.b) {
        return vec4(sample.rgb, 0);
    }

    return sample.rgba;
}
