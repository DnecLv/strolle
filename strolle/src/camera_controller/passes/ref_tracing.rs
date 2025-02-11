use rand::Rng;

use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct RefTracingPass {
    pass: CameraComputePass<gpu::RefPassParams>,
}

impl RefTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("reference_tracing")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.ref_rays.bind_readable(),
                &buffers.ref_hits.bind_writable(),
            ])
            .build(device, &engine.shaders.ref_tracing);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        depth: u8,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        let params = gpu::RefPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
            depth: depth as u32,
        };

        self.pass.run(camera, encoder, size, params);
    }
}
