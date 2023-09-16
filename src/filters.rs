extern crate palette;

use palette::rgb::Rgb;
use palette::hsv::Hsv;
use palette::FromColor;
use palette::ShiftHueAssign;
use std::str::FromStr;

pub trait Filter {
    fn apply(&self, color: &str) -> String;
}

pub struct HueFilter {
    pub hue: f32
}

impl Filter for HueFilter {
    fn apply(&self, color: &str) -> String {
        let color = Rgb::from_str(color).unwrap();

        let mut color: Hsv<f32> = Hsv::from_color(color.into_format::<f32>());
        color.shift_hue_assign(self.hue);

        let color = Rgb::from_color(color).into_format::<u8>();
        format!("#{color:x}")
    }
}
