use std::panic::catch_unwind;
use std::sync::Arc;

use once_cell::sync::OnceCell;
use pollster::block_on;
use wgpu::util::DeviceExt;

// GPU context is created lazily; if creation fails we simply skip GPU resizing.
struct GpuContext {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline_resize: wgpu::RenderPipeline,
    pipeline_globals: wgpu::RenderPipeline,
    bind_layout_resize: wgpu::BindGroupLayout,
    bind_layout_globals: wgpu::BindGroupLayout,
    max_safe_dim: u32,
    max_safe_pixels: u64,
}

static GPU_CONTEXT: OnceCell<Result<Arc<GpuContext>, String>> = OnceCell::new();

fn init_gpu_context() -> Result<Arc<GpuContext>, String> {
    // Headless instance; use all backends to maximize compatibility.
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Request an adapter; prefer high-performance if available.
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok_or_else(|| "No suitable GPU adapter found".to_string())?;

    // Request the full adapter limits so we can handle large RAWs on capable GPUs (e.g. RTX 30xx).
    let adapter_limits = adapter.limits();
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("openroom-gpu-device"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter_limits,
        },
        None,
    ))
    .map_err(|e| format!("Failed to create GPU device: {e:?}"))?;

    let device: Arc<wgpu::Device> = Arc::new(device);
    let queue: Arc<wgpu::Queue> = Arc::new(queue);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("openroom-gpu-shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
@group(0) @binding(0) var samp : sampler;
@group(0) @binding(1) var tex : texture_2d<f32>;
@group(0) @binding(2) var<uniform> globals : Globals;

struct VsOut {
  @builtin(position) pos : vec4f,
  @location(0) uv : vec2f,
};

struct Globals {
  exposure_mul : f32,
  contrast : f32,
  highlights : f32,
  shadows : f32,
  whites : f32,
  blacks : f32,
  vibrance : f32,
  saturation : f32,
  temp : f32,
  tint : f32,
  _pad0 : f32,
  _pad1 : f32,
};

@vertex
fn vs(@builtin(vertex_index) idx : u32) -> VsOut {
  var positions = array<vec2f, 3>(
    vec2f(-1.0, -3.0),
    vec2f(3.0, 1.0),
    vec2f(-1.0, 1.0)
  );
  var out : VsOut;
  let pos = positions[idx];
  out.pos = vec4f(pos, 0.0, 1.0);
  out.uv = (pos + 1.0) * 0.5;
  return out;
}

@fragment
fn fs_resize(in: VsOut) -> @location(0) vec4f {
  // clamp UV for safety and flip Y to match image origin (top-left)
  let uv = clamp(in.uv, vec2f(0.0, 0.0), vec2f(1.0, 1.0));
  let uv_flipped = vec2f(uv.x, 1.0 - uv.y);
  return textureSample(tex, samp, uv_flipped);
}

@fragment
fn fs_globals(in: VsOut) -> @location(0) vec4f {
  let uv = clamp(in.uv, vec2f(0.0, 0.0), vec2f(1.0, 1.0));
  let uv_flipped = vec2f(uv.x, 1.0 - uv.y);
  var c = textureSample(tex, samp, uv_flipped);
  var rgb = c.rgb;

  // apply globals (mirrors CPU path)
  rgb = rgb * globals.exposure_mul;
  rgb.r = rgb.r * (1.0 + globals.temp * 0.5 + globals.tint * 0.2);
  rgb.b = rgb.b * (1.0 - globals.temp * 0.5 + globals.tint * 0.2);
  rgb.g = rgb.g * (1.0 - globals.tint * 0.2);

  let l = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
  let highlights_mask = max(l - 0.5, 0.0) * 2.0;
  let shadows_mask = max(0.5 - l, 0.0) * 2.0;
  rgb = rgb * (1.0 + globals.highlights * highlights_mask);
  rgb = rgb * (1.0 + globals.shadows * shadows_mask);
  rgb = rgb + globals.whites * 0.1;
  rgb = rgb - globals.blacks * 0.1;
  rgb = (rgb - vec3f(0.5,0.5,0.5)) * (1.0 + globals.contrast) + vec3f(0.5,0.5,0.5);

  let l2 = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
  let vib_mask = clamp(1.0 - (abs(rgb.r - l2) + abs(rgb.g - l2) + abs(rgb.b - l2)) / 3.0, 0.0, 1.0);
  let vib_factor = 1.0 + globals.vibrance * vib_mask;
  let sat_factor = 1.0 + globals.saturation;
  rgb = l2 + (rgb - l2) * sat_factor * vib_factor;
  rgb = clamp(rgb, vec3f(0.0,0.0,0.0), vec3f(1.0,1.0,1.0));
  return vec4f(rgb, c.a);
}
"#
            .into(),
        ),
    });

    let bind_layout_resize = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("openroom-gpu-bind-resize"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
        ],
    });

    let bind_layout_globals = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("openroom-gpu-bind-globals"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(64),
                },
                count: None,
            },
        ],
    });

    let pipeline_layout_resize = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("openroom-gpu-pipeline-resize"),
        bind_group_layouts: &[&bind_layout_resize],
        push_constant_ranges: &[],
    });

    let pipeline_layout_globals = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("openroom-gpu-pipeline-globals"),
        bind_group_layouts: &[&bind_layout_globals],
        push_constant_ranges: &[],
    });

    let pipeline_resize = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("openroom-gpu-render-resize"),
        layout: Some(&pipeline_layout_resize),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_resize",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let pipeline_globals = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("openroom-gpu-render-globals"),
        layout: Some(&pipeline_layout_globals),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_globals",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let max_dim = device.limits().max_texture_dimension_2d;
    let max_safe_dim = max_dim.min(8192);
    let max_safe_pixels = 150_000_000; // ~150 MP guardrail

    Ok(Arc::new(GpuContext {
        device,
        queue,
        pipeline_resize,
        pipeline_globals,
        bind_layout_resize,
        bind_layout_globals,
        max_safe_dim,
        max_safe_pixels,
    }))
}

