//! FFI definitions compatible with `videodev2.h`.
//!
//! These types do not provide a "Rusty" API and should not be exposed as public APIs.

#![allow(bad_style)]

pub mod controls;

use std::ffi::c_void;
use std::os::raw::c_ulong;

use nix::libc::timeval;
use nix::{ioctl_read, ioctl_readwrite, ioctl_write_ptr};

use crate::buf_type::BufType;
use crate::{shared::*, PixelFormat};

pub const VIDEO_MAX_PLANES: usize = 8;

#[repr(C)]
#[derive(Debug)]
pub struct Capabilities {
    pub driver: [u8; 16],
    pub card: [u8; 32],
    pub bus_info: [u8; 32],
    pub version: u32,
    pub capabilities: CapabilityFlags,
    pub device_caps: CapabilityFlags,
    reserved: [u32; 3],
}

#[repr(C)]
pub struct FmtDesc {
    /// Number of the format in the enumeration, set by the application. This is in no way related
    /// to the `pixel_format` field.
    pub index: u32,
    /// Type of the data stream, set by the application. Only these types are valid here:
    ///
    /// `V4L2_BUF_TYPE_VIDEO_CAPTURE`, `V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE`,
    /// `V4L2_BUF_TYPE_VIDEO_OUTPUT`, `V4L2_BUF_TYPE_VIDEO_OUTPUT_MPLANE`,
    /// `V4L2_BUF_TYPE_VIDEO_OVERLAY`, `V4L2_BUF_TYPE_SDR_CAPTURE`, `V4L2_BUF_TYPE_SDR_OUTPUT`,
    /// `V4L2_BUF_TYPE_META_CAPTURE` and `V4L2_BUF_TYPE_META_OUTPUT`. See `v4l2_buf_type`.
    pub type_: BufType,
    pub flags: FormatFlags,
    /// Description of the format, a NUL-terminated ASCII string. This information is intended for
    /// the user, for example: “YUV 4:2:2”.
    pub description: [u8; 32],
    /// The image format identifier. This is a four character code as computed by the
    /// `v4l2_fourcc()` macro:
    ///
    /// `#define v4l2_fourcc(a,b,c,d) (((__u32)(a)<<0)|((__u32)(b)<<8)|((__u32)(c)<<16)|((__u32)(d)<<24))`
    pub pixel_format: PixelFormat,
    /// Media bus code restricting the enumerated formats, set by the application. Only applicable
    /// to drivers that advertise the `V4L2_CAP_IO_MC` capability, shall be 0 otherwise.
    pub mbus_code: u32,
    pub reserved: [u32; 3],
}

#[repr(C)]
pub struct Format {
    pub type_: BufType,
    pub fmt: FormatUnion,
}

