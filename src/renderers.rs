use std::io::Write;
use crate::color::ColorRGB;
use crate::term_framebuffer::Render;

#[derive(Clone, Copy, Default)]
pub struct Char(char);
impl Render for Char {
   type CarryOver = ();
   type Input = char;

   fn render<W: Write>(&self, writer: &mut W, _c: &mut Self::CarryOver) {
      writer.write_all(&get_char_bytes(self.0)).expect("failed to write")
   }

   fn edit(&mut self, change: Self::Input) {
      self.0 = change;
   }
}

fn get_char_bytes(ch: char) -> [u8; 4] {
   let mut buffer = [0u8; 4]; // UTF-8 can be up to 4 bytes
   let len = ch.encode_utf8(&mut buffer).len();
   let mut result = [0u8; 4];
   result[..len].copy_from_slice(&buffer[..len]);
   result
}

pub const FULL: char = '\u{2588}';
#[derive(Clone, Copy, Default)]
pub struct RGB {
   col: ColorRGB
}
impl Render for RGB {
   type CarryOver = ColorRGB;
   type Input = ColorRGB;

   fn render<W: Write>(&self, writer: &mut W, c: &mut Self::CarryOver) {
      let msg = match self.col == *c {
         true => {
            format!("{FULL}")
         }
         false => {
            let (r, g, b) = self.col.parts();
            let msg = format!("\x1b[38;2;{r};{g};{b}m{FULL}");
            *c = self.col;
            msg
         }
      };
      writer.write_all(msg.as_bytes()).unwrap();
   }

   fn edit(&mut self, change: Self::Input) {
      self.col = change
   }
}

#[derive(Clone, Copy, Default)]
pub struct RGBChar {
   pub col: ColorRGB,
   pub char: char,
}
impl Render for RGBChar {
   type CarryOver = ColorRGB;
   type Input = RGBChar;

   fn render<W: Write>(&self, writer: &mut W, c: &mut Self::CarryOver) {
      let msg = match self.col == *c {
         true => {
            format!("{}", self.char)
         }
         false => {
            let (r, g, b) = self.col.parts();
            let msg = format!("\x1b[38;2;{r};{g};{b}m{}", self.char);
            *c = self.col;
            msg
         }
      };
      writer.write_all(msg.as_bytes()).unwrap();
   }

   fn edit(&mut self, change: Self::Input) {
      *self = change;
   }
}


