// This code was adapted from Bevy (MIT/Apache2):
//
// https://github.com/bevyengine/bevy/blob/93a131661de507eb711264b11965fe1d4bb13f12/crates/bevy_render/src/color/mod.rs

use std::ops::{Add, AddAssign, Mul, MulAssign};

use glam::{vec3, vec4, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::colorspace::{HslRepresentation, SrgbColorSpace};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Color {
    /// sRGBA color
    Rgba {
        /// Red component. [0.0, 1.0]
        red: f32,
        /// Green component. [0.0, 1.0]
        green: f32,
        /// Blue component. [0.0, 1.0]
        blue: f32,
        /// Alpha component. [0.0, 1.0]
        alpha: f32,
    },
    /// RGBA color in the Linear sRGB colorspace (often colloquially referred to as "linear", "RGB", or "linear RGB").
    RgbaLinear {
        /// Red component. [0.0, 1.0]
        red: f32,
        /// Green component. [0.0, 1.0]
        green: f32,
        /// Blue component. [0.0, 1.0]
        blue: f32,
        /// Alpha component. [0.0, 1.0]
        alpha: f32,
    },
    /// HSL (hue, saturation, lightness) color with an alpha channel
    Hsla {
        /// Hue component. [0.0, 360.0]
        hue: f32,
        /// Saturation component. [0.0, 1.0]
        saturation: f32,
        /// Lightness component. [0.0, 1.0]
        lightness: f32,
        /// Alpha component. [0.0, 1.0]
        alpha: f32,
    },
}

impl Color {
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.);

    /// New `Color` from sRGB colorspace.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color::Rgba { red: r, green: g, blue: b, alpha: 1.0 }
    }

    /// New `Color` from sRGB colorspace.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::Rgba { red: r, green: g, blue: b, alpha: a }
    }

    /// New `Color` from linear RGB colorspace.
    pub const fn rgb_linear(r: f32, g: f32, b: f32) -> Color {
        Color::RgbaLinear { red: r, green: g, blue: b, alpha: 1.0 }
    }

    /// New `Color` from linear RGB colorspace.
    pub const fn rgba_linear(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::RgbaLinear { red: r, green: g, blue: b, alpha: a }
    }

    /// New `Color` with HSL representation in sRGB colorspace.
    pub const fn hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
        Color::Hsla { hue, saturation, lightness, alpha: 1.0 }
    }

    /// New `Color` with HSL representation in sRGB colorspace.
    pub const fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Color {
        Color::Hsla { hue, saturation, lightness, alpha }
    }

    /// New `Color` from sRGB colorspace.
    pub fn hex<T: AsRef<str>>(hex: T) -> Result<Color, HexColorError> {
        let hex = hex.as_ref();

        // RGB
        if hex.len() == 3 {
            let mut data = [0; 6];
            for (i, ch) in hex.chars().enumerate() {
                data[i * 2] = ch as u8;
                data[i * 2 + 1] = ch as u8;
            }
            return decode_rgb(&data);
        }

        // RGBA
        if hex.len() == 4 {
            let mut data = [0; 8];
            for (i, ch) in hex.chars().enumerate() {
                data[i * 2] = ch as u8;
                data[i * 2 + 1] = ch as u8;
            }
            return decode_rgba(&data);
        }

        // RRGGBB
        if hex.len() == 6 {
            return decode_rgb(hex.as_bytes());
        }

        // RRGGBBAA
        if hex.len() == 8 {
            return decode_rgba(hex.as_bytes());
        }

        Err(HexColorError::Length)
    }

    pub fn u8_debug(v: u8) -> Self {
        if v == 0 {
            Self::rgb(1., 1., 1.)
        } else if v == 1 {
            Self::rgb(1., 0., 0.)
        } else if v == 2 {
            Self::rgb(0., 1., 0.)
        } else if v == 3 {
            Self::rgb(0., 0., 1.)
        } else if v == 4 {
            Self::rgb(1., 0., 1.)
        } else if v == 5 {
            Self::rgb(0., 1., 1.)
        } else if v == 6 {
            Self::rgb(1., 1., 0.)
        } else {
            Self::rgb(0.5, 0.5, 0.5)
        }
    }

    /// New `Color` from sRGB colorspace.
    pub fn rgb_u8(r: u8, g: u8, b: u8) -> Color {
        Color::rgba_u8(r, g, b, u8::MAX)
    }

    // Float operations in const fn are not stable yet
    // see https://github.com/rust-lang/rust/issues/57241
    /// New `Color` from sRGB colorspace.
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::rgba(r as f32 / u8::MAX as f32, g as f32 / u8::MAX as f32, b as f32 / u8::MAX as f32, a as f32 / u8::MAX as f32)
    }

    /// Get red in sRGB colorspace.
    pub fn r(&self) -> f32 {
        match self.as_rgba() {
            Color::Rgba { red, .. } => red,
            _ => unreachable!(),
        }
    }

    /// Get green in sRGB colorspace.
    pub fn g(&self) -> f32 {
        match self.as_rgba() {
            Color::Rgba { green, .. } => green,
            _ => unreachable!(),
        }
    }

    /// Get blue in sRGB colorspace.
    pub fn b(&self) -> f32 {
        match self.as_rgba() {
            Color::Rgba { blue, .. } => blue,
            _ => unreachable!(),
        }
    }

    /// Set red in sRGB colorspace.
    pub fn set_r(&mut self, r: f32) -> &mut Self {
        *self = self.as_rgba();
        match self {
            Color::Rgba { red, .. } => *red = r,
            _ => unreachable!(),
        }
        self
    }

    /// Set green in sRGB colorspace.
    pub fn set_g(&mut self, g: f32) -> &mut Self {
        *self = self.as_rgba();
        match self {
            Color::Rgba { green, .. } => *green = g,
            _ => unreachable!(),
        }
        self
    }

    /// Set blue in sRGB colorspace.
    pub fn set_b(&mut self, b: f32) -> &mut Self {
        *self = self.as_rgba();
        match self {
            Color::Rgba { blue, .. } => *blue = b,
            _ => unreachable!(),
        }
        self
    }

    /// Get alpha.
    pub fn a(&self) -> f32 {
        match self {
            Color::Rgba { alpha, .. } | Color::RgbaLinear { alpha, .. } | Color::Hsla { alpha, .. } => *alpha,
        }
    }

    /// Set alpha.
    pub fn set_a(&mut self, a: f32) -> &mut Self {
        match self {
            Color::Rgba { alpha, .. } | Color::RgbaLinear { alpha, .. } | Color::Hsla { alpha, .. } => {
                *alpha = a;
            }
        }
        self
    }

    /// Converts a `Color` to variant `Color::Rgba`
    pub fn as_rgba(self: &Color) -> Color {
        match self {
            Color::Rgba { .. } => *self,
            Color::RgbaLinear { red, green, blue, alpha } => Color::Rgba {
                red: red.linear_to_nonlinear_srgb(),
                green: green.linear_to_nonlinear_srgb(),
                blue: blue.linear_to_nonlinear_srgb(),
                alpha: *alpha,
            },
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let [red, green, blue] = HslRepresentation::hsl_to_nonlinear_srgb(*hue, *saturation, *lightness);
                Color::Rgba { red, green, blue, alpha: *alpha }
            }
        }
    }

    /// Converts a `Color` to variant `Color::RgbaLinear`
    pub fn as_rgba_linear(self: &Color) -> Color {
        match self {
            Color::Rgba { red, green, blue, alpha } => Color::RgbaLinear {
                red: red.nonlinear_to_linear_srgb(),
                green: green.nonlinear_to_linear_srgb(),
                blue: blue.nonlinear_to_linear_srgb(),
                alpha: *alpha,
            },
            Color::RgbaLinear { .. } => *self,
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let [red, green, blue] = HslRepresentation::hsl_to_nonlinear_srgb(*hue, *saturation, *lightness);
                Color::RgbaLinear {
                    red: red.nonlinear_to_linear_srgb(),
                    green: green.nonlinear_to_linear_srgb(),
                    blue: blue.nonlinear_to_linear_srgb(),
                    alpha: *alpha,
                }
            }
        }
    }

    /// Converts a `Color` to variant `Color::Hsla`
    pub fn as_hsla(self: &Color) -> Color {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([*red, *green, *blue]);
                Color::Hsla { hue, saturation, lightness, alpha: *alpha }
            }
            Color::RgbaLinear { red, green, blue, alpha } => {
                let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([
                    red.linear_to_nonlinear_srgb(),
                    green.linear_to_nonlinear_srgb(),
                    blue.linear_to_nonlinear_srgb(),
                ]);
                Color::Hsla { hue, saturation, lightness, alpha: *alpha }
            }
            Color::Hsla { .. } => *self,
        }
    }

    /// Converts a `Color` to a `[f32; 4]` from sRGB colorspace
    pub fn as_rgba_f32(self: Color) -> [f32; 4] {
        match self {
            Color::Rgba { red, green, blue, alpha } => [red, green, blue, alpha],
            Color::RgbaLinear { red, green, blue, alpha } => {
                [red.linear_to_nonlinear_srgb(), green.linear_to_nonlinear_srgb(), blue.linear_to_nonlinear_srgb(), alpha]
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let [red, green, blue] = HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
                [red, green, blue, alpha]
            }
        }
    }

    /// Converts a `Color` to a `[f32; 4]` from linear RGB colorspace
    #[inline]
    pub fn as_linear_rgba_f32(self: Color) -> [f32; 4] {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                [red.nonlinear_to_linear_srgb(), green.nonlinear_to_linear_srgb(), blue.nonlinear_to_linear_srgb(), alpha]
            }
            Color::RgbaLinear { red, green, blue, alpha } => [red, green, blue, alpha],
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let [red, green, blue] = HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
                [red.nonlinear_to_linear_srgb(), green.nonlinear_to_linear_srgb(), blue.nonlinear_to_linear_srgb(), alpha]
            }
        }
    }

    /// Converts a `Color` to a `[f32; 4]` from HSL colorspace
    pub fn as_hsla_f32(self: Color) -> [f32; 4] {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([red, green, blue]);
                [hue, saturation, lightness, alpha]
            }
            Color::RgbaLinear { red, green, blue, alpha } => {
                let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([
                    red.linear_to_nonlinear_srgb(),
                    green.linear_to_nonlinear_srgb(),
                    blue.linear_to_nonlinear_srgb(),
                ]);
                [hue, saturation, lightness, alpha]
            }
            Color::Hsla { hue, saturation, lightness, alpha } => [hue, saturation, lightness, alpha],
        }
    }

    /// Converts `Color` to a `u32` from sRGB colorspace.
    ///
    /// Maps the RGBA channels in RGBA order to a little-endian byte array (GPUs are little-endian).
    /// `A` will be the most significant byte and `R` the least significant.
    pub fn as_rgba_u32(self: Color) -> u32 {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                u32::from_le_bytes([(red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8, (alpha * 255.0) as u8])
            }
            Color::RgbaLinear { red, green, blue, alpha } => u32::from_le_bytes([
                (red.linear_to_nonlinear_srgb() * 255.0) as u8,
                (green.linear_to_nonlinear_srgb() * 255.0) as u8,
                (blue.linear_to_nonlinear_srgb() * 255.0) as u8,
                (alpha * 255.0) as u8,
            ]),
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let [red, green, blue] = HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
                u32::from_le_bytes([(red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8, (alpha * 255.0) as u8])
            }
        }
    }

    /// Converts Color to a u32 from linear RGB colorspace.
    ///
    /// Maps the RGBA channels in RGBA order to a little-endian byte array (GPUs are little-endian).
    /// `A` will be the most significant byte and `R` the least significant.
    pub fn as_linear_rgba_u32(self: Color) -> u32 {
        match self {
            Color::Rgba { red, green, blue, alpha } => u32::from_le_bytes([
                (red.nonlinear_to_linear_srgb() * 255.0) as u8,
                (green.nonlinear_to_linear_srgb() * 255.0) as u8,
                (blue.nonlinear_to_linear_srgb() * 255.0) as u8,
                (alpha * 255.0) as u8,
            ]),
            Color::RgbaLinear { red, green, blue, alpha } => {
                u32::from_le_bytes([(red * 255.0) as u8, (green * 255.0) as u8, (blue * 255.0) as u8, (alpha * 255.0) as u8])
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let [red, green, blue] = HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
                u32::from_le_bytes([
                    (red.nonlinear_to_linear_srgb() * 255.0) as u8,
                    (green.nonlinear_to_linear_srgb() * 255.0) as u8,
                    (blue.nonlinear_to_linear_srgb() * 255.0) as u8,
                    (alpha * 255.0) as u8,
                ])
            }
        }
    }

    pub fn saturate(self, amount: f32) -> Self {
        if let Color::Hsla { hue, saturation, lightness, alpha } = self.as_hsla() {
            Self::Hsla { hue, saturation: (saturation + amount).clamp(0., 1.), lightness, alpha }
        } else {
            unreachable!()
        }
    }
    pub fn desaturate(self, amount: f32) -> Self {
        self.saturate(-amount)
    }

    pub fn lighten(self, amount: f32) -> Self {
        if let Color::Hsla { hue, saturation, lightness, alpha } = self.as_hsla() {
            Self::Hsla { hue, saturation, lightness: (lightness + amount).clamp(0., 1.), alpha }
        } else {
            unreachable!()
        }
    }
    pub fn darken(self, amount: f32) -> Self {
        self.lighten(-amount)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                let rhs = rhs.as_rgba_f32();
                *red += rhs[0];
                *green += rhs[1];
                *blue += rhs[2];
                *alpha += rhs[3];
            }
            Color::RgbaLinear { red, green, blue, alpha } => {
                let rhs = rhs.as_linear_rgba_f32();
                *red += rhs[0];
                *green += rhs[1];
                *blue += rhs[2];
                *alpha += rhs[3];
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let rhs = rhs.as_linear_rgba_f32();
                *hue += rhs[0];
                *saturation += rhs[1];
                *lightness += rhs[2];
                *alpha += rhs[3];
            }
        }
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                let rhs = rhs.as_rgba_f32();
                Color::Rgba { red: red + rhs[0], green: green + rhs[1], blue: blue + rhs[2], alpha: alpha + rhs[3] }
            }
            Color::RgbaLinear { red, green, blue, alpha } => {
                let rhs = rhs.as_linear_rgba_f32();
                Color::RgbaLinear { red: red + rhs[0], green: green + rhs[1], blue: blue + rhs[2], alpha: alpha + rhs[3] }
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                let rhs = rhs.as_linear_rgba_f32();
                Color::Hsla { hue: hue + rhs[0], saturation: saturation + rhs[1], lightness: lightness + rhs[2], alpha: alpha + rhs[3] }
            }
        }
    }
}