#[repr(C)]
pub union FormatUnion {
    pub pix: PixFormat,
    pub pix_mp: PixFormatMplane,
    pub win: Window,
    pub meta: MetaFormat,
    // TODO...
    pub raw_data: [u8; 200],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Area {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Clip {
    pub c: Rect,
    pub next: *mut Clip,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Window {
    pub w: Rect,
    pub field: Field,
    pub chromakey: u32,
    pub clips: *mut Clip,
    pub clipcount: u32,
    pub bitmap: *mut c_void,
    pub global_alpha: u8,
}

/// `v4l2_pix_format`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PixFormat {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub field: Field,
    pub bytesperline: u32,
    pub sizeimage: u32,
    pub colorspace: Colorspace,
    pub priv_: u32,
    // Below fields are only valid if `priv_` equals `V4L2_PIX_FMT_PRIV_MAGIC`.
    pub flags: PixFmtFlag,
    pub enc: u32,
    pub quantization: Quantization,
    pub xfer_func: XferFunc,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PlanePixFormat {
    pub sizeimage: u32,
    pub bytesperline: u32,
    pub reserved: [u16; 6],
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct PixFormatMplane {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub field: u32,
    pub colorspace: u32,
    pub plane_fmt: [PlanePixFormat; VIDEO_MAX_PLANES],
    pub num_planes: u8,
    pub flags: u8,
    pub enc: u8,
    pub quantization: u8,
    pub xfer_func: u8,
    pub reserved: [u8; 7],
}

#[repr(C)]
pub struct Output {
    pub index: u32,
    pub name: [u8; 32],
    pub type_: OutputType,
    /// Associated audio outputs (bitfield).
    pub audioset: u32,
    /// Modulator associated with this output.
    pub modulator: u32,
    pub std: AnalogStd,
    pub capabilities: OutputCapabilities,
    pub reserved: [u32; 3],
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct MetaFormat {
    pub dataformat: PixelFormat,
    pub buffersize: u32,
}

#[repr(C)]
pub struct Input {
    pub index: u32,
    pub name: [u8; 32],
    pub type_: InputType,
    pub audioset: u32,
    pub tuner: u32,
    pub std: AnalogStd,
    pub status: InputStatus,
    pub capabilities: InputCapabilities,
    pub reserved: [u32; 3],
}

#[repr(C)]
pub struct QueryCtrl {
    pub id: u32,
    pub type_: CtrlType,
    pub name: [u8; 32],
    pub minimum: i32,
    pub maximum: i32,
    pub step: i32,
    pub default_value: i32,
    pub flags: ControlFlags,
    pub reserved: [u32; 2],
}

#[repr(C, packed)]
pub struct QueryMenu {
    pub id: u32,
    pub index: u32,
    pub name_or_value: QueryMenuUnion,
    pub reserved: u32,
}

#[repr(C)]
pub union QueryMenuUnion {
    pub name: [u8; 32],
    pub value: i64,
}

#[derive(Debug)]
#[repr(C)]
pub struct RequestBuffers {
    pub count: u32,
    pub type_: BufType,
    pub memory: Memory,
    pub capabilities: BufCap,
    pub reserved: [u32; 1],
}

#[repr(C)]
pub struct Timecode {
    pub type_: TimecodeType,
    pub flags: TimecodeFlags,
    pub frames: u8,
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub userbits: [u8; 4],
}

#[repr(C)]
pub struct Buffer {
    pub index: u32,
    pub type_: BufType,
    pub bytesused: u32,
    pub flags: BufFlag,
    pub field: u32,
    pub timestamp: timeval,
    pub timecode: Timecode,
    pub sequence: u32,
    pub memory: Memory,
    pub m: BufferMemoryUnion,
    pub length: u32,
    pub reserved2: u32,
    pub tail: BufferTailUnion,
}

#[repr(C)]
pub union BufferMemoryUnion {
    pub offset: u32,
    pub userptr: c_ulong,
    pub planes: *mut Plane,
    pub fd: i32,
}

#[repr(C)]
pub union BufferTailUnion {
    pub request_fd: i32,
    pub reserved: u32,
}

#[repr(C)]
pub struct Plane {
    pub bytesused: u32,
    pub length: u32,
    pub m: PlaneMemoryUnion,
    pub data_offset: u32,
    pub reserved: [u32; 11],
}

#[repr(C)]
pub union PlaneMemoryUnion {
    pub mem_offset: u32,
    pub userptr: c_ulong,
    pub fd: i32,
}

#[repr(C)]
pub struct FrmSizeEnum {
    pub index: u32,
    pub pixel_format: PixelFormat,
    pub type_: FrmSizeType,
    pub union: FrmSizeUnion,
    pub reserved: [u32; 2],
}

#[repr(C)]
pub union FrmSizeUnion {
    pub discrete: FrmSizeDiscrete,
    pub stepwise: FrmSizeStepwise,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrmSizeDiscrete {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrmSizeStepwise {
    pub min_width: u32,
    pub max_width: u32,
    pub step_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub step_height: u32,
}

#[repr(C)]
pub struct FrmIvalEnum {
    pub index: u32,
    pub pixel_format: PixelFormat,
    pub width: u32,
    pub height: u32,
    pub type_: FrmIvalType,
    pub union: FrmIvalUnion,
    pub reserved: [u32; 2],
}

#[repr(C)]
pub union FrmIvalUnion {
    pub discrete: Fract,
    pub stepwise: FrmIvalStepwise,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FrmIvalStepwise {
    pub min: Fract,
    pub max: Fract,
    pub step: Fract,
}

#[repr(C)]
pub struct StreamParm {
    pub type_: BufType,
    pub union: StreamParmUnion,
}

#[repr(C)]
pub union StreamParmUnion {
    pub capture: CaptureParm,
    pub output: OutputParm,
    pub raw_data: [u8; 200],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CaptureParm {
    pub capability: StreamParamCaps,
    pub capturemode: CaptureParamFlags,
    pub timeperframe: Fract,
    pub extendedmode: u32,
    pub readbuffers: u32,
    pub reserved: [u32; 4],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct OutputParm {
    pub capability: StreamParamCaps,
    pub outputmode: u32,
    pub timeperframe: Fract,
    pub extendedmode: u32,
    pub writebuffers: u32,
    pub reserved: [u32; 4],
}

ioctl_read!(querycap, 'V', 0, Capabilities);
ioctl_readwrite!(enum_fmt, 'V', 2, FmtDesc);
ioctl_readwrite!(enuminput, 'V', 26, Input);
ioctl_readwrite!(enumoutput, 'V', 48, Output);
ioctl_readwrite!(g_fmt, 'V', 4, Format);
ioctl_readwrite!(s_fmt, 'V', 5, Format);
ioctl_readwrite!(queryctrl, 'V', 36, QueryCtrl);
ioctl_readwrite!(querymenu, 'V', 37, QueryMenu);
ioctl_readwrite!(reqbufs, 'V', 8, RequestBuffers);
ioctl_readwrite!(querybuf, 'V', 9, Buffer);
ioctl_readwrite!(qbuf, 'V', 15, Buffer);
ioctl_readwrite!(dqbuf, 'V', 17, Buffer);
ioctl_write_ptr!(streamon, 'V', 18, BufType);
ioctl_write_ptr!(streamoff, 'V', 19, BufType);
ioctl_readwrite!(s_parm, 'V', 22, StreamParm);
ioctl_readwrite!(g_ctrl, 'V', 27, controls::Control);
ioctl_readwrite!(s_ctrl, 'V', 28, controls::Control);
ioctl_readwrite!(enum_framesizes, 'V', 74, FrmSizeEnum);
ioctl_readwrite!(enum_frameintervals, 'V', 75, FrmIvalEnum);
