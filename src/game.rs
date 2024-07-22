use std::{io::Cursor, num::{NonZero, NonZeroU32}};

use ash::util::read_spv;

use crate::{math::Mat4, xr, XrShell};

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
struct Matrices {
    eye_screen_from_world: [Mat4; 2],
}
impl Default for Matrices {
    fn default() -> Self {
        Self { eye_screen_from_world: [Mat4::zero(); 2] }
    }
}

pub(crate) struct RectViewer {
    time: TimeTracker,
    delta_real_time: f32,

    xr_action_set: xr::ActionSet,
    xr_left_action: xr::Action<xr::Posef>,
    xr_right_action: xr::Action<xr::Posef>,
    xr_left_space: xr::Space,
    xr_right_space: xr::Space,
    xr_stage: xr::Space,

    wgpu_render_pipeline: wgpu::RenderPipeline,
    // TODO we need more uniform buffers!
    wgpu_uniform_buffer: wgpu::Buffer,
    wgpu_uniform_buffer_bind_group: wgpu::BindGroup,
}
impl Game for RectViewer {
    fn init(xr_shell: &XrShell) -> Result<Self> {
        let vertex_shader = unsafe {
            xr_shell
                .wgpu_device
                .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: None,
                    source: read_spv(&mut Cursor::new(&include_bytes!("fullscreen.vert.spv")[..]))?
                        .into(),
                })
        };
        let fragment_shader = unsafe {
            xr_shell
                .wgpu_device
                .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: None,
                    source: read_spv(&mut Cursor::new(
                        &include_bytes!("debug_pattern.frag.spv")[..],
                    ))?
                    .into(),
                })
        };

        let bind_group_layout = xr_shell.wgpu_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
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

        let wgpu_uniform_buffer = xr_shell.wgpu_device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Matrices>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let wgpu_uniform_buffer_bind_group = xr_shell.wgpu_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &wgpu_uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                }
            ],
        });

        // Create an action set to encapsulate our actions
        let xr_action_set =
            xr_shell
                .xr_instance
                .create_action_set("input", "input pose information", 0)?;

        let xr_right_action =
            xr_action_set.create_action::<xr::Posef>("right_hand", "Right Hand Controller", &[])?;
        let xr_left_action =
            xr_action_set.create_action::<xr::Posef>("left_hand", "Left Hand Controller", &[])?;

        // Bind our actions to input devices using the given profile
        // If you want to access inputs specific to a particular device you may specify a different
        // interaction profile
        xr_shell
            .xr_instance
            .suggest_interaction_profile_bindings(
                xr_shell
                    .xr_instance
                    .string_to_path("/interaction_profiles/khr/simple_controller")?,
                &[
                    xr::Binding::new(
                        &xr_right_action,
                        xr_shell
                            .xr_instance
                            .string_to_path("/user/hand/right/input/grip/pose")?,
                    ),
                    xr::Binding::new(
                        &xr_left_action,
                        xr_shell
                            .xr_instance
                            .string_to_path("/user/hand/left/input/grip/pose")?,
                    ),
                ],
            )?;

        // Attach the action set to the session
        xr_shell
            .xr_session
            .attach_action_sets(&[&xr_action_set])
            .unwrap();

        // Create an action space for each device we want to locate
        let xr_right_space = xr_right_action.create_space(
            xr_shell.xr_session.clone(),
            xr::Path::NULL,
            xr::Posef::IDENTITY,
        )?;
        let xr_left_space = xr_left_action.create_space(
            xr_shell.xr_session.clone(),
            xr::Path::NULL,
            xr::Posef::IDENTITY,
        )?;

        // OpenXR uses a couple different types of reference frames for positioning content; we need
        // to choose one for displaying our content! STAGE would be relative to the center of your
        // guardian system's bounds, and LOCAL would be relative to your device's starting location.
        let xr_stage = xr_shell
            .xr_session
            .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)?;

        Ok(Self {
            time: Default::default(),
            delta_real_time: 0.0,

            xr_action_set,
            xr_left_action,
            xr_right_action,
            xr_left_space,
            xr_right_space,
            xr_stage,
        
            wgpu_render_pipeline,
            wgpu_uniform_buffer,
            wgpu_uniform_buffer_bind_group,
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
            .sync_actions(&[(&self.xr_action_set).into()])
            .unwrap();

        // Find where our controllers are located in the Stage space
        let right_location = self
            .xr_right_space
            .locate(&self.xr_stage, predicted_display_time)
            .unwrap();

        let left_location = self
            .xr_left_space
            .locate(&self.xr_stage, predicted_display_time)
            .unwrap();

        let mut printed = false;
        if self
            .xr_left_action
            .is_active(&xr_shell.xr_session, xr::Path::NULL)
            .unwrap()
        {
            print!(
                "Left Hand: ({:0<12},{:0<12},{:0<12}), ",
                left_location.pose.position.x,
                left_location.pose.position.y,
                left_location.pose.position.z
            );
            printed = true;
        }

        if self
            .xr_right_action
            .is_active(&xr_shell.xr_session, xr::Path::NULL)
            .unwrap()
        {
            print!(
                "Right Hand: ({:0<12},{:0<12},{:0<12})",
                right_location.pose.position.x,
                right_location.pose.position.y,
                right_location.pose.position.z
            );
            printed = true;
        }
        if printed {
            println!();
        }
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
            render_pass.set_bind_group(0, &self.wgpu_uniform_buffer_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        Ok([command_encoder.finish()])
    }

    fn load_view_transforms(&mut self, xr_shell: &XrShell, _view_flags: xr::ViewStateFlags, views: &[xr::View]) -> Result<()> {
        // Load the views into a uniform buffer

        const NEAR_Z: f32 = 0.01;
        const FAR_Z: f32 = 50.0;

        match xr_shell.wgpu_queue.write_buffer_with(&self.wgpu_uniform_buffer, 0, NonZero::new(std::mem::size_of::<Matrices>() as u64).unwrap()) {
            Some(mut buf) => {
                let mut matrices = Matrices::default();
                for (i, view) in views.iter().enumerate() {
                    if i >= 2 {
                        continue;
                    }

                    let screen_from_view = Mat4::xr_projection_fov(view.fov, NEAR_Z, FAR_Z);
                    let world_from_view: Mat4 = view.pose.into();
                    matrices.eye_screen_from_world[i] = screen_from_view * (world_from_view.inverse().unwrap());
                }

                let bytes = bytemuck::bytes_of(&matrices);
                buf.as_mut().copy_from_slice(bytes);
            }
            None => anyhow::bail!("Couldn't write uniform buffer"),
        }
        Ok(())
    }

    fn xr_stage<'a>(&'a self) -> &'a openxr::Space {
        &self.xr_stage
    }
}