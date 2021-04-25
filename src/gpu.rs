use crate::SharedBus;
use crate::{HalfWord, Word};

const CYCLE_PER_LINE: usize = 456;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const TILEMAP0: Word = 0x9800;
const TILEMAP1: Word = 0x9C00;

pub struct Gpu {
    data: Vec<u8>,
    bus: Option<SharedBus>,
    cycles: usize,
    ly: usize,
    scroll_x: usize,
    scroll_y: usize,
    lcdc: u8,
}

impl Gpu {
    pub fn new(size: usize, bus: Option<SharedBus>) -> Gpu {
        Gpu {
            data: vec![0; size],
            bus,
            cycles: 0,
            ly: 0,
            scroll_x: 0,
            scroll_y: 0,
            lcdc: 0x91,
        }
    }

    pub fn step(&mut self) {
        if self.bus.is_none() {
            // TODO error handle
            panic!("hogehoge")
        }

        self.cycles += 4;

        if self.cycles < CYCLE_PER_LINE {
            return;
        }

        if self.ly < 144 {
            self.build_gb_tile();
        } else if self.ly == 144 {
            self.build_sprites();
        } else if self.ly >= 144 {
            self.ly = 0
        }

        self.ly += 1;
        self.cycles -= CYCLE_PER_LINE;
    }

    fn build_gb_tile(&mut self) {
        for x in 0..SCREEN_WIDTH {
            let tile_y = ((self.ly + self.scroll_y) % 0x100) / 8 * 32;
            let tile_x = (x + self.scroll_x) / 8 % 32;

            let tile_id = self.get_tile_id(tile_y, tile_x, self.get_bg_tilemap_addr());
            let palette_id = {
                let offset = (self.scroll_x % 8) + x;
                let addr = (self.ly + self.scroll_y) % 8;

                self.get_bg_palette_id(tile_id, offset, addr)
            };

            todo!() // TODO return image data
        }
    }

    fn build_sprites(&mut self) {}

    pub fn set_bus(&mut self, bus: SharedBus) {
        self.bus = Some(bus)
    }

    pub fn read(&self, address: Word) -> HalfWord {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: Word, byte: HalfWord) {
        self.data[address as usize] = byte;

        let bus = self.bus.as_ref().unwrap();
        let mut bus = bus.lock().unwrap();
        bus.write_byte(address, byte);
    }

    fn get_tile_id(&self, tile_y: usize, line_offset: usize, offset_addr: Word) -> HalfWord {
        let addr = tile_y as u16 + line_offset as u16 + offset_addr;
        let bus = self.bus.as_ref().unwrap();
        let id = bus.lock().unwrap().read_byte(addr);
        id
    }

    fn get_window_tilemap_affr(&self) -> Word {
        if self.lcdc & 0x40 == 0x40 {
            return TILEMAP1;
        }
        return TILEMAP0;
    }

    fn get_bg_tilemap_addr(&self) -> Word {
        if self.lcdc & 0x08 == 0x08 {
            return TILEMAP1;
        }
        return TILEMAP0;
    }

    fn get_bg_palette_id(&self, tile_id: HalfWord, x: usize, y: usize) -> Word {
        // TODO implement switch tile data
        let addr = u16::from((tile_id + 128) * 0x10);
        let base = self.get_tile_data_addr() + addr + (y * 2) as u16;

        let l1 = self.read_bus_byte(base);
        let l2 = self.read_bus_byte(base + 1);

        let mut palette_id = 0;
        if l1 & (0x01 << (7 - x)) != 0 {
            palette_id = 1
        }
        if l2 & (0x01 << (7 - x)) != 0 {
            palette_id += 2
        }

        palette_id
    }

    fn get_tile_data_addr(&self) -> Word {
        // TODO implement switch tile data
        0x8800
    }

    fn read_bus_byte(&self, addr: Word) -> HalfWord {
        let bus = self.bus.as_ref().unwrap().lock().unwrap();
        bus.read_byte(addr)
    }
}
