use std::fmt;

/// Four character code (fourcc) defining the encoding of pixel data in an image buffer.
///
/// fourcc codes are documented on <https://www.fourcc.org/>.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Pixelformat(u32);

impl Pixelformat {
    /// Creates a [`Pixelformat`] from a *fourcc* code.
    pub const fn from_fourcc(fourcc: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(fourcc))
    }

    /// Returns the *fourcc* code represented by `self`.
    pub const fn as_fourcc(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

// Just a shorthand for `Pixelformat::from_fourcc`.
const fn f(fourcc: &[u8; 4]) -> Pixelformat {
    Pixelformat::from_fourcc(*fourcc)
}

/// Pixel format constants.
impl Pixelformat {
    /// `gggggggg bbbbbbbb rrrrrrrr` (FIXME: is this really correct?)
    pub const BGR3: Self = f(b"BGR3");

    /// `rrrrrrrr gggggggg bbbbbbbb`
    pub const RGB3: Self = f(b"RGB3");

    /// `bbbbbbbb gggggggg rrrrrrrr aaaaaaaa`
    pub const ABGR32: Self = f(b"AR24");

    /// `bbbbbbbb gggggggg rrrrrrrr xxxxxxxx`
    ///
    /// The `xxxxxxxx` channel data is ignored.
    pub const XBGR32: Self = f(b"XR24");

    /// `aaaaaaaa bbbbbbbb gggggggg rrrrrrrr`
    pub const BGRA32: Self = f(b"RA24");

    /// `xxxxxxxx bbbbbbbb gggggggg rrrrrrrr`
    pub const BGRX32: Self = f(b"RX24");

    /// `rrrrrrrr gggggggg bbbbbbbb aaaaaaaa`
    pub const RGBA32: Self = f(b"AB24");

    /// `rrrrrrrr gggggggg bbbbbbbb xxxxxxxx`
    pub const RGBX32: Self = f(b"XB24");

    /// `aaaaaaaa rrrrrrrr gggggggg bbbbbbbb`
    pub const ARGB32: Self = f(b"BA24");

    /// `xxxxxxxx rrrrrrrr gggggggg bbbbbbbb`
    ///
    /// The `xxxxxxxx` channel data is ignored.
    pub const XRGB32: Self = f(b"BX24");

    /// `bbbbbbbb gggggggg rrrrrrrr ????????` **DEPRECATED**
    ///
    /// This format is deprecated because the meaning of the last channel is ill-defined and its
    /// interpretation depends on driver and application. It will either be ignored (`xxxxxxxx` /
    /// [`Self::XBGR32`]) or treated as an alpha channel (`aaaaaaaa` / [`Self::ABGR32`]), so one of
    /// those formats should be used instead if possible.
    pub const BGR32: Self = f(b"BGR4");

    /// `???????? rrrrrrrr gggggggg bbbbbbbb` **DEPRECATED**
    ///
    /// This format is deprecated because the meaning of the first channel is ill-defined and its
    /// interpretation depends on driver and application. It will either be ignored (`xxxxxxxx` /
    /// [`Self::XRGB32`]) or treated as an alpha channel (`aaaaaaaa` / [`Self::ARGB32`]), so one of
    /// those formats should be used instead if possible.
    pub const RGB32: Self = f(b"RGB4");

    /// `yyyyyyyy uuuuuuuu YYYYYYYY vvvvvvvv`
    ///
    /// Packed YUV/YCbCr data with 4:2:2 chroma subsampling.
    ///
    /// `uuuuuuuu` and `vvvvvvvv` are shared by 2 neighboring pixels, while `yyyyyyyy` is the left
    /// pixel's Y value, and `YYYYYYYY` is the right pixel's Y value.
    pub const YUYV: Self = f(b"YUYV");

    /// Motion JPEG, a sequence of JPEG images with omitted huffman tables.
    ///
    /// The transmitted JPEG images lack the "DHT" frame (Define Huffman Table), and instead use a
    /// predefined one. Most common JPEG decoders will handle this fine and don't need any extra
    /// preprocessing.
    pub const MJPG: Self = f(b"MJPG");

    /// Data is a sequence of regular JFIF JPEG still images.
    ///
    /// Images can be decoded with any off-the-shelf JPEG decoder, no preprocessing is needed.
    pub const JPEG: Self = f(b"JPEG");

    /// UVC payload header metadata.
    ///
    /// Data is a stream of [`UvcMetadata`][crate::uvc::UvcMetadata] structures.
    pub const UVC: Self = f(b"UVCH");
}

impl fmt::Display for Pixelformat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d] = self.0.to_le_bytes();
        let [a, b, c, d] = [a as char, b as char, c as char, d as char];
        write!(f, "{}{}{}{}", a, b, c, d)
    }
}

impl fmt::Debug for Pixelformat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(Pixelformat::RGBA32.to_string(), "AB24");
    }
}
