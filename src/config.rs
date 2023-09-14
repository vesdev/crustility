use std::ops::{Add, Div, Mul, Sub};

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
    pub current_position: Millimeter,
    pub target_position: Millimeter,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Millimeter(f32);

impl From<f32> for Millimeter {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Sub for Millimeter {
    type Output = Millimeter;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add for Millimeter {
    type Output = Millimeter;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Div for Millimeter {
    type Output = Millimeter;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Mul for Millimeter {
    type Output = Millimeter;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Millimeter {
    pub fn from_serial(value: usize) -> Self {
        Self(value as f32 * 0.01)
    }
    pub fn to_serial(self) -> u16 {
        (self.0 / 0.01) as u16
    }
}

impl<'a> From<&'a mut Millimeter> for &'a mut f32 {
    fn from(value: &'a mut Millimeter) -> Self {
        &mut value.0
    }
}

impl From<Millimeter> for f32 {
    fn from(value: Millimeter) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DKey {}
