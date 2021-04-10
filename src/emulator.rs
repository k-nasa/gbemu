use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gpu::Gpu;
use crate::ram::Ram;
use crate::{SharedBus, SharedGpu};
use anyhow::Result;
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

pub struct Emulator {
    cpu: Cpu,
    gpu: SharedGpu,
}

impl Emulator {
    pub fn new(bus: SharedBus, gpu: SharedGpu) -> Self {
        Emulator {
            cpu: Cpu::new(bus),
            gpu,
        }
    }

    pub fn from_rom_byte(bytes: Vec<u8>) -> Emulator {
        // NOTE https://w.atwiki.jp/gbspec/pages/13.html サイズはこれを見て決めた
        let video_ram = Ram::with_size(0x2000);
        let h_ram = Ram::with_size(0x2000);
        let oam_ram = Ram::with_size(0x2000);
        let mirror_ram = Ram::with_size(0x2000);
        let working_ram = Ram::with_size(0x2000);
        let cartridge = Cartridge::new(bytes);
        let gpu = Gpu::new(1024, None); // TODO implement
        let gpu = Arc::new(Mutex::new(gpu));

        let bus = Bus::new(
            cartridge,
            video_ram,
            h_ram,
            oam_ram,
            mirror_ram,
            working_ram,
            gpu.clone(),
        );

        let bus = Arc::new(Mutex::new(bus));
        gpu.lock().unwrap().set_bus(bus.clone());

        Emulator::new(bus, gpu)
    }

    pub fn start(mut self) -> Result<()> {
        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            WindowBuilder::new()
                .with_title("gbemu")
                .with_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        event_loop.run(move |event, _, control_flow| {
            self.cpu.step().unwrap();

            let mut gpu = self.gpu.lock().unwrap();
            gpu.step();

            if let Event::RedrawRequested(_) = event {
                // world.draw(pixels.get_frame());
                pixels.render().unwrap();
            }

            if input.update(&event) {
                if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Some(size) = input.window_resized() {
                    pixels.resize(size.width, size.height);
                }

                window.request_redraw();
            }
        });
    }
}
