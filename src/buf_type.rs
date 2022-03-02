use bitflags::bitflags;

use crate::CapabilityFlags;

macro_rules! buf_types {
    (
        $( $(#[$($attr:tt)+])* $name:ident = $value:literal, )+
    ) => {
        ffi_enum! {
            /// A buffer or stream type.
            pub enum BufType: u32 { // more of a "stream type", really
                $( $(#[$($attr)+])* $name = $value, )+
            }
        }

        impl BufType {
            const ALL: &'static [Self] = &[
                $( Self::$name, )+
            ];
        }

        bitflags! {
            /// Bitflags of supported buffer types.
            pub struct BufTypes: u32 {
                $( $(#[$($attr)+])* const $name = 1 << $value; )+
            }
        }

        impl BufTypes {
            const CAPS: &'static [CapabilityFlags] = &[
                $( CapabilityFlags::$name, )+
            ];
        }
    };
}

buf_types! {
    /// Single-plane video capture.
    VIDEO_CAPTURE = 1,
    /// Single-plane video output.
    VIDEO_OUTPUT = 2,
    VIDEO_OVERLAY = 3,
    VBI_CAPTURE = 4,
    VBI_OUTPUT = 5,
    SLICED_VBI_CAPTURE = 6,
    SLICED_VBI_OUTPUT = 7,
    VIDEO_OUTPUT_OVERLAY = 8,
    VIDEO_CAPTURE_MPLANE = 9,
    VIDEO_OUTPUT_MPLANE = 10,
    SDR_CAPTURE = 11,
    SDR_OUTPUT = 12,
    /// Metadata capture.
    META_CAPTURE = 13,
    /// Metadata output.
    META_OUTPUT = 14,
}

impl BufTypes {
    pub(crate) fn from_capabilities(caps: CapabilityFlags) -> Self {
        let mut buf_types = BufTypes::empty();
        for (i, cap) in Self::CAPS.iter().enumerate() {
            if caps.contains(*cap) {
                buf_types |= BufTypes::from_bits(1 << (i + 1)).unwrap();
            }
        }

        buf_types
    }
}

impl IntoIterator for BufTypes {
    type Item = BufType;
    type IntoIter = BufTypesIter;

    fn into_iter(self) -> Self::IntoIter {
        BufTypesIter {
            buf_types: self,
            index: 0,
        }
    }
}

/// Iterator over the [`BufType`]s stored in a [`BufTypes`] value.
pub struct BufTypesIter {
    buf_types: BufTypes,
    index: u32,
}

impl Iterator for BufTypesIter {
    type Item = BufType;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.index += 1;

            if self
                .buf_types
                .contains(BufTypes::from_bits(1 << self.index)?)
            {
                return Some(BufType::ALL[self.index as usize - 1]);
            }
        }
    }
}
