use gbemu::emulator::Emulator;
use log::info;

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

    info!("start emulator");
    let emu = Emulator::from_rom_byte(bytes);
    emu.start()?;

    Ok(())
}
