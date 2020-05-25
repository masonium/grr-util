use glutin::dpi::LogicalSize;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::{PossiblyCurrent, WindowedContext};
use grr::Device;
use std::error::Error;

pub struct GrrWindow {
    pub window: WindowedContext<PossiblyCurrent>,
    pub event_loop: EventLoop<()>,
    pub device: Device,
}

impl GrrWindow {
    pub fn build_window(
        w: f32,
        h: f32,
    ) -> Result<GrrWindow, Box<dyn Error>> {
        let event_loop = EventLoop::new();
        let wb = WindowBuilder::new()
            .with_title("grr - demo")
            .with_inner_size(LogicalSize {
                width: w,
                height: h,
            });
        let window = unsafe {
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_srgb(false)
                .with_gl_debug_flag(true)
                .build_windowed(wb, &event_loop)?
                .make_current()
                .unwrap()
        };

        let device = unsafe {
            Device::new(
                |symbol| window.get_proc_address(symbol) as *const _,
                grr::Debug::Enable {
                    callback: |report, source, dtype, id, msg| {
                        println!(
                            "{:8} {:?} ({:?}/{:?}): {:?}",
                            id, report, source, dtype, msg
                        );
                    },
                    flags: grr::DebugReport::FULL,
                },
            )
        };

        unsafe {
            device.disable_debug_message(
                grr::MsgFilter::All,
                grr::MsgFilter::All,
                grr::DebugReport::NOTIFICATION,
                None,
            );
        }

        Ok(GrrWindow{ window, event_loop, device })
    }

    pub fn drain(self) -> (WindowedContext<PossiblyCurrent>,
                       EventLoop<()>,
                       Device) {
        (self.window, self.event_loop, self.device)
    }
}
