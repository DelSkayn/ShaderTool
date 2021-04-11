use bytemuck::Zeroable;
use imgui::{internal::RawWrapper, DrawCmd, Textures};
use std::mem;
use wgpu::{util::DeviceExt, Buffer, Device, Queue, Texture};
use super::Renderer;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ImguiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    col: [u8; 4],
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

pub struct TextureData{
    texture: Texture,
    bind_group: wgpu::BindGroup,
}

pub struct ImguiRenderer {
    textures: Textures<TextureData>,

    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    vtx_buffer: Vec<Buffer>,
    idx_buffer: Vec<Buffer>,
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
                buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<ImguiVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Float2, 2=> Uchar4Norm],
                }]
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: swapchain.format,
                    alpha_blend: wgpu::BlendState{
                        src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    color_blend: wgpu::BlendState{
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState{
                format: Renderer::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false
            }),
            multisample: wgpu::MultisampleState::default(),
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

        let mut res = ImguiRenderer {
            textures: Textures::new(),
            render_pipeline,
            texture_bind_group_layout,
            uniform_bind_group,
            uniform_buffer,
            sampler,
            vtx_buffer: Vec::new(),
            idx_buffer: Vec::new(),
        };

        res.upload_font_texture(ctx.fonts(), &device, &queue);

        res
    }

    fn upload_font_texture(
        &mut self,
        mut fonts: imgui::FontAtlasRefMut,
        device: &Device,
        queue: &Queue,
    ) {
        self.textures.remove(fonts.tex_id);
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
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        };

        let texture = device.create_texture_with_data(queue, &disc, font_texture.data);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let id = self.textures.insert(TextureData {
            texture,
            bind_group: group,
        });
        fonts.tex_id = id;
    }

    pub fn render_imgui<'a>(
        &'a mut self,
        draw_data: &imgui::DrawData,
        device: &Device,
        queue: &Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
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

        /*
        let width = draw_data.display_size[0];
        let height = draw_data.display_size[1];

        let offset_x = draw_data.display_pos[0] / width;
        let offset_y = draw_data.display_pos[1] / height;

        let matrix = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0 - offset_x * 2.0, 1.0 + offset_y + 2.0, 0.0, 1.0],
        ];
        */

        let uniform = ImguiUniform { matrix };
        let clip_off = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        self.vtx_buffer.clear();
        self.idx_buffer.clear();

        for draw_list in draw_data.draw_lists() {
            let vtx_list = unsafe { draw_list.transmute_vtx_buffer::<ImguiVertex>() };

            let buffer_descriptor = wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vtx_list),
                usage: wgpu::BufferUsage::VERTEX,
            };

            self.vtx_buffer
                .push(device.create_buffer_init(&buffer_descriptor));

            let buffer_descriptor = wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(draw_list.idx_buffer()),
                usage: wgpu::BufferUsage::INDEX,
            };

            self.idx_buffer
                .push(device.create_buffer_init(&buffer_descriptor));
        }

        for (idx, draw_list) in draw_data.draw_lists().enumerate() {
            let vtx_buffer = &self.vtx_buffer[idx];
            let idx_buffer = &self.idx_buffer[idx];

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
                        let vtx_offset =
                            (vtx_offset * mem::size_of::<ImguiVertex>()) as wgpu::BufferAddress;

                        let clip_rect = [
                            (clip_rect[0] - clip_off[0]) * clip_scale[0],
                            (clip_rect[1] - clip_off[1]) * clip_scale[1],
                            (clip_rect[2] - clip_off[0]) * clip_scale[0],
                            (clip_rect[3] - clip_off[1]) * clip_scale[1],
                        ];

                        let scissors = (
                            clip_rect[0].max(0.0).floor() as u32,
                            clip_rect[1].max(0.0).floor() as u32,
                            (clip_rect[2] - clip_rect[0]).abs().ceil() as u32,
                            (clip_rect[3] - clip_rect[1]).abs().ceil() as u32,
                        );

                        let texture = self.textures.get(texture_id).unwrap();
                        render_pass.set_bind_group(0, &texture.bind_group, &[]);

                        render_pass.set_vertex_buffer(0, vtx_buffer.slice(vtx_offset..));
                        render_pass
                            .set_index_buffer(idx_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass
                            .set_scissor_rect(scissors.0, scissors.1, scissors.2, scissors.3);
                        let range = idx_offset as u32..(idx_offset + count) as u32;
                        render_pass.draw_indexed(range, 0, 0..1);
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
