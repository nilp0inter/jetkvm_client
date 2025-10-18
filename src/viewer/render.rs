//! wgpu renderer: uploads NV12 planes, decodes YUV→RGB with the stream's
//! signalled colorimetry, and applies AMD RCAS sharpening (FSR 1.0) in a
//! single fragment pass.

use std::sync::Arc;

use anyhow::{anyhow, Result as AnyResult};
use tracing::info;
use winit::window::Window;

use super::pipeline::{ColorMatrix, ColorRange, Frame};

const SHADER_SRC: &str = r#"
struct Uniforms {
    kr: f32,
    kb: f32,
    y_off: f32,
    y_scale: f32,
    c_off: f32,
    c_scale: f32,
    rcas_sharpness: f32,
    _pad: f32,
};

@group(0) @binding(0) var y_tex: texture_2d<f32>;
@group(0) @binding(1) var uv_tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> u: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    var o: VsOut;
    o.pos = vec4<f32>(positions[idx], 0.0, 1.0);
    o.uv  = uvs[idx];
    return o;
}

fn decode_yuv(coord: vec2<f32>) -> vec3<f32> {
    let y_raw  = textureSample(y_tex,  samp, coord).r;
    let uv_raw = textureSample(uv_tex, samp, coord).rg;
    let y  = (y_raw    - u.y_off) * u.y_scale;
    let cb = (uv_raw.r - u.c_off) * u.c_scale;
    let cr = (uv_raw.g - u.c_off) * u.c_scale;
    let kg = 1.0 - u.kr - u.kb;
    let r = y + 2.0 * (1.0 - u.kr) * cr;
    let b = y + 2.0 * (1.0 - u.kb) * cb;
    let g = (y - u.kr * r - u.kb * b) / kg;
    return clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
}

// AMD FSR 1.0 RCAS — Robust Contrast Adaptive Sharpening, FP32 reference path.
// Public-domain reference: https://github.com/GPUOpen-Effects/FidelityFX-FSR
// Operates on the decoded RGB at +/- 1 source-pixel offsets. Correct when the
// source frame resolution matches the swapchain (the common case here, thanks
// to EDID negotiation); degrades gracefully when it doesn't.
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let e = decode_yuv(in.uv);

    if (u.rcas_sharpness <= 0.0) {
        return vec4<f32>(e, 1.0);
    }

    let dim  = vec2<f32>(textureDimensions(y_tex));
    let step = vec2<f32>(1.0 / dim.x, 1.0 / dim.y);

    let b = decode_yuv(in.uv + vec2<f32>(0.0,    -step.y));
    let d = decode_yuv(in.uv + vec2<f32>(-step.x, 0.0));
    let f = decode_yuv(in.uv + vec2<f32>( step.x, 0.0));
    let h = decode_yuv(in.uv + vec2<f32>(0.0,     step.y));

    let mn4 = min(min(b, d), min(f, h));
    let mx4 = max(max(b, d), max(f, h));

    // Per-channel lobe weight: how much can we add the cross-pattern energy to
    // the centre without pushing any channel outside [0,1]?
    let hit_min = mn4 / max(4.0 * mx4, vec3<f32>(1.0e-5));
    let hit_max = (vec3<f32>(1.0) - mx4) /
                  min(4.0 * mn4 - vec3<f32>(4.0), vec3<f32>(-1.0e-5));
    let lobe_rgb = max(-hit_min, hit_max);
    let lobe_max = max(lobe_rgb.r, max(lobe_rgb.g, lobe_rgb.b));
    let lobe = max(-0.1875, min(lobe_max, 0.0)) * u.rcas_sharpness;

    let c = ((b + d + f + h) * lobe + e) / (4.0 * lobe + 1.0);
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
"#;

/// Per-frame shader uniforms. Mirrors the WGSL `Uniforms` struct.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    kr: f32,
    kb: f32,
    y_off: f32,
    y_scale: f32,
    c_off: f32,
    c_scale: f32,
    rcas_sharpness: f32,
    _pad: f32,
}

