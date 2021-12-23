//! FFI-compatible types that may also be exposed to Rust code.

use bitflags::bitflags;

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
    pub enum OutputType: u32 {
        /// This output is an analog TV modulator.
        MODULATOR = 1,
        /// Any non-modulator video output, for example Composite Video, S-Video, HDMI.
        ANALOG = 2,
        /// The video output will be copied to a video overlay.
        ANALOGVGAOVERLAY = 3,
    }
}

ffi_enum! {
    pub enum InputType: u32 {
        TUNER = 1,
        CAMERA = 2,
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
        MMAP    = 1,
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

bitflags! {
    pub struct CtrlFlags: u32 {
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
    #[repr(transparent)]
    pub struct FmtFlags: u32 {
        /// This is a compressed format.
        const COMPRESSED             = 0x0001;
        /// This format is not native to the device but emulated through software (usually libv4l2),
        /// where possible try to use a native format instead for better performance.
        const EMULATED               = 0x0002;
        /// This flag can only be used in combination with the `V4L2_FMT_FLAG_COMPRESSED` flag,
        /// since this applies to compressed formats only. This flag is valid for stateful decoders
        /// only.
        const CONTINUOUS_BYTESTREAM  = 0x0004;
        /// Dynamic resolution switching is supported by the device for this compressed bytestream
        /// format (aka coded format). It will notify the user via the event
        /// `V4L2_EVENT_SOURCE_CHANGE` when changes in the video parameters are detected.
        ///
        /// This flag can only be used in combination with the `V4L2_FMT_FLAG_COMPRESSED` flag,
        /// since this applies to compressed formats only. This flag is valid for stateful codecs
        /// only.
        const DYN_RESOLUTION         = 0x0008;
        /// The hardware encoder supports setting the CAPTURE coded frame interval separately from
        /// the OUTPUT raw frame interval. Setting the OUTPUT raw frame interval with VIDIOC_S_PARM
        /// also sets the CAPTURE coded frame interval to the same value. If this flag is set, then
        /// the CAPTURE coded frame interval can be set to a different value afterwards. This is
        /// typically used for offline encoding where the OUTPUT raw frame interval is used as a
        /// hint for reserving hardware encoder resources and the CAPTURE coded frame interval is
        /// the actual frame rate embedded in the encoded video stream.
        ///
        /// This flag can only be used in combination with the V4L2_FMT_FLAG_COMPRESSED flag, since
        /// this applies to compressed formats only. This flag is valid for stateful encoders only.
        const ENC_CAP_FRAME_INTERVAL = 0x0010;
        /// The driver allows the application to try to change the default colorspace. This flag is
        /// relevant only for capture devices. The application can ask to configure the colorspace
        /// of the capture device when calling the VIDIOC_S_FMT ioctl with V4L2_PIX_FMT_FLAG_SET_CSC
        /// set.
        const CSC_COLORSPACE         = 0x0020;
        /// The driver allows the application to try to change the default transfer function. This
        /// flag is relevant only for capture devices. The application can ask to configure the
        /// transfer function of the capture device when calling the VIDIOC_S_FMT ioctl with
        /// V4L2_PIX_FMT_FLAG_SET_CSC set.
        const CSC_XFER_FUNC          = 0x0040;
        /// The driver allows the application to try to change the default Y’CbCr encoding. This
        /// flag is relevant only for capture devices. The application can ask to configure the
        /// Y’CbCr encoding of the capture device when calling the VIDIOC_S_FMT ioctl with
        /// V4L2_PIX_FMT_FLAG_SET_CSC set.
        const CSC_YCBCR_ENC          = 0x0080;
        /// The driver allows the application to try to change the default HSV encoding. This flag
        /// is relevant only for capture devices. The application can ask to configure the HSV
        /// encoding of the capture device when calling the VIDIOC_S_FMT ioctl with
        /// V4L2_PIX_FMT_FLAG_SET_CSC set.
        const CSC_HSV_ENC            = Self::CSC_YCBCR_ENC.bits;
        /// The driver allows the application to try to change the default quantization. This flag
        /// is relevant only for capture devices. The application can ask to configure the
        /// quantization of the capture device when calling the VIDIOC_S_FMT ioctl with
        /// V4L2_PIX_FMT_FLAG_SET_CSC set.
        const CSC_QUANTIZATION       = 0x0100;
    }
}

bitflags! {
    #[repr(transparent)]
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
    #[repr(transparent)]
    pub struct OutputCapabilities: u32 {
        /// This output supports setting video timings by using VIDIOC_S_DV_TIMINGS.
        const DV_TIMINGS     = 0x00000002;
        const CUSTOM_TIMINGS = Self::DV_TIMINGS.bits;
        /// This output supports setting the TV standard by using VIDIOC_S_STD.
        const STD            = 0x00000004;
        /// This output supports setting the native size using the V4L2_SEL_TGT_NATIVE_SIZE
        /// selection target, see Common selection definitions.
        const NATIVE_SIZE    = 0x00000008;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct InputCapabilities: u32 {
        /// This input supports setting video timings by using VIDIOC_S_DV_TIMINGS.
        const DV_TIMINGS     = 0x00000002;
        const CUSTOM_TIMINGS = Self::DV_TIMINGS.bits;
        /// This input supports setting the TV standard by using VIDIOC_S_STD.
        const STD            = 0x00000004;
        /// This input supports setting the native size using the V4L2_SEL_TGT_NATIVE_SIZE
        /// selection target, see Common selection definitions.
        const NATIVE_SIZE    = 0x00000008;
    }
}

bitflags! {
    /// Device capabilities.
    #[repr(transparent)]
    pub struct CapabilityFlags: u32 {
        /// The device supports the single-planar API through the Video Capture interface.
        const VIDEO_CAPTURE        = 0x00000001;
        /// The device supports the single-planar API through the Video Output interface.
        const VIDEO_OUTPUT         = 0x00000002;
        /// The device supports the Video Overlay interface. A video overlay device typically stores
        /// captured images directly in the video memory of a graphics card, with hardware clipping
        /// and scaling.
        const VIDEO_OVERLAY        = 0x00000004;
        /// The device supports the Raw VBI Capture interface, providing Teletext and Closed Caption
        /// data.
        const VBI_CAPTURE          = 0x00000010;
        /// The device supports the Raw VBI Output interface.
        const VBI_OUTPUT           = 0x00000020;
        /// The device supports the Sliced VBI Capture interface.
        const SLICED_VBI_CAPTURE   = 0x00000040;
        /// The device supports the Sliced VBI Output interface.
        const SLICED_VBI_OUTPUT    = 0x00000080;
        /// The device supports the Radio Data System capture interface.
        const RDS_CAPTURE          = 0x00000100;
        /// The device supports the Video Output Overlay (OSD) interface. Unlike the Video Overlay
        /// interface, this is a secondary function of video output devices and overlays an image
        /// onto an outgoing video signal. When the driver sets this flag, it must clear the
        /// `V4L2_CAP_VIDEO_OVERLAY` flag and vice versa.
        ///
        /// The struct v4l2_framebuffer lacks an enum v4l2_buf_type field, therefore the type of
        /// overlay is implied by the driver capabilities.
        const VIDEO_OUTPUT_OVERLAY = 0x00000200;
        /// The device supports the ioctl VIDIOC_S_HW_FREQ_SEEK ioctl for hardware frequency
        /// seeking.
        const HW_FREQ_SEEK         = 0x00000400;
        /// The device supports the RDS output interface.
        const RDS_OUTPUT           = 0x00000800;

        /// The device supports the multi-planar API through the Video Capture interface.
        const VIDEO_CAPTURE_MPLANE = 0x00001000;
        /// The device supports the multi-planar API through the Video Output interface.
        const VIDEO_OUTPUT_MPLANE  = 0x00002000;
        /// The device supports the multi-planar API through the Video Memory-To-Memory interface.
        const VIDEO_M2M_MPLANE     = 0x00004000;
        /// The device supports the single-planar API through the Video Memory-To-Memory interface.
        const VIDEO_M2M            = 0x00008000;

        /// The device has some sort of tuner to receive RF-modulated video signals. For more
        /// information about tuner programming see Tuners and Modulators.
        const TUNER                = 0x00010000;
        /// The device has audio inputs or outputs. It may or may not support audio recording or
        /// playback, in PCM or compressed formats. PCM audio support must be implemented as ALSA or
        /// OSS interface. For more information on audio inputs and outputs see Audio Inputs and
        /// Outputs.
        const AUDIO                = 0x00020000;
        /// This is a radio receiver.
        const RADIO                = 0x00040000;
        /// The device has some sort of modulator to emit RF-modulated video/audio signals. For more
        /// information about modulator programming see Tuners and Modulators.
        const MODULATOR            = 0x00080000;

        /// The device supports the SDR Capture interface.
        const SDR_CAPTURE          = 0x00100000;
        /// The device supports the struct v4l2_pix_format extended fields.
        const EXT_PIX_FORMAT       = 0x00200000;
        /// The device supports the SDR Output interface.
        const SDR_OUTPUT           = 0x00400000;
        /// The device supports the Metadata Interface capture interface.
        const META_CAPTURE         = 0x00800000;

        /// The device supports the `read()` and/or `write()` I/O methods.
        const READWRITE            = 0x01000000;
        /// The device supports the asynchronous I/O methods.
        const ASYNCIO              = 0x02000000;
        /// The device supports (some of) the streaming I/O methods.
        const STREAMING            = 0x04000000;
        /// The device supports the Metadata Interface output interface.
        const META_OUTPUT          = 0x08000000;

        /// This is a touch device.
        const TOUCH                = 0x10000000;
        /// There is only one input and/or output seen from userspace. The whole video topology
        /// configuration, including which I/O entity is routed to the input/output, is configured
        /// by userspace via the Media Controller. See Part IV - Media Controller API.
        const IO_MC                = 0x20000000;
        /// The driver fills the device_caps field. This capability can only appear in the
        /// capabilities field and never in the device_caps field.
        const DEVICE_CAPS          = 0x80000000;
    }
}

bitflags! {
    #[repr(transparent)]
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
    #[repr(transparent)]
    pub struct VbiFlags: u32 {
        const UNSYNC     = 1 << 0;
        const INTERLACED = 1 << 1;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct ServiceSet: u32 {
        const TELETEXT_B  = 0x0001;
        const VPS         = 0x0400;
        const CAPTION_525 = 0x1000;
        const WSS_625     = 0x4000;
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct PixFmtFlag: u32 {
        const PREMUL_ALPHA = 0x00000001;
        const SET_CSC      = 0x00000002;
    }
}

bitflags! {
    #[repr(transparent)]
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
    #[repr(transparent)]
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
    #[repr(transparent)]
    pub struct TimecodeFlags: u32 {
        const DROPFRAME            = 0x0001;
        const COLORFRAME           = 0x0002;
        const USERBITS_MASK        = 0x000C;
        const USERBITS_USERDEFINED = 0x0000;
        const USERBITS_8BITCHARS   = 0x0008;
    }
}
