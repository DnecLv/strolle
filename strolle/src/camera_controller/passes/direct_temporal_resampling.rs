use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectTemporalResamplingPass {
    pass: CameraComputePass,
}

impl DirectTemporalResamplingPass {
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
        let pass = CameraComputePass::builder("direct_temporal_resampling")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.lights.bind_readable(),
                &engine.images.bind_atlas(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.direct_candidates.bind_readable(),
                &buffers.direct_prev_reservoirs.bind_readable(),
                &buffers.direct_curr_reservoirs.bind_writable(),
            ])
            .build(device, &engine.shaders.direct_temporal_resampling);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        self.pass.run(camera, encoder, size, &camera.pass_params());
    }
}
