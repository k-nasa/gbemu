use gbemu::bus::Bus;
use gbemu::cartridge::Cartridge;
use gbemu::emulator::Emulator;
use gbemu::gpu::Gpu;
use gbemu::ram::Ram;
use log::info;

use anyhow::Result;

fn main() -> Result<()> {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 1 {
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
    let gpu = Gpu::with_size(1024); // TODO implement
    let bus = Bus::new(
        cartridge,
        video_ram,
        h_ram,
        oam_ram,
        mirror_ram,
        working_ram,
        gpu,
    );

    let emu = Emulator::new(bus);

    info!("start emulator");
    emu.start()?;

    Ok(())
}
