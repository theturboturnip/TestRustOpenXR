use std::{marker::PhantomData, num::{NonZero, NonZeroU32}};

use crate::{controls::{Controls, PointAndClickControls}, math::Mat4, shell::XrShell, spv_shader_bytes, xr};

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
struct TimeTracker {
    last_time: Option<xr::Time>,
    elapsed_ns: i64,
}
enum TimeDelta {
    FirstFrame,
    Delta { nanos: i64, secs: f32 },
}
impl TimeTracker {
    /// If the time is greater than 1/2 a second, only step the time by [DEF_DELTA_NANOS].
    pub const MAX_DELTA_NANOS: i64 = 500_000_000;
    /// If something goes wrong, use 1/60th of a second as the default step
    pub const DEF_DELTA_NANOS: i64 = 16_666_666;

    fn real_time_nanos(&self) -> i64 {
        self.elapsed_ns
    }
    fn real_time_secs(&self) -> f64 {
        (self.elapsed_ns as f64) / 1e9
    }

    fn delta(&mut self, predicted_display_time: xr::Time) -> TimeDelta {
        let last_time = self.last_time;
        self.last_time = Some(predicted_display_time);
        match last_time {
            None => TimeDelta::FirstFrame,
            Some(last_time) => {
                let mut nanos = predicted_display_time.as_nanos().wrapping_sub(last_time.as_nanos());
                if nanos < 0 || nanos > Self::MAX_DELTA_NANOS {
                    nanos = Self::DEF_DELTA_NANOS;
                }
                self.elapsed_ns = match self.elapsed_ns.checked_add(nanos) {
                    None => {
                        // Overflow happened! No good way to handle this...
                        0
                    }
                    Some(next) => next
                };
                // nanos will never be >500_000_000
                // => will always fit in a f32
                TimeDelta::Delta { nanos, secs: (nanos as f32) / 1e9  }
            }
        }
    }
}
impl Default for TimeTracker {
    fn default() -> Self {
        Self {
            last_time: None,
            elapsed_ns: 0,
        }
    }
}

pub(crate) trait Game: Sized {
    fn init(xr_shell: &XrShell) -> Result<Self>;

    // Getter
    fn xr_stage(&self) -> &xr::Space;

    /// Advance the game state to the predicted time
    /// TODO pull TimeTracker out of this and into App, just push TimeDelta into tick_to
    fn tick_to(&mut self, xr_shell: &XrShell, predicted_display_time: xr::Time);

    /// Record the command buffers for rendering, and return them for submission.
    /// Command buffers that don't depend on the view transforms can and should be submitted early, not returned.
    /// The command buffers that *are* returned will not be submitted immediately - [Game::load_view_transforms] will be called first.
    /// This allows the final command buffer to be submitted as close to the point we receive the estimated head positions as possible.
    /// 
    /// TODO we may want to be able to present OpenXR with different composition layers - how to do that?
    /// Right now we separate render() and load_view_transforms() because the final composition layers need the views too,
    /// and I don't want to return them from prepare_render()...
    type CommandBuffers: IntoIterator<Item = wgpu::CommandBuffer>;
    fn prepare_render(&mut self, xr_shell: &XrShell, target_render_view: &wgpu::TextureView) -> Result<Self::CommandBuffers>;

