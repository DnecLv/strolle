use std::mem;
use std::ops::Range;

use log::info;

use crate::{
    gpu, BindGroup, Camera, CameraBuffers, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DrawingPass {
    bg0: BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl DrawingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        config: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        info!("Initializing pass: drawing");

        let bg0 = BindGroup::builder("strolle_drawing_bg0")
            .add(&buffers.camera)
            .add(&buffers.directs.as_ro_sampled_bind())
            .add(&buffers.indirects.as_ro_sampled_bind())
            .add(&buffers.normals.as_ro_sampled_bind())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_drawing_pipeline_layout"),
                bind_group_layouts: &[bg0.as_ref()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::DrawingPassParams>() as u32,
                    },
                }],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_drawing_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.shaders.drawing,
                    entry_point: "main_vs",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.shaders.drawing,
                    entry_point: "main_fs",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.viewport.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self { bg0, pipeline }
    }

    pub fn run<P>(
        &self,
        camera: &CameraController<P>,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) where
        P: Params,
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_drawing_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let params = gpu::DrawingPassParams {
            viewport_mode: camera.camera.mode.serialize(),
        };

        pass.set_scissor_rect(
            camera.camera.viewport.position.x,
            camera.camera.viewport.position.y,
            camera.camera.viewport.size.x,
            camera.camera.viewport.size.y,
        );
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.as_ref(), &[]);
        pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::bytes_of(&params),
        );
        pass.draw(0..3, 0..1);
    }
}
