use self::{
    addresses::{ACTOR_COLLECTION, MAIN_RAM_OFFSET},
    io::MemCursor,
};

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Seek, SeekFrom};

pub mod addresses;
pub mod io;

pub fn read_actors(ram: &[u8]) -> std::io::Result<ActorCollection> {
    let mut mem_cursor = MemCursor::new(ram, MAIN_RAM_OFFSET as u64);
    ActorCollection::read(&mut mem_cursor)
}

#[derive(Debug, PartialEq, Clone)]
pub struct HitCollection {
    /// length of 32
    hits: Vec<Hit>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ActorCollection {
    /// length of 7
    actors: Vec<Option<Actor>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Hit {
    pub next: Option<Box<Hit>>,
    /// addr 0x14
    pub effect_data_addr: HitEffectData,
    /// addr 0x24
    pub x1: i16,
    pub y1: i16,
    pub x2: i16,
    pub y2: i16,
    /// addr 0x238
    pub hp: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct HitEffectData {
    /// addr 0xa
    pub effect: u8,
    pub power: i8,
    /// addr 0x14
    pub dmg: i16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Actor {
    pub next: Option<Box<Actor>>,
    /// addr 0x80
    pub x: i32,
    pub y: i32,
    pub speed_x: i16,
    pub speed_y: i16,
}

impl ActorCollection {
    pub fn read<T: AsRef<[u8]>>(cursor: &mut MemCursor<T>) -> std::io::Result<Self> {
        cursor.seek(SeekFrom::Start(ACTOR_COLLECTION as u64))?;
        let addresses = (0..7)
            .map(|_| cursor.read_u32::<LittleEndian>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            actors: addresses
                .iter()
                .map(|addr| {
                    if *addr == 0 {
                        Ok(None)
                    } else {
                        cursor.seek(SeekFrom::Start(*addr as u64))?;
                        Actor::read(cursor).map(Some)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Actor {
    pub fn read<T: AsRef<[u8]>>(cursor: &mut MemCursor<T>) -> std::io::Result<Self> {
        let next_addr = cursor.read_u32::<LittleEndian>()?;
        cursor.seek(SeekFrom::Current(0x80 - 4))?;
        let x = cursor.read_i32::<LittleEndian>()?;
        let y = cursor.read_i32::<LittleEndian>()?;
        let speed_x = cursor.read_i16::<LittleEndian>()?;
        let speed_y = cursor.read_i16::<LittleEndian>()?;
        Ok(Self {
            next: if next_addr == 0 {
                None
            } else {
                Some(Box::new(Self::read(cursor)?))
            },
            x,
            y,
            speed_x,
            speed_y,
        })
    }
}

fn check_memory(ram: &[u8]) {
    // use std::io::{Seek, SeekFrom};
    let mut mem_cursor = MemCursor::new(ram, MAIN_RAM_OFFSET as u64);
    let actors = ActorCollection::read(&mut mem_cursor).unwrap();
    // jp version stuff
    // mem_cursor
    //     .seek(SeekFrom::Start(0x02049e0c_u64))
    //     .unwrap();
    // // let actors = ActorCollection::read(&mut mem_cursor).unwrap();
    // let actor = Actor::read(&mut mem_cursor).unwrap();
    println!("{:#?}", actors);
}
