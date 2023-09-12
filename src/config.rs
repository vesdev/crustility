use std::ops::RangeBounds;

use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub hkeys: Vec<HKey>,
    pub dkeys: Vec<DKey>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HKey {
    pub rt: Option<RapidTrigger>,
    pub hysterisis: Hysterisis,
    pub hid: bool,
    pub char: String,
    pub rest: usize,
    pub down: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RapidTrigger {
    pub continuos: bool, //continuos rapid trigger
    pub down_sensitivity: Millimeter,
    pub up_sensitivity: Millimeter,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Hysterisis {
    pub lower: Millimeter,
    pub upper: Millimeter,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Millimeter(f32);

impl From<f32> for Millimeter {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
impl Millimeter {
    pub fn from_serial(value: u16) -> Self {
        Self(value as f32 * 0.01)
    }
    pub fn to_serial(&self) -> u16 {
        (self.0 / 0.01) as u16
    }
}

impl<'a> From<&'a mut Millimeter> for &'a mut f32 {
    fn from(value: &'a mut Millimeter) -> Self {
        &mut value.0
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DKey {}
