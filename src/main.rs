#![doc(html_root_url = "https://docs.rs/wgpu-app-sample/0.17.4")]
//! Rust sample for wgpu-app management Vertex Texture CameraAngle
//!
//! partial fork (remove wasm32) from
//!
//! https://github.com/gfx-rs/wgpu/tree/v0.17/examples/cube
//!

use std::{borrow::Cow, future::Future, mem, pin::Pin, task};
use bytemuck;
use winit::event::{self, WindowEvent};
use wgpu::util::DeviceExt;
use wgpu;

use wgpu_app::{vt, app};

/// A wrapper for `pop_error_scope` futures that panics if an error occurs.
///
/// Given a future `inner` of an `Option<E>` for some error type `E`,
/// wait for the future to be ready, and panic if its value is `Some`.
///
/// This can be done simpler with `FutureExt`, but we don't want to add
/// a dependency just for this small case.
struct ErrorFuture<F> {
  inner: F
}

impl<F: Future<Output = Option<wgpu::Error>>> Future for ErrorFuture<F> {
  type Output = ();
  fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<()> {
    let inner = unsafe { self.map_unchecked_mut(|me| &mut me.inner) };
    inner.poll(cx).map(|error| {
      if let Some(e) = error { panic!("Rendering {e}"); }
    })
  }
}

pub struct App {
  wg: vt::WG,
  yrp: vt::YRP
}

impl app::App for App {
  fn optional_features() -> wgpu::Features {
    wgpu::Features::POLYGON_MODE_LINE
  }

