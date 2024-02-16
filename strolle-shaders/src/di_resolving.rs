use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 1, uniform)] world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    atmosphere_transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 2)]
    atmosphere_transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)] atmosphere_sky_lut_tex: Tex,
    #[spirv(descriptor_set = 1, binding = 4)]
    atmosphere_sky_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    next_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 8, storage_buffer)]
    prev_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 9)] output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let lights = LightsView::new(lights);
    let atmosphere = Atmosphere::new(
        atmosphere_transmittance_lut_tex,
        atmosphere_transmittance_lut_sampler,
        atmosphere_sky_lut_tex,
        atmosphere_sky_lut_sampler,
    );

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    let res =
        DiReservoir::read(next_reservoirs, camera.screen_to_idx(screen_pos));

    let color = if hit.is_some() {
        lights.get(res.sample.light_id).radiance(hit) * res.w
    } else {
        atmosphere.sample(world.sun_direction(), hit.direction, 1.0)
    };

    unsafe {
        output.write(screen_pos, color.extend(0.0));
    }

    res.write(prev_reservoirs, screen_idx);
}
