use std::fmt;

/// Little-endian four character code (fourcc) identifying a pixel format.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Pixelformat(u32);

impl Pixelformat {
    /// `rrrrrrrr gggggggg bbbbbbbb aaaaaaaa`
    pub const RGBA32: Self = fourcc(b"AB24");
    /// `aaaaaaaa rrrrrrrr gggggggg bbbbbbbb`
    pub const RGB32: Self = fourcc(b"RGB4");

    /// Motion JPEG, a sequence of JPEG images with omitted huffman tables.
    ///
    /// The transmitted JPEG images lack the "DHT" frame (Define Huffman Table), and instead use a
    /// predefined one.
    pub const MJPG: Self = fourcc(b"MJPG");

    /// Data is a sequence of regular JPEG still images.
    ///
    /// Images can be decoded with any off-the-shelf JPEG decoder, no preprocessing is needed.
    pub const JPEG: Self = fourcc(b"JPEG");

    /// UVC payload header metadata.
    pub const UVC: Self = fourcc(b"UVCH");

    pub const YUYV: Self = fourcc(b"YUYV");

    pub fn from_fourcc(fourcc: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(fourcc))
    }
}

const fn fourcc(fourcc: &[u8; 4]) -> Pixelformat {
    Pixelformat(u32::from_le_bytes(*fourcc))
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
