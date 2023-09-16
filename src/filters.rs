extern crate palette;

use palette::Hsv;
use palette::Srgb;
use palette::FromColor;
use palette::ShiftHueAssign;
use std::str::FromStr;

pub trait Filter {
    fn apply(&self, color: &Srgb) -> Srgb;
}

pub trait Filterable {
    fn apply_filter(&mut self, filter: &impl Filter);
}

impl<F: Filterable> Filterable for Option<F> {
    fn apply_filter(&mut self, filter: &impl Filter) {
       self.as_mut().map(|x| x.apply_filter(filter));
    }
}

impl Filterable for Srgb {
    fn apply_filter(&mut self, filter: &impl Filter) {
        *self = filter.apply(self);
    }
}

// TODO remove this abomination
impl Filterable for String {
    fn apply_filter(&mut self, filter: &impl Filter) {
        let mut color = Srgb::from_str(self).unwrap().into_format();
        color.apply_filter(filter);
        *self = format!("#{:x}", color.into_format::<u8>());
    }
}

pub struct HueFilter {
    pub hue: f32
}

impl Filter for HueFilter {
    fn apply(&self, color: &Srgb) -> Srgb {
        let mut color: Hsv = Hsv::from_color(color.into_format());
        color.shift_hue_assign(self.hue);
        Srgb::from_color(color)
    }
}
