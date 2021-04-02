use imgui::{DrawCmd, TextureId, Textures};
use wgpu::{util::DeviceExt, Device, Queue, Texture};
use bytemuck::Zeroable;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImguiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    col: [f32; 4],
}

impl ImguiVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ImguiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 2]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for ImguiVertex {}
unsafe impl bytemuck::Zeroable for ImguiVertex {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ImguiUniform {
    matrix: [[f32; 4]; 4],
}

unsafe impl bytemuck::Pod for ImguiUniform {}
unsafe impl bytemuck::Zeroable for ImguiUniform {}

pub struct TextureData {
    texture: Texture,
    bind_group: wgpu::BindGroup,
}

pub struct ImguiRenderer {
    font_texture: TextureData,
    textures: Textures<TextureData>,

    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl ImguiRenderer {
    pub fn new(
        ctx: &mut imgui::Context,
        device: &Device,
        queue: &Queue,
        swapchain: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let vs_module = device.create_shader_module(&wgpu::include_spirv!("imgui.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("imgui.frag.spv"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("imgui texture group layout"),
            });

        let uniform = ImguiUniform::zeroed();

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("imgui uniform buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("imgui uniform bind group layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform bind group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("imgui render pipeline"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("imgui render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[ImguiVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swapchain.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("imgui sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        ctx.io_mut()
            .backend_flags
            .insert(imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET);

        let font_texture = Self::upload_font_texture(
            ctx.fonts(),
            device,
            queue,
            &sampler,
            &texture_bind_group_layout,
        );

        ImguiRenderer {
            font_texture,
            textures: Textures::new(),
            render_pipeline,
            texture_bind_group_layout,
            uniform_bind_group,
            uniform_buffer,
            sampler,
        }
    }

    fn upload_font_texture(
        mut fonts: imgui::FontAtlasRefMut,
        device: &Device,
        queue: &Queue,
        sampler: &wgpu::Sampler,
        group_layout: &wgpu::BindGroupLayout,
    ) -> TextureData {
        let font_texture = fonts.build_rgba32_texture();

        let extend = wgpu::Extent3d {
            width: font_texture.width,
            height: font_texture.height,
            depth: 1,
        };

        let disc = wgpu::TextureDescriptor {
            label: Some("imgui font atlas"),
            size: extend,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Uint,
            usage: wgpu::TextureUsage::SAMPLED,
        };

        let texture = device.create_texture_with_data(queue, &disc, font_texture.data);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba32Uint),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        TextureData {
            texture,
            bind_group: group,
        }
    }

    fn lookup_texture(&self, id: TextureId) -> Option<&TextureData> {
        if id.id() == usize::MAX {
            Some(&self.font_texture)
        } else {
            self.textures.get(id)
        }
    }

    pub fn render_imgui(
        &mut self,
        draw_data: &imgui::DrawData,
        device: &Device,
        queue: &Queue,
        encoder: &wgpu::CommandEncoder,
        frame: &wgpu::SwapChainFrame,
    ) {
        let fb_width = draw_data.display_size[0] * draw_data.framebuffer_scale[0];
        let fb_height = draw_data.display_size[1] * draw_data.framebuffer_scale[1];

        if !(fb_width > 0.0 && fb_height > 0.0) {
            return;
        }

        let left = draw_data.display_pos[0];
        let right = draw_data.display_pos[0] + draw_data.display_size[0];
        let top = draw_data.display_pos[1];
        let bottom = draw_data.display_pos[1] + draw_data.display_size[1];
        let matrix = [
            [(2.0 / (right - left)), 0.0, 0.0, 0.0],
            [0.0, (2.0 / (top - bottom)), 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [
                (right + left) / (left - right),
                (top + bottom) / (bottom - top),
                0.0,
                1.0,
            ],
        ];

        let uniform = ImguiUniform { matrix };
        let clip_off = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imgui render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        for draw_list in draw_data.draw_lists() {
            let vtx_list = draw_list.transmute_vtx_buffer::<ImguiVertex>();

            let buffer_descriptor = wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vtx_list),
                usage: wgpu::BufferUsage::VERTEX,
            };

            let vtx_buffer = device.create_buffer_init(&buffer_descriptor);

            let buffer_descriptor = wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(draw_list.idx_buffer()),
                usage: wgpu::BufferUsage::INDEX,
            };

            let index_buffer = device.create_buffer_init(&buffer_descriptor);

            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements {
                        count,
                        cmd_params:
                            imgui::DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                                ..
                            },
                    } => {
                        let vtx_offset = vtx_offset as wgpu::BufferAddress;
                        let idx_offset = idx_offset as wgpu::BufferAddress;
                        let count = count as wgpu::BufferAddress;

                        let clip_rect = [
                            (clip_rect[0] - clip_off[0]) * clip_scale[0],
                            (clip_rect[1] - clip_off[1]) * clip_scale[1],
                            (clip_rect[2] - clip_off[0]) * clip_scale[0],
                            (clip_rect[3] - clip_off[1]) * clip_scale[1],
                        ];

                        if clip_rect[0] < fb_width
                            && clip_rect[1] < fb_height
                            && clip_rect[2] >= 0.0
                            && clip_rect[3] >= 0.0
                        {
                            let scissors = (
                                clip_rect[0] as u32,
                                clip_rect[1] as u32,
                                (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                                (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                            );

                            let texture = self.lookup_texture(texture_id).unwrap();
                            render_pass.set_bind_group(0, &texture.bind_group, &[]);

                            render_pass.set_vertex_buffer(0, vtx_buffer.slice(vtx_offset..));
                            render_pass.set_index_buffer(
                                index_buffer.slice(idx_offset..(idx_offset + count)),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass
                                .set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);
                            render_pass.draw_indexed(0..count, 0, 0..1);
                        }
                    }
                    DrawCmd::ResetRenderState => (),
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd);
                    },
                }
            }
        }
    }
}
