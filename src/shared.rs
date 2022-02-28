//! FFI-compatible types that may also be exposed to Rust code.

use std::fmt;

// This macro enforces that all `bitflags!` types in here are marked
// `#[repr(transparent)]` and thus FFI-safe.
macro_rules! bitflags {
    ($($t:tt)*) => {
        bitflags::bitflags! {
            #[repr(transparent)]
            $($t)*
        }
    };
}

ffi_enum! {
    pub enum TunerType: u32 {
        RADIO      = 1,
        ANALOG_TV  = 2,
        DIGITAL_TV = 3,
        SDR        = 4,
        RF         = 5,
    }
}

ffi_enum! {
    /// Describes what kind of device an [`Output`][crate::Output] is.
    pub enum OutputType: u32 {
        /// This output is an analog TV modulator.
        MODULATOR = 1,
        /// Non-modulated analog TV, or a digital signal.
        ANALOG = 2,
        /// The video output will be copied to a video overlay.
        ANALOGVGAOVERLAY = 3,
    }
}

ffi_enum! {
    /// Describes what kind of device an [`Input`][crate::Input] is.
    pub enum InputType: u32 {
        /// The input is an RF Demodulator/Tuner.
        TUNER = 1,
        /// Input is a camera, HDMI capture device, or another non-tuner input.
        CAMERA = 2,
        /// The input is a touch screen or surface.
        TOUCH = 3,
    }
}

ffi_enum! {
    pub enum Colorspace: u32 {
        DEFAULT        = 0,
        SMPTE170M      = 1,
        SMPTE240M      = 2,
        REC709         = 3,
        BT878          = 4,
        _470_SYSTEM_M  = 5,
        _470_SYSTEM_BG = 6,
        JPEG           = 7,
        SRGB           = 8,
        OPRGB          = 9,
        BT2020         = 10,
        RAW            = 11,
        DCI_P3         = 12,
    }
}

ffi_enum! {
    pub enum Quantization: u32 {
        DEFAULT    = 0,
        FULL_RANGE = 1,
        LIM_RANGE  = 2,
    }
}

ffi_enum! {
    pub enum XferFunc: u32 {
        DEFAULT   = 0,
        _709      = 1,
        SRGB      = 2,
        OPRGB     = 3,
        SMPTE240M = 4,
        NONE      = 5,
        DCI_P3    = 6,
        SMPTE2084 = 7,
    }
}

ffi_enum! {
    pub enum Field: u32 {
        /// Lets the driver choose.
        ANY           = 0,
        /// Don't use fields.
        NONE          = 1,
        TOP           = 2,
        BOTTOM        = 3,
        INTERLACED    = 4,
        SEQ_TB        = 5,
        SEQ_BT        = 6,
        ALTERNATE     = 7,
        INTERLACED_TB = 8,
        INTERLACED_BT = 9,
    }
}

ffi_enum! {
    pub enum CtrlType: u32 {
        INTEGER             = 1,
        BOOLEAN             = 2,
        MENU                = 3,
        BUTTON              = 4,
        INTEGER64           = 5,
        CTRL_CLASS          = 6,
        STRING              = 7,
        BITMASK             = 8,
        INTEGER_MENU        = 9,

        //COMPOUND_TYPES    = 0x0100,
        U8                  = 0x0100,
        U16                 = 0x0101,
        U32                 = 0x0102,
        AREA                = 0x0106,

        H264_SPS            = 0x0200,
        H264_PPS            = 0x0201,
        H264_SCALING_MATRIX = 0x0202,
        H264_SLICE_PARAMS   = 0x0203,
        H264_DECODE_PARAMS  = 0x0204,
        H264_PRED_WEIGHTS   = 0x0205,

        FWHT_PARAMS         = 0x0220,
    }
}

ffi_enum! {
    pub enum Memory: u32 {
        /// Buffers are allocated by the driver and `mmap`ped into userspace.
        MMAP    = 1,
        /// Buffers are allocated by userspace and a pointer is passed to the driver.
        USERPTR = 2,
        OVERLAY = 3,
        DMABUF  = 4,
    }
}

ffi_enum! {
    pub enum TimecodeType: u32 {
        T_24FPS = 1,
        T_25FPS = 2,
        T_30FPS = 3,
        T_50FPS = 4,
        T_60FPS = 5,
    }
}

ffi_enum! {
    pub enum FrmSizeType: u32 {
        DISCRETE = 1,
        CONTINUOUS = 2,
        STEPWISE = 3,
    }
}

ffi_enum! {
    pub enum FrmIvalType: u32 {
        DISCRETE = 1,
        CONTINUOUS = 2,
        STEPWISE = 3,
    }
}

