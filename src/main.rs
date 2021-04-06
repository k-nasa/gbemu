use gbemu::bus::Bus;
use gbemu::cartridge::Cartridge;
use gbemu::emulator::Emulator;
use gbemu::gpu::Gpu;
use gbemu::ram::Ram;
use log::info;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use anyhow::Result;

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.is_empty() {
        anyhow::bail!("Plese speficy filepath")
    }

    let filename = &args[1];

    info!("loading file {}", filename);
    let bytes = std::fs::read(filename).unwrap();

    // NOTE https://w.atwiki.jp/gbspec/pages/13.html サイズはこれを見て決めた
    let video_ram = Ram::with_size(0x2000);
    let h_ram = Ram::with_size(0x2000);
    let oam_ram = Ram::with_size(0x2000);
    let mirror_ram = Ram::with_size(0x2000);
    let working_ram = Ram::with_size(0x2000);
    let cartridge = Cartridge::new(bytes);
    let gpu = Gpu::new(1024, None); // TODO implement
    let gpu = Arc::new(Mutex::new(gpu));
    let gpucell = RefCell::new(gpu.clone());

    let bus = Bus::new(
        cartridge,
        video_ram,
        h_ram,
        oam_ram,
        mirror_ram,
        working_ram,
        gpucell,
    );

    let bus = Arc::new(Mutex::new(bus));
    gpu.lock().unwrap().set_bus(RefCell::new(bus.clone()));

    let emu = Emulator::new(bus.clone(), gpu.clone());

    info!("start emulator");
    emu.start()?;

    Ok(())
}
