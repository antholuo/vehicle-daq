After a riveting discussion with chatgpt about embedded driver design in rust, I think I want to try and use bitfields.

Something like this
```rust
use bitfield::bitfield;

bitfield! {
    /// Mixed single/multi-bit register 0x30
    pub struct Reg0x30(u8);
    impl Debug;

    pub enable, set_enable: 7;
    pub mode, set_mode: 6;
    pub speed, set_speed: 5,3;
    pub range, set_range: 2,0;
}

// Add associated constants or helper methods
impl Reg0x30 {
    /// Register address
    pub const ADDR: u8 = 0x30;

    /// Optional read-modify-write helper
    pub fn new() -> Self {
        Self(0)
    }
}
```

This will be a lot of manual labor, but I think this is a design that hits many of the points I've thought about.

There's roughly 400 fields. Could I get through 200 a day? 1 rest day, then spend the next 2 days writing all logic? whew that's a loooot of work. Honestly it should actually be pretty speedy because its just one line per field again with less information. Maybe I can try to bust it out tomorrow.