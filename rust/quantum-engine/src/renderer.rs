use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    time: f32,
    _pad: f32,
    width: f32,
    height: f32,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Renderer {
    pub async fn new(
        native_window: *mut std::ffi::c_void,
        initial_width: u32,
        initial_height: u32,
    ) -> Self {
        // Create raw window handle from the pointer
        let raw = raw_window_handle::AndroidNdkWindowHandle::new(
            std::ptr::NonNull::new(native_window).expect("native_window is null"),
        );
        let raw_handle = raw_window_handle::RawWindowHandle::AndroidNdk(raw.into());

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        // Safety: surface must be created from valid raw handle
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: raw_window_handle::RawDisplayHandle::Android(
                        raw_window_handle::AndroidDisplayHandle::new(),
                    ),
                    raw_window_handle: raw_handle,
                })
                .expect("Failed to create surface")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Quantum Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: initial_width.max(1),
            height: initial_height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // ---- Uniforms (time + resolution) ----
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniforms BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms BG"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // ---- "Quantum field" shader: full-screen effect driven by time ----
        let shader_source = r#"
            struct Uniforms {
                time: f32,
                _pad: f32,
                width: f32,
                height: f32,
            };
            @group(0) @binding(0) var<uniform> u: Uniforms;

            struct VsOut {
                @builtin(position) pos: vec4<f32>,
            };

            @vertex
            fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
                // Full-screen triangle
                var p = array<vec2<f32>, 3>(
                    vec2(-1.0, -3.0),
                    vec2( 3.0,  1.0),
                    vec2(-1.0,  1.0)
                );
                var out: VsOut;
                out.pos = vec4<f32>(p[i], 0.0, 1.0);
                return out;
            }

            @fragment
            fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
                let res = vec2<f32>(u.width, u.height);
                // Centered, aspect-corrected coords in ~[-1, 1]
                let uv = (in.pos.xy * 2.0 - res) / min(res.x, res.y);
                let t = u.time;

                // Deep space background with a subtle vertical wash
                var col = vec3<f32>(0.015, 0.01, 0.04) + 0.03 * vec3<f32>(0.3, 0.1, 0.5) * (1.0 - uv.y);

                // --- Orbiting quantum "particles" (glowing points) ---
                for (var k = 0; k < 6; k++) {
                    let fk = f32(k);
                    let ang = t * (0.35 + 0.12 * fk) + fk * 1.0471976; // staggered speeds
                    let rad = 0.35 + 0.28 * sin(t * 0.4 + fk * 2.4);
                    let p = vec2<f32>(cos(ang), sin(ang)) * rad;
                    let d = length(uv - p);
                    // Additive glow, hue shifts per particle
                    let hue = 0.5 + 0.5 * sin(fk * 1.3 + t * 0.25);
                    let pcol = mix(vec3<f32>(0.48, 0.18, 0.97), vec3<f32>(0.20, 0.65, 1.0), hue);
                    col += pcol * 0.0045 / (d * d + 0.0015);
                }

                // --- Interference rings emanating from the center ---
                let r = length(uv);
                let wave = sin(r * 18.0 - t * 2.2) * 0.5 + 0.5;
                let ring = wave * exp(-r * 1.8);
                col += vec3<f32>(0.35, 0.12, 0.75) * ring * 0.35;

                // --- Slow plasma shimmer over everything ---
                let sh = sin(uv.x * 3.0 + t * 0.7) * sin(uv.y * 3.0 - t * 0.5);
                col += vec3<f32>(0.10, 0.03, 0.22) * (sh * 0.5 + 0.5) * 0.25;

                // Vignette
                col *= 1.0 - 0.35 * smoothstep(0.6, 1.6, r);

                // Soft tone map
                col = col / (col + vec3<f32>(1.0));
                col = pow(col, vec3<f32>(0.85));

                return vec4<f32>(col, 1.0);
            }
        "#;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("QuantumField"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            uniform_buffer,
            bind_group,
            width: initial_width.max(1),
            height: initial_height.max(1),
        }
    }

    pub fn render(&mut self, time: f32) {
        let uniforms = Uniforms {
            time,
            _pad: 0.0,
            width: self.width as f32,
            height: self.height as f32,
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let output = match self.surface.get_current_texture() {
            Ok(o) => o,
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                // Reconfigure and retry once; skip the frame if it still fails.
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(o) => o,
                    Err(e) => {
                        log::warn!("skipping frame: {e:?}");
                        return;
                    }
                }
            }
            Err(e) => {
                log::warn!("skipping frame: {e:?}");
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