impl AddAssign<Vec4> for Color {
    fn add_assign(&mut self, rhs: Vec4) {
        let rhs: Color = rhs.into();
        *self += rhs;
    }
}

impl Add<Vec4> for Color {
    type Output = Color;

    fn add(self, rhs: Vec4) -> Self::Output {
        let rhs: Color = rhs.into();
        self + rhs
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.as_rgba_f32()
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Color::rgba(r, g, b, a)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Color::rgb(r, g, b)
    }
}

impl From<Color> for Vec4 {
    fn from(color: Color) -> Self {
        let color: [f32; 4] = color.into();
        vec4(color[0], color[1], color[2], color[3])
    }
}
impl From<Color> for Vec3 {
    fn from(color: Color) -> Self {
        let color: [f32; 4] = color.into();
        vec3(color[0], color[1], color[2])
    }
}

impl From<Vec4> for Color {
    fn from(vec4: Vec4) -> Self {
        Color::rgba(vec4.x, vec4.y, vec4.z, vec4.w)
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        if let Color::RgbaLinear { red, green, blue, alpha } = color.as_rgba_linear() {
            wgpu::Color { r: red as f64, g: green as f64, b: blue as f64, a: alpha as f64 }
        } else {
            unreachable!()
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        match self {
            Color::Rgba { red, green, blue, alpha } => Color::Rgba { red: red * rhs, green: green * rhs, blue: blue * rhs, alpha },
            Color::RgbaLinear { red, green, blue, alpha } => {
                Color::RgbaLinear { red: red * rhs, green: green * rhs, blue: blue * rhs, alpha }
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                Color::Hsla { hue: hue * rhs, saturation: saturation * rhs, lightness: lightness * rhs, alpha }
            }
        }
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        match self {
            Color::Rgba { red, green, blue, .. } | Color::RgbaLinear { red, green, blue, .. } => {
                *red *= rhs;
                *green *= rhs;
                *blue *= rhs;
            }
            Color::Hsla { hue, saturation, lightness, .. } => {
                *hue *= rhs;
                *saturation *= rhs;
                *lightness *= rhs;
            }
        }
    }
}

impl Mul<Vec4> for Color {
    type Output = Color;

