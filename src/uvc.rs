//! USB Video Class extensions.

mod raw;

use std::{
    io, mem,
    os::unix::prelude::{AsRawFd, RawFd},
};

use bitflags::bitflags;

use crate::Device;

use self::raw::{XuControlQuery, XuQuery};

/// `UVCH` meta capture format.
#[derive(Clone, Copy, Debug)]
pub struct UvcMetadata {
    #[allow(dead_code)]
    raw: RawMetadata,
}

impl UvcMetadata {
    pub const MAX_SIZE: usize = mem::size_of::<RawMetadata>();

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() <= Self::MAX_SIZE);

        unsafe {
            // Safety: all-zero is valid for this type.
            let mut raw: RawMetadata = mem::zeroed();
            // Safety: the assert guarantees that `bytes.len()` bytes fit, and arbitrary bytes are
            // valid for the type.
            std::ptr::copy(bytes.as_ptr(), &mut raw as *mut _ as _, bytes.len());
            Self { raw }
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct RawMetadata {
    ts: u64,
    sof: u16,

    header_length: u8,
    header_info: HeaderInfo,
    presentation_time: u32,
    source_clock: [u8; 6],
}

bitflags! {
    #[repr(transparent)]
    struct HeaderInfo: u8 {
        const FRAME_ID               = 1 << 0;
        const END_OF_FRAME           = 1 << 1;
        const PRESENTATION_TIME      = 1 << 2;
        const SOURCE_CLOCK_REFERENCE = 1 << 3;
        /// Payload-specific bit.
        const PAYLOAD                = 1 << 4;
        const STILL_IMAGE            = 1 << 5;
        const ERROR                  = 1 << 6;
        const END_OF_HEADER          = 1 << 7;
    }
}

/// Grants access to operations that are specific to UVC devices.
pub struct UvcExt<'a> {
    device: &'a Device,
}

impl<'a> UvcExt<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { device }
    }

    pub fn extension_unit(&self, unit_id: u8) -> ExtensionUnit<'_> {
        ExtensionUnit {
            unit_id,
            device: self.device,
        }
    }
}

pub struct ExtensionUnit<'a> {
    unit_id: u8,
    device: &'a Device,
}

impl<'a> ExtensionUnit<'a> {
    fn fd(&self) -> RawFd {
        self.device.file.as_raw_fd()
    }

    pub fn control_info(&self, selector: u8) -> io::Result<ControlInfo> {
        let mut info = 0;
        let mut query = XuControlQuery {
            unit: self.unit_id,
            selector,
            query: XuQuery::GET_INFO,
            size: 1,
            data: &mut info,
        };

        unsafe {
            raw::ctrl_query(self.fd(), &mut query)?;

            Ok(ControlInfo::from_bits_unchecked(info))
        }
    }
}

bitflags! {
    pub struct ControlInfo: u8 {

    }
}
