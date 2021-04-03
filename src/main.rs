use gbemu::bus::Bus;
use gbemu::cartridge::Cartridge;
use gbemu::emulator::Emulator;
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

    let bus = Bus::new(Cartridge::new(bytes));
    let emu = Emulator::new(bus);

    info!("start emulator");
    emu.start()?;

    Ok(())
}
