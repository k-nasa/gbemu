use gbemu::bus::Bus;
use gbemu::cartridge::Cartridge;
use gbemu::emulator::Emulator;

fn main() {
    let bus = Bus::new(Cartridge::new(Vec::new()));

    let emu = Emulator::new(bus);

    emu.start();
}
