use ::imgui::{Context,DrawData};
use anyhow::Result;
use winit::window::Window;

mod imgui;


pub struct Texture{
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    depth: Texture,

    imgui_renderer: imgui::ImguiRenderer,
}

impl Renderer {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    fn build_depth_texture(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Texture{
        let size = wgpu::Extent3d{
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor{
            label: Some("depth texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor{
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });
        Texture{
            texture,
            view,
            sampler,
        }
    }


    pub async fn new(window: &Window, imgui: &mut Context) -> Result<Self> {

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(anyhow::anyhow!("failed to create wgpu instance"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let size = window.inner_size();
        dbg!(size);

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };


        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let imgui_renderer = imgui::ImguiRenderer::new(imgui, &device, &queue, &sc_desc);

        let depth = Self::build_depth_texture(&device,&sc_desc);

        Ok(Renderer {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            depth,
            imgui_renderer,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>){
        dbg!(size);
        if self.sc_desc.width == size.width && self.sc_desc.height == size.height{
            return;
        }
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface,&self.sc_desc);
        self.depth = Self::build_depth_texture(&self.device,&self.sc_desc);
    }

    pub fn render(&mut self,draw_data: &DrawData) -> Result<()>{

        let frame = match self.swap_chain.get_current_frame(){
            Ok(x) => x,
            Err(_) => {
                return Ok(());
            }
        };

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor{
                    attachment: &self.depth.view,
                    depth_ops: Some(wgpu::Operations{
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.imgui_renderer.render_imgui(&draw_data,&self.device,&self.queue,&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