  fn init(
    config: &wgpu::SurfaceConfiguration,
    _adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    queue: &wgpu::Queue
  ) -> Self {
    // Create the vertex and index buffers
    let vertex_size = mem::size_of::<vt::Vertex>();
    let vips = [
      vt::locscale(&[0.0, 0.0, 0.0], 1.0,
        vt::create_vertices_cube_6_textures(|(i, bg, m)| (i + bg) % m)),
      vt::locscale(&[0.0, -2.0, 0.0], 0.5,
        vt::create_vertices_cube_expansion_plan(|_| 4)),
      vt::locscale(&[2.0, 0.0, 0.0], 0.5,
        vt::create_vertices_cube_6_textures(|(i, _, _)| i))
    ].into_iter().map(|(vertex_data, index_data, fi)| {vt::VIP{
      vs: device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: wgpu::BufferUsages::VERTEX}),
      is: device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&index_data),
        usage: wgpu::BufferUsages::INDEX}),
      p: fi}
    }).collect();

    // Create pipeline layout
    let bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor{
        label: None,
        entries: &[
          wgpu::BindGroupLayoutEntry{
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer{
              ty: wgpu::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: wgpu::BufferSize::new(64) // mat4x4<f32>
            },
            count: None
          },
          wgpu::BindGroupLayoutEntry{
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture{
              multisampled: false,
              sample_type: wgpu::TextureSampleType::Uint,
              view_dimension: wgpu::TextureViewDimension::D2
            },
            count: None
          },
          wgpu::BindGroupLayoutEntry{
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer{
              ty: wgpu::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: wgpu::BufferSize::new(16) // 2 x vec2<u32>
            },
            count: None
          }
        ]
      });
    let pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor{
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[]
      });

    // Create buffer resources (as transform mat4x4<f32> on bind_group: 0)
    let mvp = vt::CameraAngle::new(
      glam::Vec3::new(1.5f32, -5.0, 3.0),
      glam::Vec3::ZERO,
      glam::Vec3::Z).generate_mvp(config.width as f32 / config.height as f32);
    let uniform_buf = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor{
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(mvp.as_ref()), // &[f32; 16]
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
      });

    // bind group (texture texels texture_extent -> texture_view -> binding: 1)
    let cols4u: Vec<&[u8; 4]> = vec![ // for texels_rgba
      &[255, 63, 127, 128], // left top
      &[127, 255, 63, 128], // right top
      &[63, 127, 255, 128], // left bottom
      &[255, 255, 63, 128]]; // right bottom
    let cols4f: Vec<&[f32; 4]> = vec![ // for texels_mandelbrot_4c
      &[5.0, 15.0, 50.0, 0.0], // right Orange-Yellow
      &[50.0, 5.0, 15.0, 0.0], // left Blue-Green
      &[15.0, 50.0, 5.0, 0.0], // back Violet
      &[5.0, 50.0, 15.0, 0.0], // front Pink-Magenta
      &[50.0, 15.0, 5.0, 0.0], // top Cyan-Blue (skip)
      &[15.0, 5.0, 50.0, 0.0]]; // bottom Lime-Green (skip)
    let mut texels_list = vec![ // textures
      vt::load_texels("res/tex_RGBY_4x4_24_bpr12.png").unwrap(), // to 4 16
      vt::load_texels("res/tex_RGBY_2x2_24_bpr6.png").unwrap(), // to 4 8
      vt::load_texels("res/tex_RGBY_256x192_landscape.png").unwrap(), // 4 1024
      vt::load_texels("res/tex_RGBY_192x256_portrait.png").unwrap(), // 4 768
      vt::load_texels("res/tex_cube_256x256.png").unwrap(), // 4 1024 bpr
      vt::create_texels_rgba(256, &cols4u)];
    for c4f in cols4f {
      texels_list.push(vt::create_texels_mandelbrot_4c(256, c4f))
    }
    let bind_group = texels_list.into_iter().map(|(hwd, texels)| {
      let texture_extent = wgpu::Extent3d{
        width: hwd.1,
        height: hwd.0,
        depth_or_array_layers: 1 // always 1 not 4 (does not mean color depth)
      };
      let texture = device.create_texture(&wgpu::TextureDescriptor{
        label: None,
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2, // always D2 (not D3)
        // https://docs.rs/wgpu/0.17.1/wgpu/enum.TextureFormat.html
        format: match hwd.2 {
          1 => wgpu::TextureFormat::R8Uint, // when gray
          4 => wgpu::TextureFormat::Rgba8Uint, // when RGBA
          _ => wgpu::TextureFormat::Rgba8Uint}, // expect 3 but not support RGB
        usage: wgpu::TextureUsages::TEXTURE_BINDING
         | wgpu::TextureUsages::COPY_DST,
        view_formats: &[]
      });
      let texture_view = texture.create_view(
        &wgpu::TextureViewDescriptor::default());
      queue.write_texture(
        texture.as_image_copy(),
        &texels,
        wgpu::ImageDataLayout{
          offset: 0,
          bytes_per_row: Some(hwd.3), // Some(bytes_per_row)
          rows_per_image: Some(hwd.0) // None
        },
        texture_extent
      );
      // Create buffer resources (as TexSZ on bind_group: 2)
      let tex_sz = vt::TexSZ{w: hwd.1, h: hwd.0, ext: [0, 256]};
      let tex_sz_buf = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor{
          label: Some("Texture Size Buffer"),
          contents: bytemuck::cast_slice(tex_sz.as_ref()), // &[u32; 4]
          usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });
      vt::TextureBindGroup{
        group: device.create_bind_group(&wgpu::BindGroupDescriptor{
          layout: &bind_group_layout,
          entries: &[
            wgpu::BindGroupEntry{
              binding: 0,
              resource: uniform_buf.as_entire_binding()
            },
            wgpu::BindGroupEntry{
              binding: 1,
              resource: wgpu::BindingResource::TextureView(&texture_view)
            },
            wgpu::BindGroupEntry{
              binding: 2,
              resource: tex_sz_buf.as_entire_binding() // tex_sz.[w|h|ext]
            }
          ],
          label: None}),
        sz: tex_sz,
        buf: tex_sz_buf}
    }).collect();

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
      label: None,
      source: wgpu::ShaderSource::Wgsl(
        Cow::Borrowed(include_str!("shader_3d_texture.wgsl")))
    });

    let vertex_buffers = [wgpu::VertexBufferLayout{
      array_stride: vertex_size as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![
        0 => Float32x4, // pos
        1 => Float32x4, // norm
        2 => Uint32x4, // col
        3 => Float32x2, // tex_coord
        4 => Uint32] // ix
    }];

    let pipeline = device.create_render_pipeline(
      &wgpu::RenderPipelineDescriptor{
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState{
          module: &shader,
          entry_point: "vs_main",
          buffers: &vertex_buffers
        },
        fragment: Some(wgpu::FragmentState{
          module: &shader,
          entry_point: "fs_main",
          targets: &[Some(config.view_formats[0].into())]
        }),
        primitive: wgpu::PrimitiveState{
          front_face: wgpu::FrontFace::Ccw, // (from wire)
          cull_mode: Some(wgpu::Face::Back), // None Some(...::Back/Front)
          polygon_mode: wgpu::PolygonMode::Fill, // Fill Line Point (from wire)
          // https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/
          // topology: wgpu::PrimitiveTopology::TriangleList,
          // strip_index_format: None,
          unclipped_depth: true, // Requires Features::DEPTH_CLIP_CONTROL
          conservative: false, // Requires Features::CONSERVATIVE_RASTERIZATION
          ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None
      });

    let pipeline_wire = if device.features()
      .contains(wgpu::Features::POLYGON_MODE_LINE) {
      let pipeline_wire = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor{
          label: None,
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState{
            module: &shader,
            entry_point: "vs_main",
            buffers: &vertex_buffers
          },
          fragment: Some(wgpu::FragmentState{
            module: &shader,
            entry_point: "fs_wire",
            targets: &[Some(wgpu::ColorTargetState{
              format: config.view_formats[0],
              blend: Some(wgpu::BlendState{
                color: wgpu::BlendComponent{
                  operation: wgpu::BlendOperation::Add,
                  src_factor: wgpu::BlendFactor::SrcAlpha,
                  dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha
                },
                alpha: wgpu::BlendComponent::REPLACE
              }),
              write_mask: wgpu::ColorWrites::ALL
            })]
          }),
          primitive: wgpu::PrimitiveState{
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back), // None Some(...::Back/Front)
            polygon_mode: wgpu::PolygonMode::Line, // Fill Line Point
            ..Default::default()
          },
          depth_stencil: None,
          multisample: wgpu::MultisampleState::default(),
          multiview: None
        });
      Some(pipeline_wire)
    } else {
      None
    };

    // Done
    App{
      wg: vt::WG{vips, bind_group, bg: 0, mvp, uniform_buf,
        pipeline, pipeline_wire, wire: false},
      yrp: vt::YRP{yaw: 60.0, roll: 60.0, pitch: 60.0, tick: 0}
    }
  }

  fn update(
    &mut self,
    ev: WindowEvent, // winit::event::WindowEvent
    config: &wgpu::SurfaceConfiguration,
    device: &wgpu::Device,
    queue: &wgpu::Queue
  ) {
    match ev {
    WindowEvent::CursorEntered{device_id: _} => { log::info!(" Enter"); }
    WindowEvent::CursorLeft{device_id: _} => { log::info!(" Left"); }
    WindowEvent::CursorMoved{device_id: _, position, ..} => {
      log::info!(" ({:7.3e} {:7.3e})", position.x, position.y); // 800 600
      self.yrp.tick += 1;
    }
    WindowEvent::MouseInput{device_id: _, state, button, ..} => {
      // state: event::ElementState::Pressed, // Released
      // button: event::MouseButton::Left, // Right Middle Other(u16)
      log::info!(" {:?} {:?}", button, state);
      match (button, state) {
      (event::MouseButton::Left, event::ElementState::Pressed) => {
        self.yrp = vt::YRP{yaw: 60.0, roll: 60.0, pitch: 60.0, tick: 0};
      }
      _ => {}
      }
    }
    WindowEvent::KeyboardInput{device_id: _, input, ..} => {
      match input {
      event::KeyboardInput{scancode, state, virtual_keycode, ..} => {
        log::info!(" {:?} {:?}", state, scancode);
        if let Some(vk) = virtual_keycode {
          log::info!(" {:?}", vk); // JIS 106 key '+' mapped as '='
          match (state, vk) {
          (event::ElementState::Released, _) => {},
          (_, event::VirtualKeyCode::Key0 | event::VirtualKeyCode::Numpad0) =>
            self.yrp = vt::YRP{yaw: -90.0, roll: 0.0, pitch: 0.0, tick: 0},
          (_, event::VirtualKeyCode::Left) => self.yrp.yaw -= 6.0,
          (_, event::VirtualKeyCode::Right) => self.yrp.yaw += 6.0,
          (_, event::VirtualKeyCode::LControl) => self.yrp.roll -= 6.0,
          (_, event::VirtualKeyCode::RControl) => self.yrp.roll += 6.0,
          (_, event::VirtualKeyCode::Up) => self.yrp.pitch -= 6.0,
          (_, event::VirtualKeyCode::Down) => self.yrp.pitch += 6.0,
          (_, event::VirtualKeyCode::W) => self.wg.wire = !self.wg.wire,
          (_, event::VirtualKeyCode::T) =>
            self.wg.bg = (self.wg.bg + 1) % self.wg.bind_group.len(),
          _ => self.yrp.tick = 0
          }
        } else {
          log::warn!(" unknown"); // '~' '`' '*'
        }
      }
      // _ => {} // unreachable ('r' prints GlobalReport)
      }
    }
    _ => {}
    }
    self.wg.update_matrix(config, device, queue, &self.yrp);
  }

  fn resize(
    &mut self,
    config: &wgpu::SurfaceConfiguration,
    device: &wgpu::Device,
    queue: &wgpu::Queue
  ) {
    self.wg.update_matrix(config, device, queue, &self.yrp);
  }

  fn render(
    &mut self,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    spawner: &app::Spawner
  ) {
    self.wg.draw(view, device, queue);
    // If an error occurs, report it and panic.
    spawner.spawn_local(ErrorFuture{ inner: device.pop_error_scope() });
  }
}

fn main() {
  app::run::<App>("cube");
}
