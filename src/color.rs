#[allow(dead_code, unused_variables)]

#[derive(Default, Copy, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ColorRGB([u8; 3]);
impl ColorRGB {
   pub const BLACK: Self = ColorRGB([0; 3]);
   pub const WHITE: Self = ColorRGB([255; 3]);

   #[inline(always)]
   pub fn r(&self) -> u8 {
      self.0[0]
   }

   #[inline(always)]
   pub fn g(&self) -> u8 {
      self.0[1]
   }

   #[inline(always)]
   pub fn b(&self) -> u8 {
      self.0[2]
   }

   #[inline(always)]
   pub fn parts(&self) -> (u8, u8, u8) {
      (self.r(), self.g(), self.b())
   }

   #[inline(always)]
   pub fn from_f32(r: f32, g: f32, b: f32) -> ColorRGB {
      ColorRGB([float_to_256(r), float_to_256(g),float_to_256(b)])
   }
}

pub fn float_to_256(float: f32) -> u8 {
   (float * 255.0).round() as u8
}