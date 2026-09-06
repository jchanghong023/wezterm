//! Colors for attributes

#[cfg(feature = "use_serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::result::Result;
pub use wezterm_cell::color::{AnsiColor, ColorAttribute, RgbColor, SrgbaTuple};

#[derive(Clone, PartialEq)]
pub struct Palette256(pub [SrgbaTuple; 256]);

#[cfg(feature = "use_serde")]
impl Serialize for Palette256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_vec().serialize(serializer)
    }
}

#[cfg(feature = "use_serde")]
impl<'de> Deserialize<'de> for Palette256 {
    fn deserialize<D>(deserializer: D) -> Result<Palette256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Vec::<SrgbaTuple>::deserialize(deserializer)?;
        use std::convert::TryInto;
        Ok(Self(s.try_into().map_err(|_| {
            serde::de::Error::custom("Palette256 size mismatch")
        })?))
    }
}

impl std::iter::FromIterator<SrgbaTuple> for Palette256 {
    fn from_iter<I: IntoIterator<Item = SrgbaTuple>>(iter: I) -> Self {
        let mut colors = [SrgbaTuple::default(); 256];
        for (s, d) in iter.into_iter().zip(colors.iter_mut()) {
            *d = s;
        }
        Self(colors)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
pub struct ColorPalette {
    pub colors: Palette256,
    pub foreground: SrgbaTuple,
    pub background: SrgbaTuple,
    pub cursor_fg: SrgbaTuple,
    pub cursor_bg: SrgbaTuple,
    pub cursor_border: SrgbaTuple,
    pub selection_fg: SrgbaTuple,
    pub selection_bg: SrgbaTuple,
    pub scrollbar_thumb: SrgbaTuple,
    pub split: SrgbaTuple,
}

impl fmt::Debug for Palette256 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // If we wanted to dump all of the entries, we'd use this:
        // self.0[..].fmt(fmt)
        // However, we typically don't care about those and we're interested
        // in the Debug-ability of ColorPalette that embeds us.
        write!(fmt, "[suppressed]")
    }
}

impl ColorPalette {
    pub fn resolve_fg(&self, color: ColorAttribute) -> SrgbaTuple {
        match color {
            ColorAttribute::Default => self.foreground,
            ColorAttribute::PaletteIndex(idx) => self.colors.0[idx as usize],
            ColorAttribute::TrueColorWithPaletteFallback(color, _)
            | ColorAttribute::TrueColorWithDefaultFallback(color) => color.into(),
        }
    }
    pub fn resolve_bg(&self, color: ColorAttribute) -> SrgbaTuple {
        match color {
            ColorAttribute::Default => self.background,
            ColorAttribute::PaletteIndex(idx) => self.colors.0[idx as usize],
            ColorAttribute::TrueColorWithPaletteFallback(color, _)
            | ColorAttribute::TrueColorWithDefaultFallback(color) => color.into(),
        }
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_PALETTE: ColorPalette = ColorPalette::compute_default();
}

impl Default for ColorPalette {
    /// Construct a default color palette
    fn default() -> ColorPalette {
        DEFAULT_PALETTE.clone()
    }
}

impl ColorPalette {
    fn compute_default() -> Self {
        let mut colors = [SrgbaTuple::default(); 256];

        // The XTerm ansi color set
        // Custom neutral dark-gray ANSI set (spec: keep red=red, green=green mapping)
        let ansi: [SrgbaTuple; 16] = [
            // Black
            RgbColor::new_8bpc(0x17, 0x18, 0x1b).into(),
            // Maroon
            RgbColor::new_8bpc(0xe5, 0x8a, 0x8a).into(),
            // Green
            RgbColor::new_8bpc(0xa5, 0xc9, 0x8b).into(),
            // Olive
            RgbColor::new_8bpc(0xe4, 0xc3, 0x8a).into(),
            // Navy
            RgbColor::new_8bpc(0x8a, 0xb4, 0xf8).into(),
            // Purple
            RgbColor::new_8bpc(0xc5, 0xa3, 0xe6).into(),
            // Teal
            RgbColor::new_8bpc(0x8d, 0xcb, 0xd3).into(),
            // Silver
            RgbColor::new_8bpc(0xda, 0xdd, 0xe3).into(),
            // Grey
            RgbColor::new_8bpc(0x7c, 0x85, 0x94).into(),
            // Red
            RgbColor::new_8bpc(0xf2, 0xa3, 0xa3).into(),
            // Lime
            RgbColor::new_8bpc(0xb8, 0xdc, 0xa1).into(),
            // Yellow
            RgbColor::new_8bpc(0xf0, 0xd7, 0xa8).into(),
            // Blue
            RgbColor::new_8bpc(0xa4, 0xc7, 0xff).into(),
            // Fuchsia
            RgbColor::new_8bpc(0xd8, 0xb9, 0xf2).into(),
            // Aqua
            RgbColor::new_8bpc(0xa5, 0xde, 0xe5).into(),
            // White
            RgbColor::new_8bpc(0xf1, 0xf3, 0xf5).into(),
        ];

        colors[0..16].copy_from_slice(&ansi);

        // 216 color cube.
        // This isn't the perfect color cube, but it matches the values used
        // by xterm, which are slightly brighter.
        static RAMP6: [u8; 6] = [0, 0x5f, 0x87, 0xaf, 0xd7, 0xff];
        for idx in 0..216 {
            let blue = RAMP6[idx % 6];
            let green = RAMP6[idx / 6 % 6];
            let red = RAMP6[idx / 6 / 6 % 6];

            colors[16 + idx] = RgbColor::new_8bpc(red, green, blue).into();
        }

        // 24 grey scales
        static GREYS: [u8; 24] = [
            0x08, 0x12, 0x1c, 0x26, 0x30, 0x3a, 0x44, 0x4e, 0x58, 0x62, 0x6c, 0x76, 0x80, 0x8a,
            0x94, 0x9e, 0xa8, 0xb2, /* Grey70 */
            0xbc, 0xc6, 0xd0, 0xda, 0xe4, 0xee,
        ];

        for idx in 0..24 {
            let grey = GREYS[idx];
            colors[232 + idx] = RgbColor::new_8bpc(grey, grey, grey).into();
        }

        let foreground = RgbColor::new_8bpc(0xda, 0xdd, 0xe3).into();
        let background = RgbColor::new_8bpc(0x1e, 0x1f, 0x22).into();

        let cursor_bg = RgbColor::new_8bpc(0xcd, 0xd3, 0xdd).into();
        let cursor_border = RgbColor::new_8bpc(0xcd, 0xd3, 0xdd).into();
        let cursor_fg = RgbColor::new_8bpc(0x1e, 0x1f, 0x22).into();

        let selection_fg = RgbColor::new_8bpc(0xf1, 0xf3, 0xf5).into();
        let selection_bg = RgbColor::new_8bpc(0x35, 0x46, 0x60).into();

        let scrollbar_thumb = RgbColor::new_8bpc(0x46, 0x4b, 0x55).into();
        let split = RgbColor::new_8bpc(0x3b, 0x3f, 0x48).into();

        ColorPalette {
            colors: Palette256(colors),
            foreground,
            background,
            cursor_fg,
            cursor_bg,
            cursor_border,
            selection_fg,
            selection_bg,
            scrollbar_thumb,
            split,
        }
    }
}
