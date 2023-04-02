use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct NdsKey: u32 {
        const A = 0b0000_0000_0001;
        const B = 0b0000_0000_0010;
        const Select = 0b0000_0000_0100;
        const Start = 0b0000_0000_1000;
        const Right = 0b0000_0001_0000;
        const Left = 0b0000_0010_0000;
        const Up = 0b0000_0100_0000;
        const Down = 0b0000_1000_0000;
        const R = 0b0001_0000_0000;
        const L = 0b0010_0000_0000;
        const X = 0b0100_0000_0000;
        const Y = 0b1000_0000_0000;
    }
}