impl ShaderUniforms {
    fn from_frame(frame: &Frame, sharpness: f32) -> Self {
        let (kr, kb) = frame.matrix.coefficients();
        let (y_off, y_scale, c_scale) = match frame.range {
            // Limited range: Y in [16,235], CbCr in [16,240]. Expand to [0,1].
            ColorRange::Limited => (16.0 / 255.0, 255.0 / 219.0, 255.0 / 224.0),
            // Full range: already [0,1]; chroma is still centred at 128.
            ColorRange::Full => (0.0, 1.0, 1.0),
        };
        // AMD RCAS convention: con.x = exp2(-sharpness_user). User input 0
        // (no sharpening) is mapped to 0.0 here so the shader's early-out
        // disables the sharpener entirely; values >0 enable it with a
        // strength of exp2(-sharpness).
        let rcas_sharpness = if sharpness <= 0.0 {
            0.0
        } else {
            (-sharpness).exp2()
        };
        Self {
            kr,
            kb,
            y_off,
            y_scale,
            c_off: 128.0 / 255.0,
            c_scale,
            rcas_sharpness,
            _pad: 0.0,
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_buffer: wgpu::Buffer,
    sharpness: f32,
    y_texture: Option<wgpu::Texture>,
    uv_texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    frame_dims: Option<(u32, u32)>,
    last_colorimetry: Option<(ColorMatrix, ColorRange)>,
    window: Arc<Window>,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, sharpness: f32) -> AnyResult<Self> {
        let size = window.inner_size();
        // Use only primary backends (Vulkan on Linux, Metal on macOS, DX12 on
        // Windows). The GLES fallback in wgpu-hal panics during EGL init on
        // some NixOS configurations and is not needed on our target platforms.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| anyhow!("create_surface: {e}"))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("no wgpu adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("viewer device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| anyhow!("request_device: {e}"))?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Prefer a non-sRGB UNORM format: the BT.709 YUV→RGB conversion in the
        // fragment shader already produces gamma-encoded display-space values.
        // Writing those to an sRGB target would apply an extra linear→sRGB
        // transform on top, lifting blacks and washing out the image.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewer shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewer bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewer uniforms"),
            size: std::mem::size_of::<ShaderUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("viewer pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("viewer rp"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("viewer sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            sharpness,
            y_texture: None,
            uv_texture: None,
            bind_group: None,
            frame_dims: None,
            last_colorimetry: None,
            window,
        })
    }

    pub fn set_sharpness(&mut self, sharpness: f32) {
        self.sharpness = sharpness;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn ensure_textures(&mut self, width: u32, height: u32) {
        if self.frame_dims == Some((width, height)) {
            return;
        }
        let y_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("y plane"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let uv_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("uv plane"),
            size: wgpu::Extent3d {
                width: width / 2,
                height: height / 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let y_view = y_tex.create_view(&Default::default());
        let uv_view = uv_tex.create_view(&Default::default());

        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewer bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&y_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&uv_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        self.y_texture = Some(y_tex);
        self.uv_texture = Some(uv_tex);
        self.bind_group = Some(bg);
        self.frame_dims = Some((width, height));
    }

    pub fn upload_frame(&mut self, frame: &Frame) {
        self.ensure_textures(frame.width, frame.height);

        // Surface VUI changes from the H.264 stream. The JetKVM is known to
        // shift its signalled colorimetry a few frames into a session (cause
        // unknown — most likely an HDMI receiver handshake on the device).
        // Logging each change pins down whether the visible colour shift the
        // user sees corresponds to a matrix/range flip.
        let current = (frame.matrix, frame.range);
        if self.last_colorimetry != Some(current) {
            info!(
                "stream colorimetry: matrix={:?} range={:?} ({}x{})",
                frame.matrix, frame.range, frame.width, frame.height
            );
            self.last_colorimetry = Some(current);
        }

        let uniforms = ShaderUniforms::from_frame(frame, self.sharpness);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let y_tex = self.y_texture.as_ref().unwrap();
        let uv_tex = self.uv_texture.as_ref().unwrap();

        write_plane(
            &self.queue,
            y_tex,
            &frame.y_plane,
            frame.width,
            frame.height,
            frame.stride_y,
            1,
        );
        write_plane(
            &self.queue,
            uv_tex,
            &frame.uv_plane,
            frame.width / 2,
            frame.height / 2,
            frame.stride_uv,
            2,
        );
    }

    pub fn render(&mut self) -> AnyResult<()> {
        let Some(bind_group) = &self.bind_group else {
            // No frame yet — just present a black surface so the swapchain
            // stays healthy.
            let output = self
                .surface
                .get_current_texture()
                .map_err(|e| anyhow!("acquire frame: {e}"))?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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
            }
            self.queue.submit(Some(encoder.finish()));
            output.present();
            return Ok(());
        };

        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| anyhow!("acquire frame: {e}"))?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewer pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}

fn write_plane(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    src: &[u8],
    width: u32,
    height: u32,
    src_stride: u32,
    bytes_per_texel: u32,
) {
    let bytes_per_row_unpadded = width * bytes_per_texel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = ((bytes_per_row_unpadded + align - 1) / align) * align;

    if padded_bpr == src_stride && src.len() as u32 >= padded_bpr * height {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            src,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        return;
    }

    // Repack rows to the wgpu-required padded stride.
    let mut padded = vec![0u8; (padded_bpr * height) as usize];
    for row in 0..height {
        let src_off = (row * src_stride) as usize;
        let dst_off = (row * padded_bpr) as usize;
        let row_len = bytes_per_row_unpadded as usize;
        let end = src_off + row_len;
        if end > src.len() {
            break;
        }
        padded[dst_off..dst_off + row_len].copy_from_slice(&src[src_off..end]);
    }

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &padded,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(padded_bpr),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}
