use glutin::dpi::LogicalSize;
use glutin::event_loop::EventLoop;
use glutin::platform::unix::HeadlessContextExt;
use glutin::platform::desktop::EventLoopExtDesktop;
use glutin::window::WindowBuilder;
use glutin::{Context, PossiblyCurrent, WindowedContext};
use grr::Device;
use std::error::Error;

/// Single window with an OpenGL / `grr` device and event loop and OpenGL
/// debugging turned on.
pub struct GrrWindow {
    pub window: WindowedContext<PossiblyCurrent>,
    pub event_loop: EventLoop<()>,
    pub device: Device,
}

impl GrrWindow {
    /// Return the individual components used to construct the window.
    ///
    /// Unnecessary with winit 0.22+ and the use of run_return.
    pub fn drain(self) -> (WindowedContext<PossiblyCurrent>, EventLoop<()>, Device) {
        (self.window, self.event_loop, self.device)
    }
}

pub struct GrrImgui {
    last_frame: std::time::Instant,
    pub imgui_context: imgui::Context,
    pub imgui_platform: imgui_winit_support::WinitPlatform,
}

impl<'d> GrrImgui {
    pub fn new(w: &GrrWindow) -> grr::Result<GrrImgui> {
	let mut imgui_context = imgui::Context::create();
	imgui_context.set_ini_filename(None);

	let imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

	let hidpi_factor = w.window.window().scale_factor();
	let font_size = (13.0 * hidpi_factor) as f32;

	imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
		config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..imgui::FontConfig::default()
		}),
            }]);

	Ok(GrrImgui {
	    last_frame: std::time::Instant::now(),
	    imgui_context,
	    imgui_platform,
	})
    }

    pub fn renderer<'a>(&mut self, device: &'a grr::Device) -> grr::Result<grr_imgui::Renderer<'a>> {
	unsafe {grr_imgui::Renderer::new(&mut self.imgui_context, device) }
    }

    /// Return the UI for the current frame
    pub fn ui(&mut self) -> imgui::Ui {
	self.imgui_context.frame()
    }

    /// Should be called on `glutin::event::MainEventsCleared` events.
    pub fn on_events_cleared(&mut self, w: &WindowedContext<PossiblyCurrent>) {
	self.imgui_platform.prepare_frame(self.imgui_context.io_mut(), &w.window()).ok();
    }

    /// Should be called on `glutin::event::Event::NewEvents(_)` events.
    pub fn on_new_events(&mut self) {
	self.last_frame = self.imgui_context.io_mut().update_delta_time(self.last_frame);
    }

    /// Should be called on every event.
    pub fn on_event(&mut self, event: &glutin::event::Event<()>, w: &WindowedContext<PossiblyCurrent>) {
	self.imgui_platform.handle_event(self.imgui_context.io_mut(), &w.window(), event);
    }

}

/// Headless OpenGL context.
pub struct GrrHeadless {
    pub window: Context<PossiblyCurrent>,
    pub event_loop: EventLoop<()>,
    pub device: Device,
}

impl GrrHeadless {
    /// Return the individual components used to construct the window.
    ///
    /// Unnecessary with winit 0.22+ and the use of run_return.
    pub fn drain(self) -> (Context<PossiblyCurrent>, EventLoop<()>, Device) {
        (self.window, self.event_loop, self.device)
    }
}

pub struct GrrBuilder {
    resizable: bool,
    samples: Option<u16>,
    vsync: bool,
    srgb: bool,
    gl_debug: Option<grr::DebugReport>,
}

impl GrrBuilder {
    pub fn new() -> GrrBuilder {
        GrrBuilder {
            resizable: true,
            samples: None,
            vsync: false,
            srgb: false,
            gl_debug: Some(
                grr::DebugReport::WARNING
                    | grr::DebugReport::ERROR
                    | grr::DebugReport::PERFORMANCE_WARNING,
            ),
        }
    }

    pub fn resizeable(self, b: bool) -> GrrBuilder {
        GrrBuilder {
            resizable: b,
            ..self
        }
    }
    pub fn multisamples(self, ns: impl Into<Option<u16>>) -> GrrBuilder {
        GrrBuilder {
            samples: ns.into(),
            ..self
        }
    }
    pub fn vsync(self, v: bool) -> GrrBuilder {
        GrrBuilder { vsync: v, ..self }
    }
    pub fn srgb(self, s: bool) -> GrrBuilder {
        GrrBuilder { srgb: s, ..self }
    }
    pub fn gl_debug(self, d: impl Into<Option<grr::DebugReport>>) -> GrrBuilder {
        GrrBuilder {
            gl_debug: d.into(),
            ..self
        }
    }

    fn debug(&self) -> grr::Debug<grr::DebugCallback> {
        if let Some(flags) = self.gl_debug {
            grr::Debug::Enable {
                callback: |report, source, dtype, id, msg| {
                    println!(
                        "{:8} {:?} ({:?}/{:?}): {:?}",
                        id, report, source, dtype, msg
                    );
                },
                flags: flags,
            }
        } else {
            grr::Debug::Disable
        }
    }

    pub fn build_windowed(self, title: &str, w: f32, h: f32) -> Result<GrrWindow, Box<dyn Error>> {
        let event_loop = EventLoop::new();
        let wb = WindowBuilder::new()
            .with_title(title)
            .with_resizable(self.resizable)
            .with_inner_size(LogicalSize {
                width: w,
                height: h,
            });
        let window = unsafe {
            let mut cx = glutin::ContextBuilder::new()
                .with_vsync(self.vsync)
                .with_srgb(self.srgb)
                .with_gl_debug_flag(self.gl_debug.is_some());

            if let Some(ms) = self.samples {
                if ms > 0 {
                    cx = cx.with_multisampling(ms as u16);
                }
            }
            cx.build_windowed(wb, &event_loop)?.make_current().unwrap()
        };

        let device = unsafe {
            Device::new(
                |symbol| window.get_proc_address(symbol) as *const _,
                self.debug(),
            )
        };


        Ok(GrrWindow {
            window,
            event_loop,
            device,
        })
    }

    pub fn build_headless(self) -> Result<GrrHeadless, Box<dyn Error>> {
        let event_loop = EventLoop::new();
        let window = unsafe {
            glutin::ContextBuilder::new()
                .with_vsync(self.vsync)
                .with_srgb(self.srgb)
                .with_gl_debug_flag(self.gl_debug.is_some())
                .build_surfaceless(&event_loop)?
                .make_current()
                .unwrap()
        };

        let device = unsafe {
            Device::new(
                |symbol| window.get_proc_address(symbol) as *const _,
                self.debug(),
            )
        };

        Ok(GrrHeadless {
            window,
            event_loop,
            device,
        })
    }
}
