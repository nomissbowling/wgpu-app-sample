struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) tex_coord: vec2<f32>,
  // @location(1) col: vec4<f32>,
  // @location(2) norm: vec3<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
  @location(0) position: vec4<f32>,
  @location(1) tex_coord: vec2<f32>,
  // @location(2) col: vec4<f32>, // vertexcol
  // @location(3) norm: vec3<f32>, // normal
) -> VertexOutput {
  var result: VertexOutput;
  result.position = transform * position;
  result.tex_coord = tex_coord;
  // result.col = col;
  // result.norm = norm;
  return result;
}

@group(0)
@binding(1)
var rgba_color: texture_2d<u32>;

@group(0)
@binding(2)
var<uniform> tex_sz: vec2<u32>;

/*
@group(0)
@binding(3)
var smplr: sampler;
*/

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  let w = f32(tex_sz.x);
  let h = f32(tex_sz.y);
  let tex = textureLoad(rgba_color, vec2<i32>(vertex.tex_coord * w), 0);
  let a = f32(tex.w) / 255.0;
/*
  let s = 1.0 - a;
  let r = (a * f32(tex.x) + s * vertex.col.x) / 255.0;
  let g = (a * f32(tex.y) + s * vertex.col.y) / 255.0;
  let b = (a * f32(tex.z) + s * vertex.col.z) / 255.0;
*/
  let r = f32(tex.x) / 255.0;
  let g = f32(tex.y) / 255.0;
  let b = f32(tex.z) / 255.0;
  return vec4<f32>(r, g, b, a);
//  return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