    fn mul(self, rhs: Vec4) -> Self::Output {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                Color::Rgba { red: red * rhs.x, green: green * rhs.y, blue: blue * rhs.z, alpha: alpha * rhs.w }
            }
            Color::RgbaLinear { red, green, blue, alpha } => {
                Color::RgbaLinear { red: red * rhs.x, green: green * rhs.y, blue: blue * rhs.z, alpha: alpha * rhs.w }
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                Color::Hsla { hue: hue * rhs.x, saturation: saturation * rhs.y, lightness: lightness * rhs.z, alpha: alpha * rhs.w }
            }
        }
    }
}

impl MulAssign<Vec4> for Color {
    fn mul_assign(&mut self, rhs: Vec4) {
        match self {
            Color::Rgba { red, green, blue, alpha } | Color::RgbaLinear { red, green, blue, alpha } => {
                *red *= rhs.x;
                *green *= rhs.y;
                *blue *= rhs.z;
                *alpha *= rhs.w;
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                *hue *= rhs.x;
                *saturation *= rhs.y;
                *lightness *= rhs.z;
                *alpha *= rhs.w;
            }
        }
    }
}

impl Mul<Vec3> for Color {
    type Output = Color;

    fn mul(self, rhs: Vec3) -> Self::Output {
        match self {
            Color::Rgba { red, green, blue, alpha } => Color::Rgba { red: red * rhs.x, green: green * rhs.y, blue: blue * rhs.z, alpha },
            Color::RgbaLinear { red, green, blue, alpha } => {
                Color::RgbaLinear { red: red * rhs.x, green: green * rhs.y, blue: blue * rhs.z, alpha }
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                Color::Hsla { hue: hue * rhs.x, saturation: saturation * rhs.y, lightness: lightness * rhs.z, alpha }
            }
        }
    }
}

