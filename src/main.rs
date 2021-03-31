use gbemu::bus::Bus;
use gbemu::cartridge::Cartridge;
use gbemu::emulator::Emulator;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 1 {
        return;
    }

    let filename = &args[1];
    let bytes = std::fs::read(filename).unwrap();

    let bus = Bus::new(Cartridge::new(bytes));
    let emu = Emulator::new(bus);

    emu.start();
}