fn gpu_context() -> Option<Arc<GpuContext>> {
    let res = GPU_CONTEXT.get_or_init(|| {
        catch_unwind(|| init_gpu_context()).unwrap_or_else(|_| {
            Err("GPU context init panicked; GPU path disabled for this session".to_string())
        })
    });
    match res {
        Ok(ctx) => Some(ctx.clone()),
        Err(_) => None,
    }
}

pub fn available() -> bool {
    gpu_context().is_some()
}

// Resize an RGBA8 image using the GPU. Returns None if GPU is unavailable or any step fails.
pub fn resize_rgba(
    src: &image::RgbaImage,
    target_w: u32,
    target_h: u32,
) -> Option<image::RgbaImage> {
    let ctx = gpu_context()?;
    if target_w == 0 || target_h == 0 {
        return None;
    }

    // Respect device limits; very large RAWs may exceed max texture dimension.
    if src.width() > ctx.max_safe_dim
        || src.height() > ctx.max_safe_dim
        || target_w > ctx.max_safe_dim
        || target_h > ctx.max_safe_dim
    {
        return None;
    }
    let pixels = (src.width() as u64) * (src.height() as u64);
    if pixels > ctx.max_safe_pixels {
        return None;
    }

    let device = &ctx.device;
    let queue = &ctx.queue;

    let src_size = wgpu::Extent3d {
        width: src.width(),
        height: src.height(),
        depth_or_array_layers: 1,
    };
    let dst_size = wgpu::Extent3d {
        width: target_w,
        height: target_h,
        depth_or_array_layers: 1,
    };

    let src_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("openroom-gpu-src"),
        size: src_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &src_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        src.as_raw(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * src.width()),
            rows_per_image: Some(src.height()),
        },
        src_size,
    );

    let src_view = src_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("openroom-gpu-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("openroom-gpu-bind-resize"),
        layout: &ctx.bind_layout_resize,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
        ],
    });

    let dst_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("openroom-gpu-dst"),
        size: dst_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let dst_view = dst_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("openroom-gpu-encoder"),
    });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("openroom-gpu-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.pipeline_resize);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    // Buffer for readback
    let bytes_per_row = 4 * target_w;
    let padded_bytes_per_row = ((bytes_per_row as usize
        + (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize - 1))
        / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let output_buffer_size = (padded_bytes_per_row * target_h as usize) as u64;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("openroom-gpu-readback"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &dst_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row as u32),
                rows_per_image: Some(target_h),
            },
        },
        dst_size,
    );

    queue.submit(Some(encoder.finish()));

    // Wait for GPU work to finish and map the buffer.
    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) =
        futures_intrusive::channel::shared::oneshot_channel::<Result<(), wgpu::BufferAsyncError>>();
    buffer_slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    device.poll(wgpu::Maintain::Wait);
    let _ = block_on(rx.receive());

    let data = buffer_slice.get_mapped_range();
    let mut out = image::RgbaImage::new(target_w, target_h);

    for y in 0..target_h as usize {
        let src_start = y * padded_bytes_per_row;
        let src_end = src_start + bytes_per_row as usize;
        let row = &data[src_start..src_end];
        let dst_start = y * (bytes_per_row as usize);
        let dst_end = dst_start + (bytes_per_row as usize);
        out.as_mut()[dst_start..dst_end].copy_from_slice(row);
    }

    drop(data);
    output_buffer.unmap();

    Some(out)
}