impl MulAssign<Vec3> for Color {
    fn mul_assign(&mut self, rhs: Vec3) {
        match self {
            Color::Rgba { red, green, blue, .. } | Color::RgbaLinear { red, green, blue, .. } => {
                *red *= rhs.x;
                *green *= rhs.y;
                *blue *= rhs.z;
            }
            Color::Hsla { hue, saturation, lightness, .. } => {
                *hue *= rhs.x;
                *saturation *= rhs.y;
                *lightness *= rhs.z;
            }
        }
    }
}

impl Mul<[f32; 4]> for Color {
    type Output = Color;

    fn mul(self, rhs: [f32; 4]) -> Self::Output {
        match self {
            Color::Rgba { red, green, blue, alpha } => {
                Color::Rgba { red: red * rhs[0], green: green * rhs[1], blue: blue * rhs[2], alpha: alpha * rhs[3] }
            }
            Color::RgbaLinear { red, green, blue, alpha } => {
                Color::RgbaLinear { red: red * rhs[0], green: green * rhs[1], blue: blue * rhs[2], alpha: alpha * rhs[3] }
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                Color::Hsla { hue: hue * rhs[0], saturation: saturation * rhs[1], lightness: lightness * rhs[2], alpha: alpha * rhs[3] }
            }
        }
    }
}

