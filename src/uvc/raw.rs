//! From `linux/uvcvideo.h`.

use uoctl::{Ioctl, _IOWR};

// These are from `linux/usb/video.h`
ffi_enum! {
    pub enum XuQuery: u8 {
        SET_CUR  = 0x01,
        GET_CUR  = 0x81,
        GET_MIN  = 0x82,
        GET_MAX  = 0x83,
        GET_RES  = 0x84,
        GET_LEN  = 0x85,
        GET_INFO = 0x86,
        GET_DEF  = 0x87,
    }
}

#[repr(C)]
pub struct XuControlQuery {
    pub unit: u8,
    pub selector: u8,
    pub query: XuQuery,
    pub size: u16,
    pub data: *mut u8,
}

pub const UVCIOC_CTRL_QUERY: Ioctl<*mut XuControlQuery> = _IOWR(b'u', 0x21);