    fn load_view_transforms(&mut self, xr_shell: &XrShell, view_flags: xr::ViewStateFlags, views: &[xr::View]) -> Result<()>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Eyes {
    eye_screen_from_world: [Mat4; 2],
}
const _: () = assert!(std::mem::size_of::<Eyes>() == 128);
impl Default for Eyes {
    fn default() -> Self {
        Self { eye_screen_from_world: [Mat4::zero(); 2] }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PerObject {
    world_from_model: Mat4,
}
const _: () = assert!(std::mem::size_of::<PerObject>() == 64);

struct UniformBuffer<T: bytemuck::Pod + bytemuck::Zeroable + Sized> {
    buffer: wgpu::Buffer,
    _t: PhantomData<T>,
}
impl<T: bytemuck::Pod + bytemuck::Zeroable + Sized> UniformBuffer<T> {
    const _CHECK_SIZE: () = assert!(std::mem::size_of::<T>() > 0);

    fn create(xr_shell: &XrShell) -> Self {
        let buffer = xr_shell.wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<T>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        Self { 
            buffer,
            _t: PhantomData::default(),
        }
    }

    fn overwrite(&self, xr_shell: &XrShell, value: &T) -> Result<()> {
        match xr_shell.wgpu_queue.write_buffer_with(&self.buffer, 0, NonZero::new(std::mem::size_of::<T>() as u64).unwrap()) {
            Some(mut buf) => {
                let bytes = bytemuck::bytes_of(value);
                buf.as_mut().copy_from_slice(bytes);
                Ok(())
            }
            None => anyhow::bail!("Couldn't write uniform buffer"),
        }
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}


/// All meshes right now are rendered with the same shader, which hardcodes a quad
struct Quad {
    per_object_uniforms: UniformBuffer<PerObject>,
    bindings: wgpu::BindGroup,
}

impl Quad {
    fn new(xr_shell: &XrShell, bind_group_layout: &wgpu::BindGroupLayout, eye_uniform_buffer: &wgpu::Buffer) -> Self {
        let per_object_uniforms = UniformBuffer::create(xr_shell);
        Self {
            bindings: xr_shell.wgpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: eye_uniform_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: per_object_uniforms.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            }),
            per_object_uniforms,
        }
    }

    fn update_uniforms(&self, xr_shell: &XrShell, world_from_model: Mat4) -> Result<()> {
        self.per_object_uniforms.overwrite(xr_shell, &PerObject {
            world_from_model
        })
    }

    fn enqueue_draw(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(0, &self.bindings, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

pub(crate) struct RectViewer {
    time: TimeTracker,
    delta_real_time: f32,

    xr_stage: xr::Space,

    controls: PointAndClickControls,
    
    wgpu_render_pipeline: wgpu::RenderPipeline,
    eye_uniform_buffer: UniformBuffer<Eyes>,
    meshes: [Quad; 3],
}
impl Game for RectViewer {
    fn init(xr_shell: &XrShell) -> Result<Self> {
        let vertex_shader = xr_shell.compile_spv(&spv_shader_bytes!("fullscreen.vert"))?;
        let fragment_shader = xr_shell.compile_spv(&spv_shader_bytes!("debug_pattern.frag"))?;

        let bind_group_layout = xr_shell.wgpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });

        let pipeline_layout =
            xr_shell
                .wgpu_device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let wgpu_render_pipeline =
            xr_shell
                .wgpu_device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    cache: None, // TODO caching
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vertex_shader,
                        entry_point: "main",
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0x0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_shader,
                        entry_point: "main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            blend: None,
                            write_mask: wgpu::ColorWrites::RED
                                | wgpu::ColorWrites::GREEN
                                | wgpu::ColorWrites::BLUE,
                        })],
                        compilation_options: Default::default(),
                    }),
                    // Render to both eyes in multipass
                    multiview: Some(NonZeroU32::new(2).unwrap()),
                });

        let eye_uniform_buffer = UniformBuffer::create(xr_shell);

        let meshes = [
            Quad::new(xr_shell, &bind_group_layout, eye_uniform_buffer.buffer()),
            Quad::new(xr_shell, &bind_group_layout, eye_uniform_buffer.buffer()),
            Quad::new(xr_shell, &bind_group_layout, eye_uniform_buffer.buffer()),
        ];
        meshes[0].update_uniforms(xr_shell, Mat4::identity())?;

        let controls = PointAndClickControls::new(
            xr_shell, "point_and_click", "Point & Click"
        )?;

        // Bind our actions to input devices using the given profile
        // If you want to access inputs specific to a particular device you may specify a different
        // interaction profile
        for interaction_binding in controls.suggested_bindings(&xr_shell.xr_instance)? {
            xr_shell
                .xr_instance
                .suggest_interaction_profile_bindings(
                    xr_shell
                        .xr_instance
                        .string_to_path(interaction_binding.0)?,
                    &interaction_binding.1
                )?;
        }

        // Attach the action set to the session
        xr_shell
            .xr_session
            .attach_action_sets(&[&controls.action_set()])?;

        // OpenXR uses a couple different types of reference frames for positioning content; we need
        // to choose one for displaying our content! STAGE would be relative to the center of your
        // guardian system's bounds, and LOCAL would be relative to your device's starting location.
        let xr_stage = xr_shell
            .xr_session
            .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)?;

        Ok(Self {
            time: Default::default(),
            delta_real_time: 0.0,

            controls,
            xr_stage,
        
            wgpu_render_pipeline,
            eye_uniform_buffer,
            meshes,
        })
    }

    fn tick_to(&mut self, xr_shell: &XrShell, predicted_display_time: openxr::Time) {
        let delta = self.time.delta(predicted_display_time);
        self.delta_real_time = match delta {
            TimeDelta::FirstFrame => 0.0,
            TimeDelta::Delta { secs, .. } => secs,
        };
        log::info!("delta_time: {}", self.delta_real_time);

        xr_shell
            .xr_session
            .sync_actions(&[self.controls.action_set().into()])
            .unwrap();

        // Find where our controllers are located in the Stage space
        let inputs = self.controls.locate(xr_shell, &self.xr_stage, predicted_display_time).unwrap();

        // let mut printed = false;
        if let Some(lh) = inputs.lh {
            self.meshes[1].update_uniforms(xr_shell, lh.point.into()).unwrap();
            // print!(
            //     "Left Hand: ({:0<12},{:0<12},{:0<12}), ",
            //     lh.point.position.0[0],
            //     lh.point.position.0[1],
            //     lh.point.position.0[2]
            // );
            // printed = true;
        }

        if let Some(rh) = inputs.rh {
            self.meshes[2].update_uniforms(xr_shell, rh.point.into()).unwrap();
            // print!(
            //     "Right Hand: ({:0<12},{:0<12},{:0<12})",
            //     rh.point.position.0[0],
            //     rh.point.position.0[1],
            //     rh.point.position.0[2]
            // );
            // printed = true;
        }
        // if printed {
        //     println!();
        // }
    }

    type CommandBuffers = [wgpu::CommandBuffer; 1];
    fn prepare_render(&mut self, xr_shell: &XrShell, target_render_view: &wgpu::TextureView) -> Result<Self::CommandBuffers> {
        let mut command_encoder = xr_shell
            .wgpu_device
            .create_command_encoder(&Default::default());

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.2 + (self.time.real_time_secs() as f64 % 0.8),
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_viewport(
                0_f32,
                0_f32,
                xr_shell.xr_swapchain.resolution.width as _,
                xr_shell.xr_swapchain.resolution.height as _,
                0_f32,
                1_f32,
            );
            render_pass.set_scissor_rect(
                0,
                0,
                xr_shell.xr_swapchain.resolution.width,
                xr_shell.xr_swapchain.resolution.height,
            );

            render_pass.set_pipeline(&self.wgpu_render_pipeline);
            for quad in self.meshes.iter() {
                quad.enqueue_draw(&mut render_pass);
            }
        }

        Ok([command_encoder.finish()])
    }

    fn load_view_transforms(&mut self, xr_shell: &XrShell, _view_flags: xr::ViewStateFlags, views: &[xr::View]) -> Result<()> {
        // Load the views into a uniform buffer

        const NEAR_Z: f32 = 0.01;
        const FAR_Z: f32 = 50.0;

        let mut matrices = Eyes::default();
        for (i, view) in views.iter().enumerate() {
            if i >= 2 {
                continue;
            }

            let screen_from_view = Mat4::xr_projection_fov(view.fov, NEAR_Z, FAR_Z);
            let world_from_view: Mat4 = view.pose.into();
            matrices.eye_screen_from_world[i] = screen_from_view * (world_from_view.inverse().unwrap());
        }

        self.eye_uniform_buffer.overwrite(xr_shell, &matrices)
    }

    fn xr_stage<'a>(&'a self) -> &'a openxr::Space {
        &self.xr_stage
    }
}