impl MulAssign<[f32; 4]> for Color {
    fn mul_assign(&mut self, rhs: [f32; 4]) {
        match self {
            Color::Rgba { red, green, blue, alpha } | Color::RgbaLinear { red, green, blue, alpha } => {
                *red *= rhs[0];
                *green *= rhs[1];
                *blue *= rhs[2];
                *alpha *= rhs[3];
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                *hue *= rhs[0];
                *saturation *= rhs[1];
                *lightness *= rhs[2];
                *alpha *= rhs[3];
            }
        }
    }
}

impl Mul<[f32; 3]> for Color {
    type Output = Color;

    fn mul(self, rhs: [f32; 3]) -> Self::Output {
        match self {
            Color::Rgba { red, green, blue, alpha } => Color::Rgba { red: red * rhs[0], green: green * rhs[1], blue: blue * rhs[2], alpha },
            Color::RgbaLinear { red, green, blue, alpha } => {
                Color::RgbaLinear { red: red * rhs[0], green: green * rhs[1], blue: blue * rhs[2], alpha }
            }
            Color::Hsla { hue, saturation, lightness, alpha } => {
                Color::Hsla { hue: hue * rhs[0], saturation: saturation * rhs[1], lightness: lightness * rhs[2], alpha }
            }
        }
    }
}

