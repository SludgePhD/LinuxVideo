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
    /// Data types supported by a device control.
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

        VP8_FRAME           = 0x0240,

        MPEG2_QUANTISATION  = 0x0250,
        MPEG2_SEQUENCE      = 0x0251,
        MPEG2_PICTURE       = 0x0252,
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
        /// The control is disabled and cannot be modified.
        const DISABLED         = 0x0001;
        /// The control is in use by another application and cannot be modified.
        const GRABBED          = 0x0002;
        /// The value of this control cannot be changed.
        const READ_ONLY        = 0x0004;
        /// Modifying the value of this control may change the value of other controls in the same
        /// control class.
        const UPDATE           = 0x0008;
        /// The control is not available in the current device configuration.
        const INACTIVE         = 0x0010;
        /// The control's value is best displayed as a slider-like control in a UI.
        const SLIDER           = 0x0020;
        /// The control's value is not readable.
        const WRITE_ONLY       = 0x0040;
        /// The value of the control may change spuriously, even without writing to it.
        const VOLATILE         = 0x0080;
        /// The control's value is a non-scalar type behind a pointer.
        const HAS_PAYLOAD      = 0x0100;
        /// Setting the control's value will propagate to the driver, even when setting it to its
        /// current value.
        ///
        /// This is typically set for "trigger" controls that execute a device action when set.
        const EXECUTE_ON_WRITE = 0x0200;
        /// Modifying this control's value may change the video buffer layout.
        const MODIFY_LAYOUT    = 0x0400;

        // Used internally, but not of interest to users of this library.
        //const NEXT_CTRL        = 0x80000000;
    }
}

pub(crate) const CONTROL_FLAGS_NEXT_CTRL: u32 = 0x80000000;