bitflags! {
    /// Flags describing the state of a device control.
    pub struct ControlFlags: u32 {
        const DISABLED         = 0x0001;
        const GRABBED          = 0x0002;
        const READ_ONLY        = 0x0004;
        const UPDATE           = 0x0008;
        const INACTIVE         = 0x0010;
        const SLIDER           = 0x0020;
        const WRITE_ONLY       = 0x0040;
        const VOLATILE         = 0x0080;
        const HAS_PAYLOAD      = 0x0100;
        const EXECUTE_ON_WRITE = 0x0200;
        const MODIFY_LAYOUT    = 0x0400;

        const NEXT_CTRL        = 0x80000000;
    }
}

bitflags! {
    pub struct FmtFlags: u32 {
        const COMPRESSED             = 0x0001;
        const EMULATED               = 0x0002;
        const CONTINUOUS_BYTESTREAM  = 0x0004;
        const DYN_RESOLUTION         = 0x0008;
        const ENC_CAP_FRAME_INTERVAL = 0x0010;
        const CSC_COLORSPACE         = 0x0020;
        const CSC_XFER_FUNC          = 0x0040;
        const CSC_YCBCR_ENC          = 0x0080;
        const CSC_HSV_ENC            = Self::CSC_YCBCR_ENC.bits;
        const CSC_QUANTIZATION       = 0x0100;
    }
}

bitflags! {
    /// Analog video standards.
    pub struct AnalogStd: u64 { // NB: this is v4l2_std_id
        const PAL_B       = 0x0000001;
        const PAL_B1      = 0x0000002;
        const PAL_G       = 0x0000004;
        const PAL_H       = 0x0000008;
        const PAL_I       = 0x0000010;
        const PAL_D       = 0x0000020;
        const PAL_D1      = 0x0000040;
        const PAL_K       = 0x0000080;

        const PAL_M       = 0x0000100;
        const PAL_N       = 0x0000200;
        const PAL_NC      = 0x0000400;
        const PAL_60      = 0x0000800;

        const NTSC_M      = 0x00001000;
        const NTSC_M_JP   = 0x00002000;
        const NTSC_443    = 0x00004000;
        const NTSC_M_KR   = 0x00008000;

        const SECAM_B     = 0x00010000;
        const SECAM_D     = 0x00020000;
        const SECAM_G     = 0x00040000;
        const SECAM_H     = 0x00080000;
        const SECAM_K     = 0x00100000;
        const SECAM_K1    = 0x00200000;
        const SECAM_L     = 0x00400000;
        const SECAM_LC    = 0x00800000;

        const ATSC_8_VSB  = 0x01000000;
        const ATSC_16_VSB = 0x02000000;
    }
}

bitflags! {
    pub struct OutputCapabilities: u32 {
        /// The output allows configuring video timings via `VIDIOC_S_DV_TIMINGS`.
        const DV_TIMINGS     = 0x00000002;
        const CUSTOM_TIMINGS = Self::DV_TIMINGS.bits;
        const STD            = 0x00000004;
        const NATIVE_SIZE    = 0x00000008;
    }
}

bitflags! {
    pub struct InputCapabilities: u32 {
        const DV_TIMINGS     = 0x00000002;
        const CUSTOM_TIMINGS = Self::DV_TIMINGS.bits;
        const STD            = 0x00000004;
        const NATIVE_SIZE    = 0x00000008;
    }
}

bitflags! {
    /// Device capabilities.
    pub struct CapabilityFlags: u32 {
        const VIDEO_CAPTURE        = 0x00000001;
        const VIDEO_OUTPUT         = 0x00000002;
        const VIDEO_OVERLAY        = 0x00000004;
        const VBI_CAPTURE          = 0x00000010;
        const VBI_OUTPUT           = 0x00000020;
        const SLICED_VBI_CAPTURE   = 0x00000040;
        const SLICED_VBI_OUTPUT    = 0x00000080;
        const RDS_CAPTURE          = 0x00000100;
        const VIDEO_OUTPUT_OVERLAY = 0x00000200;
        const HW_FREQ_SEEK         = 0x00000400;
        const RDS_OUTPUT           = 0x00000800;

        const VIDEO_CAPTURE_MPLANE = 0x00001000;
        const VIDEO_OUTPUT_MPLANE  = 0x00002000;
        const VIDEO_M2M_MPLANE     = 0x00004000;
        const VIDEO_M2M            = 0x00008000;

        const TUNER                = 0x00010000;
        const AUDIO                = 0x00020000;
        const RADIO                = 0x00040000;
        const MODULATOR            = 0x00080000;

        const SDR_CAPTURE          = 0x00100000;
        const EXT_PIX_FORMAT       = 0x00200000;
        const SDR_OUTPUT           = 0x00400000;
        const META_CAPTURE         = 0x00800000;

        const READWRITE            = 0x01000000;
        const ASYNCIO              = 0x02000000;
        const STREAMING            = 0x04000000;
        const META_OUTPUT          = 0x08000000;

        const TOUCH                = 0x10000000;
        const IO_MC                = 0x20000000;
        const DEVICE_CAPS          = 0x80000000;
    }
}