pub fn apply_globals_rgba(
    src: &image::RgbaImage,
    globals: &crate::models::GlobalAdjustments,
) -> Option<image::RgbaImage> {
    let ctx = gpu_context()?;
    if src.width() > ctx.max_safe_dim || src.height() > ctx.max_safe_dim {
        return None;
    }
    let pixels = (src.width() as u64) * (src.height() as u64);
    if pixels > ctx.max_safe_pixels {
        return None;
    }

    let device = &ctx.device;
    let queue = &ctx.queue;

    let size = wgpu::Extent3d {
        width: src.width(),
        height: src.height(),
        depth_or_array_layers: 1,
    };

    let src_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("openroom-gpu-globals-src"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &src_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        src.as_raw(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * src.width()),
            rows_per_image: Some(src.height()),
        },
        size,
    );

    let src_view = src_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("openroom-gpu-globals-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    // Pack globals into a uniform buffer (align to 16-byte multiples).
    let to_f32 = |v: f32| v;
    let data_f32 = [
        to_f32(2f32.powf(globals.exposure_ev)),
        to_f32(globals.contrast / 100.0),
        to_f32(globals.highlights / 100.0),
        to_f32(globals.shadows / 100.0),
        to_f32(globals.whites / 100.0),
        to_f32(globals.blacks / 100.0),
        to_f32(globals.vibrance / 100.0),
        to_f32(globals.saturation / 100.0),
        to_f32(globals.temp / 100.0),
        to_f32(globals.tint / 100.0),
        0.0,
        0.0,
    ];
    let mut raw_bytes = Vec::with_capacity(data_f32.len() * 4);
    for f in data_f32 {
        raw_bytes.extend_from_slice(&f.to_ne_bytes());
    }

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("openroom-gpu-globals-uniform"),
        contents: &raw_bytes,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("openroom-gpu-bind-globals"),
        layout: &ctx.bind_layout_globals,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });

    let dst_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("openroom-gpu-globals-dst"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let dst_view = dst_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("openroom-gpu-globals-encoder"),
    });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("openroom-gpu-globals-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.pipeline_globals);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    let bytes_per_row = 4 * src.width();
    let padded_bytes_per_row = ((bytes_per_row as usize
        + (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize - 1))
        / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let output_buffer_size = (padded_bytes_per_row * src.height() as usize) as u64;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("openroom-gpu-globals-readback"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &dst_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row as u32),
                rows_per_image: Some(src.height()),
            },
        },
        size,
    );

    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) =
        futures_intrusive::channel::shared::oneshot_channel::<Result<(), wgpu::BufferAsyncError>>();
    buffer_slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    device.poll(wgpu::Maintain::Wait);
    let _ = block_on(rx.receive());

    let data = buffer_slice.get_mapped_range();
    let mut out = image::RgbaImage::new(src.width(), src.height());
    for y in 0..src.height() as usize {
        let src_start = y * padded_bytes_per_row;
        let src_end = src_start + bytes_per_row as usize;
        let row = &data[src_start..src_end];
        let dst_start = y * (bytes_per_row as usize);
        let dst_end = dst_start + (bytes_per_row as usize);
        out.as_mut()[dst_start..dst_end].copy_from_slice(row);
    }
    drop(data);
    output_buffer.unmap();

    Some(out)
}