bitflags! {
    pub struct FormatFlags: u32 {
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
    /// Describes the capabilities of a device [`Output`][crate::Output].
    pub struct OutputCapabilities: u32 {
        /// The output allows configuring video timings via `VIDIOC_S_DV_TIMINGS`.
        const DV_TIMINGS     = 0x00000002;
        const CUSTOM_TIMINGS = Self::DV_TIMINGS.bits;
        const STD            = 0x00000004;
        const NATIVE_SIZE    = 0x00000008;
    }
}

bitflags! {
    /// Describes the capabilities of a device [`Input`][crate::Input].
    pub struct InputCapabilities: u32 {
        /// The input allows configuring video timings via `VIDIOC_S_DV_TIMINGS`.
        const DV_TIMINGS     = 0x00000002;
        const CUSTOM_TIMINGS = Self::DV_TIMINGS.bits;
        const STD            = 0x00000004;
        const NATIVE_SIZE    = 0x00000008;
    }
}

bitflags! {
    /// Device capabilities.
    pub struct CapabilityFlags: u32 {
        /// Device supports capturing video data via [`Device::video_capture`][crate::Device::video_capture].
        const VIDEO_CAPTURE        = 0x00000001;
        /// Device supports outputting video data via [`Device::video_output`][crate::Device::video_output].
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
        /// Device supports capturing metadata via [`Device::meta_capture`][crate::Device::meta_capture].
        const META_CAPTURE         = 0x00800000;

        /// Device supports the read/write based I/O method.
        const READWRITE            = 0x01000000;
        /// Device supports asynchronous I/O (this is apparently not yet defined or used though).
        const ASYNCIO              = 0x02000000;
        /// Device supports the buffer-based streaming I/O method.
        const STREAMING            = 0x04000000;
        const META_OUTPUT          = 0x08000000;

        /// Device is a touch input device.
        const TOUCH                = 0x10000000;
        /// Device uses the Media Controller API for configuration.
        const IO_MC                = 0x20000000;
        /// Driver supports per-device capabilities.
        const DEVICE_CAPS          = 0x80000000;
    }
}

bitflags! {
    /// Bitflags describing the current status of a device [`Input`][crate::Input].
    pub struct InputStatus: u32 {
        /// Input has no power and is turned off.
        const NO_POWER   = 0x00000001;
        /// Input is not receiving a video signal.
        const NO_SIGNAL  = 0x00000002;
        /// The input signal contains no color data.
        const NO_COLOR   = 0x00000004;

        /// The input produces a horizontally flipped image which needs to be corrected in
        /// userspace.
        const HFLIP      = 0x00000010;
        /// The input produces a vertically flipped image which needs to be corrected in userspace.
        const VFLIP      = 0x00000020;

        /// Analog Input: Not locked to HSYNC.
        const NO_H_LOCK   = 0x00000100;
        /// Analog Input: The input's [Color Killer](https://en.wikipedia.org/wiki/Color_killer) circuit is
        /// active.
        const COLOR_KILL  = 0x00000200;
        /// Analog Input: Not locked to VSYNC.
        const NO_V_LOCK   = 0x00000400;
        /// Analog Input: No analog standard lock (when using auto-detection of the format).
        const NO_STD_LOCK = 0x00000800;

        /// Digital Input: Not synced to video data.
        const NO_SYNC     = 0x00010000;
        /// Digital Input: No equalizer lock.
        const NO_EQU      = 0x00020000;
        /// Digital Input: No carrier recovered.
        const NO_CARRIER  = 0x00040000;

        /// VCR Input: Macrovision copy protection detected.
        const MACROVISION = 0x01000000;
        /// Access to the video stream was denied.
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
#[derive(Clone, Copy, Hash)]
#[repr(C)]
pub struct Fract {
    numerator: u32,
    denominator: u32,
}

impl Fract {
    #[inline]
    pub fn new(numerator: u32, denominator: u32) -> Self {
        assert_ne!(denominator, 0, "denominator must not be zero");
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
        fmt::Display::fmt(self, f)
    }
}

impl PartialEq for Fract {
    fn eq(&self, other: &Self) -> bool {
        let [a, b] = same_denom(*self, *other);
        a.numerator == b.numerator
    }
}

impl Eq for Fract {}

impl PartialOrd for Fract {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let [a, b] = same_denom(*self, *other);
        a.numerator.partial_cmp(&b.numerator)
    }
}

impl Ord for Fract {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let [a, b] = same_denom(*self, *other);
        a.numerator.cmp(&b.numerator)
    }
}

fn same_denom(f1: Fract, f2: Fract) -> [Fract; 2] {
    let multiple = lcm(f1.denominator, f2.denominator);
    [
        Fract::new(f1.numerator * (multiple / f1.denominator), multiple),
        Fract::new(f2.numerator * (multiple / f2.denominator), multiple),
    ]
}

const fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b > 0 {
        let t = b;
        b = a % b;
        a = t;
    }

    a
}

const fn lcm(a: u32, b: u32) -> u32 {
    a * b / gcd(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(6, 9), 3);
        assert_eq!(gcd(7, 13), 1);
        assert_eq!(1920 / gcd(1920, 1080), 16);
        assert_eq!(1080 / gcd(1920, 1080), 9);

        // degenerate case where one of the arguments is 0 - the other one will be returned
        assert_eq!(gcd(0, 7), 7);
        assert_eq!(gcd(7, 0), 7);
        assert_eq!(gcd(0, 0), 0);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(1, 1), 1);
        assert_eq!(lcm(1, 3), 3);
        assert_eq!(lcm(3, 1), 3);

        assert_eq!(lcm(3, 5), 15);
        assert_eq!(lcm(5, 3), 15);
    }

    #[test]
    fn test_same_denom() {
        let a = Fract::new(2, 3);
        let b = Fract::new(3, 5);
        let [x, y] = same_denom(a, b);
        assert_eq!(x.numerator, 10);
        assert_eq!(x.denominator, 15);
        assert_eq!(y.numerator, 9);
        assert_eq!(y.denominator, 15);
    }
}
