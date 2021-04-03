use gbemu::bus::Bus;
use gbemu::cartridge::Cartridge;
use gbemu::emulator::Emulator;
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

    // FIXME サイズは適当に決めているのでちゃんとした値に治す
    let video_ram = Ram::with_size(1024);
    let h_ram = Ram::with_size(1024);
    let oam_ram = Ram::with_size(1024);
    let mirror_ram = Ram::with_size(1024);
    let working_ram = Ram::with_size(1024);
    let cartridge = Cartridge::new(bytes);
    let bus = Bus::new(
        cartridge,
        video_ram,
        h_ram,
        oam_ram,
        mirror_ram,
        working_ram,
    );

    let emu = Emulator::new(bus);

    info!("start emulator");
    emu.start()?;

    Ok(())
}
