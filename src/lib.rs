use std::sync::atomic::Ordering;

use anyhow::Result;

use ash::vk;

use game::Game;
use wgpu_types as wgt;

use openxr as xr;

mod game;
mod math;
mod shell;
use shell::{PollStatus, XrShell};

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

struct App<G: Game> {
    xr_shell: XrShell,
    game: G,
}

impl<G: Game> App<G> {
    fn new() -> Result<Self> {
        let vk_target_version = vk::make_api_version(0, 1, 1, 0); // Vulkan 1.1 guarantees multiview support

        let features = wgpu::Features::SPIRV_SHADER_PASSTHROUGH | wgt::Features::MULTIVIEW;
        let limits = wgt::Limits::default();

        let xr_shell = XrShell::new("OpenXR Wgpu", 1, vk_target_version, features, limits)?;
        let game = G::init(&xr_shell)?;

        Ok(Self {
            xr_shell,
            game,
        })
    }

    pub fn poll_events(&mut self) -> Result<PollStatus> {
        self.xr_shell.poll_events()
    }

    pub fn frame_update(&mut self) -> Result<()> {
        // Block until the previous frame is finished displaying, and is ready for another one.
        // Also returns a prediction of when the next frame will be displayed, for use with
        // predicting locations of controllers, viewpoints, etc.
        let frame_state = self.xr_shell.xr_frame_waiter.wait()?;

        self.game.tick_to(&self.xr_shell, frame_state.predicted_display_time);

        // Spec: "An application must eventually match each xrWaitFrame call with one call to xrBeginFrame"
        self.xr_shell.xr_frame_stream.begin()?;

        // Id would be nice if we could consistently end() the frame here to be clear about
        // the 1:1 relationship begin() and end() but we can't (practically) return the
        // slice of layers from the render function and so we rely on render() calling
        // frame_stream.end() before it returns.

        let (rendered, render_status) = if frame_state.should_render {
            (true, self.render(frame_state))
        } else {
            (false, Ok(()))
        };

        // Spec: "Every application must call xrBeginFrame before calling xrEndFrame, and should call
        //  xrEndFrame before calling xrBeginFrame again."
        if !rendered || render_status.is_err() {
            self.xr_shell.xr_frame_stream.end(
                frame_state.predicted_display_time,
                self.xr_shell.xr_current_blend_mode,
                &[],
            )?;
        };

        render_status
    }

    pub fn render(&mut self, frame_state: xr::FrameState) -> Result<()> {
        log::info!("Render");
        debug_assert!(frame_state.should_render);

        // We need to ask which swapchain image to use for rendering! Which one will we get?
        // Who knows! It's up to the runtime to decide.
        let image_index = self
            .xr_shell
            .xr_swapchain
            .handle
            .lock()
            .unwrap()
            .acquire_image()?;

        // Wait until the image is available to render to. The compositor could still be
        // reading from it.
        self.xr_shell
            .xr_swapchain
            .handle
            .lock()
            .unwrap()
            .wait_image(xr::Duration::INFINITE)?;

        let command_buffers = self.game.prepare_render(
            &self.xr_shell,
            &self.xr_shell.xr_swapchain.buffers[image_index as usize].color,
        )?;

        // Fetch the view transforms. To minimize latency, we intentionally do this *after*
        // recording commands to render the scene, i.e. at the last possible moment before
        // rendering begins in earnest on the GPU. Uniforms dependent on this data can be sent
        // to the GPU just-in-time by writing them to per-frame host-visible memory which the
        // GPU will only read once the command buffer is submitted.
        let (view_flags, views) = self.xr_shell.xr_session.locate_views(
            XrShell::VIEW_TYPE,
            frame_state.predicted_display_time,
            self.game.xr_stage(),
        )?;

        self.game.load_view_transforms(&self.xr_shell, view_flags, &views)?;

        self.xr_shell.wgpu_queue.submit(command_buffers);

        self.xr_shell
            .xr_swapchain
            .handle
            .lock()
            .unwrap()
            .release_image()?;

        // Tell OpenXR what to present for this frame
        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di {
                width: self.xr_shell.xr_swapchain.resolution.width as _,
                height: self.xr_shell.xr_swapchain.resolution.height as _,
            },
        };

        let swapchain = &self.xr_shell.xr_swapchain.handle.lock().unwrap();

        self.xr_shell.xr_frame_stream.end(
            frame_state.predicted_display_time,
            self.xr_shell.xr_current_blend_mode,
            &[&xr::CompositionLayerProjection::new()
                .space(self.game.xr_stage())
                .views(&[
                    // TODO use a custom Space here for world-space stuff instead of locking to camera view.
                    // This information may be used for reprojection.
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[0].pose)
                        .fov(views[0].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(swapchain)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[1].pose)
                        .fov(views[1].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(swapchain)
                                .image_array_index(1)
                                .image_rect(rect),
                        ),
                ])],
        )?;

        Ok(())
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Trace));

    let mut app = App::<game::RectViewer>::new().unwrap();

    log::trace!("Running mainloop...");
    'mainloop: loop {
        android_app.poll_events(Some(Duration::from_secs(0)), |event| {
            log::info!("Android event {:?}", event);
        });

        let status = app.poll_events().unwrap();

        if status.contains(PollStatus::QUIT) {
            log::info!("Mainloop Quitting");
            break 'mainloop;
        }

        if status.contains(PollStatus::FRAME) {
            app.frame_update().unwrap();
        }
    }
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    let mut app = App::<game::RectViewer>::new().unwrap();

    let r = app.xr_shell.quit_signal.clone();
    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    });

    log::trace!("Running mainloop...");
    'mainloop: loop {
        let status = app.poll_events()?;

        if status.contains(PollStatus::QUIT) {
            log::info!("Mainloop Quitting");
            break 'mainloop;
        }

        if status.contains(PollStatus::FRAME) {
            app.frame_update()?;
        }
    }

    Ok(())
}
