struct VertexOutput {
  @location(0) tex_coord: vec2<f32>,
  @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
  @location(0) position: vec4<f32>,
  @location(1) tex_coord: vec2<f32>
) -> VertexOutput {
  var result: VertexOutput;
  result.tex_coord = tex_coord;
  result.position = transform * position;
  return result;
}

@group(0)
@binding(1)
var rgba_color: texture_2d<u32>;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  let tex = textureLoad(rgba_color, vec2<i32>(vertex.tex_coord * 256.0), 0);
  let r = f32(tex.x) / 255.0;
  let g = f32(tex.y) / 255.0;
  let b = f32(tex.z) / 255.0;
  let a = f32(tex.w) / 255.0;
  return vec4<f32>(r, g, b, a);
//  return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
