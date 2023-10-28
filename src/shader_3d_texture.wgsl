// https://www.w3.org/TR/WGSL/#builtin-inputs-outputs

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) norm: vec4<f32>,
  @location(1) col: vec4<u32>,
  @location(2) tex_coord: vec2<f32>,
  @location(3) ix: u32, // not builtin(vertex_index)
};

@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(
  @location(0) position: vec4<f32>,
  @location(1) norm: vec4<f32>, // normal
  @location(2) col: vec4<u32>, // vertexcol
  @location(3) tex_coord: vec2<f32>,
  @builtin(vertex_index) ix: u32,
) -> VertexOutput {
  var result: VertexOutput;
  result.position = transform * position;
  result.norm = norm;
  result.col = col;
  result.tex_coord = tex_coord;
  result.ix = ix;
  return result;
}

struct TexSZ {
  w: u32,
  h: u32,
  ext: vec2<u32>, // x=mode: 0 square, 1 landscape, 2 portrait, 3 y=max: square
};

@group(0) @binding(1) var rgba_color: texture_2d<u32>;
@group(0) @binding(2) var<uniform> tex_sz: TexSZ;
// @group(0) @binding(3) var smplr: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  let w = f32(tex_sz.w);
  let h = f32(tex_sz.h);
  let m = f32(tex_sz.ext.y);
  var tc: vec2<f32> = vertex.tex_coord;
  if(tex_sz.ext.x == 0u){ tc.x *= w; tc.y *= h; } // square
  else if(tex_sz.ext.x == 1u){ tc.x *= w; tc.y *= w; } // landscape
  else if(tex_sz.ext.x == 2u){ tc.x *= h; tc.y *= h; } // portrait
  else{ tc.x *= m; tc.y *= m; } // y=max: square
  let tex = textureLoad(rgba_color, vec2<i32>(tc), 0);
  let vcr = f32(vertex.col.x);
  let vcg = f32(vertex.col.y);
  let vcb = f32(vertex.col.z);
  let vca = f32(vertex.col.w);
  let a = f32(tex.w) / 255.0;
  let s = 1.0 - a;
  let r = (a * f32(tex.x) + s * vcr) / 255.0;
  let g = (a * f32(tex.y) + s * vcg) / 255.0;
  let b = (a * f32(tex.z) + s * vcb) / 255.0;
/*
  let r = f32(tex.x) / 255.0;
  let g = f32(tex.y) / 255.0;
  let b = f32(tex.z) / 255.0;
*/
  return vec4<f32>(r, g, b, a);
//  return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