bitflags! {
    pub struct InputStatus: u32 {
        const NO_POWER   = 0x00000001;
        const NO_SIGNAL  = 0x00000002;
        const NO_COLOR   = 0x00000004;

        const HFLIP      = 0x00000010;
        const VFLIP      = 0x00000020;

        const NO_H_LOCK   = 0x00000100;
        const COLOR_KILL  = 0x00000200;
        const NO_V_LOCK   = 0x00000400;
        const NO_STD_LOCK = 0x00000800;

        const NO_SYNC     = 0x00010000;
        const NO_EQU      = 0x00020000;
        const NO_CARRIER  = 0x00040000;

        const MACROVISION = 0x01000000;
        const NO_ACCESS   = 0x02000000;
        const VTR         = 0x04000000;
    }
}

bitflags! {
    pub struct VbiFlags: u32 {
        const UNSYNC     = 1 << 0;
        const INTERLACED = 1 << 1;
    }
}

bitflags! {
    pub struct ServiceSet: u32 {
        const TELETEXT_B  = 0x0001;
        const VPS         = 0x0400;
        const CAPTION_525 = 0x1000;
        const WSS_625     = 0x4000;
    }
}

bitflags! {
    pub struct PixFmtFlag: u32 {
        const PREMUL_ALPHA = 0x00000001;
        const SET_CSC      = 0x00000002;
    }
}

bitflags! {
    pub struct BufCap: u32 {
        const SUPPORTS_MMAP                 = 1 << 0;
        const SUPPORTS_USERPTR              = 1 << 1;
        const SUPPORTS_DMABUF               = 1 << 2;
        const SUPPORTS_REQUESTS             = 1 << 3;
        const SUPPORTS_ORPHANED_BUFS        = 1 << 4;
        const SUPPORTS_M2M_HOLD_CAPTURE_BUF = 1 << 5;
        const SUPPORTS_MMAP_CACHE_HINTS     = 1 << 6;
    }
}

bitflags! {
    pub struct BufFlag: u32 {
        const MAPPED               = 0x00000001;
        const QUEUED               = 0x00000002;
        const DONE                 = 0x00000004;
        const KEYFRAME             = 0x00000008;
        const PFRAME               = 0x00000010;
        const BFRAME               = 0x00000020;
        const ERROR                = 0x00000040;
        const IN_REQUEST           = 0x00000080;
        const TIMECODE             = 0x00000100;
        const M2M_HOLD_CAPTURE_BUF = 0x00000200;
        const PREPARED             = 0x00000400;
        const NO_CACHE_INVALIDATE  = 0x00000800;
        const NO_CACHE_CLEAN       = 0x00001000;
        const TIMESTAMP_MASK       = 0x0000e000;
        const TIMESTAMP_UNKNOWN    = 0x00000000;
        const TIMESTAMP_MONOTONIC  = 0x00002000;
        const TIMESTAMP_COPY       = 0x00004000;
        const TIMESTAMP_SRC_MASK   = 0x00070000;
        const TIMESTAMP_SRC_EOF    = 0x00000000;
        const TIMESTAMP_SRC_SOE    = 0x00010000;
        const LAST                 = 0x00100000;
        const REQUEST_FD           = 0x00800000;
    }
}

bitflags! {
    pub struct TimecodeFlags: u32 {
        const DROPFRAME            = 0x0001;
        const COLORFRAME           = 0x0002;
        const USERBITS_MASK        = 0x000C;
        const USERBITS_USERDEFINED = 0x0000;
        const USERBITS_8BITCHARS   = 0x0008;
    }
}

bitflags! {
    pub struct StreamParamCaps: u32 {
        const TIMEPERFRAME = 0x1000;
    }
}

bitflags! {
    pub struct CaptureParamFlags: u32 {
        const HIGHQUALITY = 0x0001;
    }
}

/// A fractional value (`numerator / denominator`).
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Fract {
    numerator: u32,
    denominator: u32,
}

impl Fract {
    #[inline]
    pub fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    #[inline]
    pub fn numerator(&self) -> u32 {
        self.numerator
    }

    #[inline]
    pub fn denominator(&self) -> u32 {
        self.denominator
    }

    /// Returns this fraction as an `f32`.
    #[inline]
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

impl fmt::Display for Fract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl fmt::Debug for Fract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
