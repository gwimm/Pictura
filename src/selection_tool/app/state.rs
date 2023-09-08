use futures::core::mouse;
use iced::window::{self, Level};
use iced_wgpu::{
    wgpu::{self, Backends, Device, Instance, Queue, Surface, TextureFormat},
    Backend, Renderer,
};
use log::info;
use winit::{
    dpi::{LogicalPosition, PhysicalPosition, PhysicalSize},
    event::ModifiersState,
    event_loop::EventLoop,
    window::Window,
};

use iced_winit::{
    conversion::{self, window_event},
    futures,
    runtime::{program::State as IcedState, Debug},
    settings, Clipboard, Viewport,
};

use crate::selection_tool::theme::Theme;

use super::{App, Message};

pub struct MouseState {
    pub pressed: bool,
    pub was_pressed: bool,
    pub position: LogicalPosition<f64>,
}

pub struct State {
    pub iced_state: IcedState<App>,

    pub window: Window,
    viewport: Viewport,
    pub cursor_position: Option<PhysicalPosition<f64>>,
    clipboard: Clipboard,
    modifiers: ModifiersState,
    surface: Surface,
    debug: Debug,
    device: Device,
    renderer: Renderer<Theme>,
    queue: Queue,
    mouse_state: MouseState,
    tl: PhysicalPosition<f64>,
}

fn create_window<T>(
    event_loop: &EventLoop<T>,
    tl: PhysicalPosition<f64>,
    br: PhysicalPosition<f64>,
) -> Window {
    let win_window = settings::Window {
        resizable: false,
        decorations: false,
        position: iced_winit::Position::Specific(0i32, 0i32),
        visible: true,
        transparent: false,
        level: Level::AlwaysOnTop,
        icon: None,
        min_size: None,
        max_size: None,
        size: ((br.x - tl.x) as u32, (br.y - tl.y) as u32),
        platform_specific: window::PlatformSpecific::default(),
    };

    info!("Window Size: {:?}", win_window.size);
    info!("Window Location {:?}", win_window.position);

    let window = Window::new(&event_loop).unwrap();
    let monitor = window.primary_monitor();
    drop(window);

    let window = win_window
        .into_builder("Pictura", monitor, Some("Pictura".to_string()))
        .with_transparent(true)
        //.with_override_redirect(true)
        .build(&event_loop)
        .unwrap();
    window.set_outer_position(PhysicalPosition::new(tl.x, tl.y));
    window
}

/// "Figures out what settings to use for the GPU, for vulkan, dunno why.
/// I don't really know what this does, this is copy pasted from.. I got you though"
/// from tyttggfdsddgh on Discord.
async fn adapter(instance: &Instance, surface: &Surface) -> (TextureFormat, Device, Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let capabilities = surface.get_capabilities(&adapter);

    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(wgpu::TextureFormat::is_srgb)
        .or_else(|| capabilities.formats.first().copied())
        .expect("Get preferred format");

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();

    (format, device, queue)
}

impl State {
	  // Setup the state.
    pub fn setup<T>(
        event_loop: &EventLoop<T>,
        tl: PhysicalPosition<f64>,
        br: PhysicalPosition<f64>,
    ) -> State {
        let window = create_window(event_loop, tl, br);
        let physical_size = window.inner_size();
        let viewport = Self::create_viewport(&physical_size, &window);
				let backend = Self::create_backend();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let (format, device, queue) =
            futures::futures::executor::block_on(adapter(&instance, &surface));

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: physical_size.width,
                height: physical_size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );

        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(
            &device,
            &queue,
            iced_wgpu::Settings::default(),
            format,
        ));
				let clipboard = Clipboard::connect(&window);

        Self {
            iced_state: App::program_state(viewport.logical_size(), &mut renderer, &mut debug),

            mouse_state: MouseState {
                pressed: false,
                was_pressed: false,
                position: LogicalPosition::<f64>::new(0.0, 0.0),
            },

            window,
            viewport,
            cursor_position: None,
            clipboard,
            modifiers: Default::default(),
            surface,
            debug,
            device,
            renderer,
            queue,
            tl,
        }
    }

    pub fn release(&mut self) -> Option<(PhysicalPosition<f64>, PhysicalPosition<f64>)> {
        if self.is_mouse_pressed() {
            self.mouse_state.pressed = false;
            self.iced_state.queue_message(Message::OnMouseReleased);
            let pressed_pos = self.mouse_state.position;
            return Some((
                PhysicalPosition::new(pressed_pos.x + self.tl.x, pressed_pos.y + self.tl.y),
                PhysicalPosition::new(
                    self.cursor_position.unwrap().x + self.tl.x,
                    self.cursor_position.unwrap().y + self.tl.y,
                ),
            ));
        }
        None
    }

		/// Redraw iced also sets up a new frame for winit.
    pub fn request_redraw(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(error) => match error {
								// TODO: investigate whether or not all other errors are recoverable
                wgpu::SurfaceError::OutOfMemory => {
                    panic!("Swapchain error: {error}. Rendering cannot continue.")
                }
                _ => {
                    // Try rendering again next frame.
                    self.window.request_redraw();
										return
                }
            },
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        _ = self.iced_state.program();

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // And then iced on top
        self.renderer.with_primitives(|backend, primitive| {
            backend.present(
                &self.device,
                &self.queue,
                &mut encoder,
                None,
                &view,
                primitive,
                &self.viewport,
                &self.debug.overlay(),
            );
        });

        // Then we submit the work
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        // Update the mouse cursor
        self.window
            .set_cursor_icon(iced_winit::conversion::mouse_interaction(
                self.iced_state.mouse_interaction(),
            ));
    }

    /// We update iced (TODO: better doc comments)
    pub fn update_iced(&mut self) {
        let _ = self.iced_state.update(
            self.viewport.logical_size(),
            self.cursor_position
                .map(|p| conversion::cursor_position(p, self.viewport.scale_factor()))
                .map(mouse::Cursor::Available)
                .unwrap_or(mouse::Cursor::Unavailable),
            &mut self.renderer,
            &Theme,
            &iced_winit::core::renderer::Style {
                text_color: iced_winit::core::Color::WHITE,
            },
            &mut self.clipboard,
            &mut self.debug,
        );

        // and request a redraw
        self.window.request_redraw();
    }

    pub fn map_to_iced(&mut self, event: &winit::event::WindowEvent<'_>) {
        if let Some(event) = window_event(event, self.window.scale_factor(), self.modifiers) {
            self.iced_state.queue_event(event);
        }
        self.window.request_redraw();
    }

    // TODO: throw error
    pub fn press(&mut self) {
        self.mouse_state.pressed = !self.is_mouse_pressed();
        self.mouse_state.was_pressed = true;
        self.mouse_state.position = self
            .cursor_position
            .unwrap()
            .to_logical(self.window.scale_factor());
        self.iced_state.queue_message(Message::OnMousePressed);
    }

    pub fn is_mouse_pressed(&self) -> bool {
        self.mouse_state.pressed
    }

    pub fn queue_message(&mut self, message: Message) {
        self.iced_state.queue_message(message);
    }

    fn create_backend() -> Backends {
        let default_backend = Backends::PRIMARY;
        wgpu::util::backend_bits_from_env().unwrap_or(default_backend)
    }

    fn create_viewport(physical_size: &PhysicalSize<u32>, window: &Window) -> Viewport {
        iced_winit::Viewport::with_physical_size(
            iced::Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        )
    }
}