impl MulAssign<[f32; 3]> for Color {
    fn mul_assign(&mut self, rhs: [f32; 3]) {
        match self {
            Color::Rgba { red, green, blue, .. } | Color::RgbaLinear { red, green, blue, .. } => {
                *red *= rhs[0];
                *green *= rhs[1];
                *blue *= rhs[2];
            }
            Color::Hsla { hue, saturation, lightness, .. } => {
                *hue *= rhs[0];
                *saturation *= rhs[1];
                *lightness *= rhs[2];
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum HexColorError {
    #[error("Unexpected length of hex string")]
    Length,
    #[error("Error parsing hex value")]
    Hex(#[from] hex::FromHexError),
}

fn decode_rgb(data: &[u8]) -> Result<Color, HexColorError> {
    let mut buf = [0; 3];
    match hex::decode_to_slice(data, &mut buf) {
        Ok(_) => {
            let r = buf[0] as f32 / 255.0;
            let g = buf[1] as f32 / 255.0;
            let b = buf[2] as f32 / 255.0;
            Ok(Color::rgb(r, g, b))
        }
        Err(err) => Err(HexColorError::Hex(err)),
    }
}

fn decode_rgba(data: &[u8]) -> Result<Color, HexColorError> {
    let mut buf = [0; 4];
    match hex::decode_to_slice(data, &mut buf) {
        Ok(_) => {
            let r = buf[0] as f32 / 255.0;
            let g = buf[1] as f32 / 255.0;
            let b = buf[2] as f32 / 255.0;
            let a = buf[3] as f32 / 255.0;
            Ok(Color::rgba(r, g, b, a))
        }
        Err(err) => Err(HexColorError::Hex(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_color() {
        assert_eq!(Color::hex("FFF").unwrap(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::hex("000").unwrap(), Color::rgb(0.0, 0.0, 0.0));
        assert!(Color::hex("---").is_err());

        assert_eq!(Color::hex("FFFF").unwrap(), Color::rgba(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::hex("0000").unwrap(), Color::rgba(0.0, 0.0, 0.0, 0.0));
        assert!(Color::hex("----").is_err());

        assert_eq!(Color::hex("FFFFFF").unwrap(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::hex("000000").unwrap(), Color::rgb(0.0, 0.0, 0.0));
        assert!(Color::hex("------").is_err());

        assert_eq!(Color::hex("FFFFFFFF").unwrap(), Color::rgba(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::hex("00000000").unwrap(), Color::rgba(0.0, 0.0, 0.0, 0.0));
        assert!(Color::hex("--------").is_err());

        assert!(Color::hex("1234567890").is_err());
    }

    #[test]
    fn conversions_vec4() {
        let starting_vec4 = Vec4::new(0.4, 0.5, 0.6, 1.0);
        let starting_color = Color::from(starting_vec4);

        assert_eq!(starting_vec4, Vec4::from(starting_color),);

        let transformation = Vec4::new(0.5, 0.5, 0.5, 1.0);

        assert_eq!(starting_color * transformation, Color::from(starting_vec4 * transformation),);
    }

    #[test]
    fn mul_and_mulassign_f32() {
        let transformation = 0.5;
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(starting_color * transformation, Color::rgba(0.4 * 0.5, 0.5 * 0.5, 0.6 * 0.5, 1.0),);

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_f32by3() {
        let transformation = [0.4, 0.5, 0.6];
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(starting_color * transformation, Color::rgba(0.4 * 0.4, 0.5 * 0.5, 0.6 * 0.6, 1.0),);

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_f32by4() {
        let transformation = [0.4, 0.5, 0.6, 0.9];
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(starting_color * transformation, Color::rgba(0.4 * 0.4, 0.5 * 0.5, 0.6 * 0.6, 1.0 * 0.9),);

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_vec3() {
        let transformation = Vec3::new(0.2, 0.3, 0.4);
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(starting_color * transformation, Color::rgba(0.4 * 0.2, 0.5 * 0.3, 0.6 * 0.4, 1.0),);

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_vec4() {
        let transformation = Vec4::new(0.2, 0.3, 0.4, 0.5);
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(starting_color * transformation, Color::rgba(0.4 * 0.2, 0.5 * 0.3, 0.6 * 0.4, 1.0 * 0.5),);

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }
}
