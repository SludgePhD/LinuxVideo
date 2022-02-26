ffi_enum! {
    pub enum CtrlClass: u32 {
        USER            = 0x00980000,
        CODEC           = 0x00990000,
        CAMERA          = 0x009a0000,
        FM_TX           = 0x009b0000,
        FLASH           = 0x009c0000,
        JPEG            = 0x009d0000,
        IMAGE_SOURCE    = 0x009e0000,
        IMAGE_PROC      = 0x009f0000,
        DV              = 0x00a00000,
        FM_RX           = 0x00a10000,
        RF_TUNER        = 0x00a20000,
        DETECT          = 0x00a30000,
        CODEC_STATELESS = 0x00a40000,
    }
}

ffi_enum! {
    /// Identifies a device control.
    ///
    /// This type has associated constants to refer to standard controls with predefined meanings,
    /// but drivers can add their own driver-specific controls as well.
    pub enum Cid: u32 {
        BRIGHTNESS                  = Self::BASE.0 + 0, // comes first so it shows up in debug output
        BASE                        = CtrlClass::USER.0 | 0x900,

        /// User-class control base ID.
        USER_BASE                   = Self::BASE.0,
        CONTRAST                    = Self::BASE.0 + 1,
        SATURATION                  = Self::BASE.0 + 2,
        HUE                         = Self::BASE.0 + 3,
        // missing: offset +4
        AUDIO_VOLUME                = Self::BASE.0 + 5,
        AUDIO_BALANCE               = Self::BASE.0 + 6,
        AUDIO_BASS                  = Self::BASE.0 + 7,
        AUDIO_TREBLE                = Self::BASE.0 + 8,
        AUDIO_MUTE                  = Self::BASE.0 + 9,
        AUDIO_LOUDNESS              = Self::BASE.0 + 10,
        /// Deprecated
        AUDIO_LEVEL                 = Self::BASE.0 + 11,
        AUTO_WHITE_BALANCE          = Self::BASE.0 + 12,
        DO_WHITE_BALANCE            = Self::BASE.0 + 13,
        RED_BALANCE                 = Self::BASE.0 + 14,
        BLUE_BALANCE                = Self::BASE.0 + 15,
        GAMMA                       = Self::BASE.0 + 16,
        WHITENESS                   = Self::GAMMA.0, // Deprecated
        EXPOSURE                    = Self::BASE.0 + 17,
        AUTOGAIN                    = Self::BASE.0 + 18,
        GAIN                        = Self::BASE.0 + 19,
        HFLIP                       = Self::BASE.0 + 20,
        VFLIP                       = Self::BASE.0 + 21,
        // gap
        POWER_LINE_FREQUENCY        = Self::BASE.0 + 24,
        HUE_AUTO                    = Self::BASE.0 + 25,
        WHITE_BALANCE_TEMPERATURE   = Self::BASE.0 + 26,
        SHARPNESS                   = Self::BASE.0 + 27,
        BACKLIGHT_COMPENSATION      = Self::BASE.0 + 28,
        CHROMA_AGC                  = Self::BASE.0 + 29,
        COLOR_KILLER                = Self::BASE.0 + 30,
        COLORFX                     = Self::BASE.0 + 31,
        AUTOBRIGHTNESS              = Self::BASE.0 + 32,
        BAND_STOP_FILTER            = Self::BASE.0 + 33,
        ROTATE                      = Self::BASE.0 + 34,
        BG_COLOR                    = Self::BASE.0 + 35,
        CHROMA_GAIN                 = Self::BASE.0 + 36,
        ILLUMINATORS_1              = Self::BASE.0 + 37,
        ILLUMINATORS_2              = Self::BASE.0 + 38,
        MIN_BUFFERS_FOR_CAPTURE     = Self::BASE.0 + 39,
        MIN_BUFFERS_FOR_OUTPUT      = Self::BASE.0 + 40,
        ALPHA_COMPONENT             = Self::BASE.0 + 41,
        COLORFX_CBCR                = Self::BASE.0 + 42,

        LASTP1                      = Self::BASE.0 + 43,

        /// Camera-class control base ID.
        CAMERA_CLASS_BASE           = CtrlClass::CAMERA.0 | 0x900,
        CAMERA_CLASS                = CtrlClass::CAMERA.0 | 1,
        EXPOSURE_AUTO               = Self::CAMERA_CLASS_BASE.0 + 1,
        EXPOSURE_ABSOLUTE           = Self::CAMERA_CLASS_BASE.0 + 2,
        EXPOSURE_AUTO_PRIORITY      = Self::CAMERA_CLASS_BASE.0 + 3,
        PAN_RELATIVE                = Self::CAMERA_CLASS_BASE.0 + 4,
        TILT_RELATIVE               = Self::CAMERA_CLASS_BASE.0 + 5,
        PAN_RESET                   = Self::CAMERA_CLASS_BASE.0 + 6,
        TILT_RESET                  = Self::CAMERA_CLASS_BASE.0 + 7,

        PAN_ABSOLUTE                = Self::CAMERA_CLASS_BASE.0 + 8,
        TILT_ABSOLUTE               = Self::CAMERA_CLASS_BASE.0 + 9,

        FOCUS_ABSOLUTE              = Self::CAMERA_CLASS_BASE.0 + 10,
        FOCUS_RELATIVE              = Self::CAMERA_CLASS_BASE.0 + 11,
        FOCUS_AUTO                  = Self::CAMERA_CLASS_BASE.0 + 12,

        ZOOM_ABSOLUTE               = Self::CAMERA_CLASS_BASE.0 + 13,
        ZOOM_RELATIVE               = Self::CAMERA_CLASS_BASE.0 + 14,
        ZOOM_CONTINUOUS             = Self::CAMERA_CLASS_BASE.0 + 15,

        PRIVACY                     = Self::CAMERA_CLASS_BASE.0 + 16,

        IRIS_ABSOLUTE               = Self::CAMERA_CLASS_BASE.0 + 17,
        IRIS_RELATIVE               = Self::CAMERA_CLASS_BASE.0 + 18,

        AUTO_EXPOSURE_BIAS          = Self::CAMERA_CLASS_BASE.0 + 19,
        AUTO_N_PRESET_WHITE_BALANCE = Self::CAMERA_CLASS_BASE.0 + 20,

        WIDE_DYNAMIC_RANGE          = Self::CAMERA_CLASS_BASE.0 + 21,
        IMAGE_STABILIZATION         = Self::CAMERA_CLASS_BASE.0 + 22,

        ISO_SENSITIVITY             = Self::CAMERA_CLASS_BASE.0 + 23,
        ISO_SENSITIVITY_AUTO        = Self::CAMERA_CLASS_BASE.0 + 24,

        EXPOSURE_METERING           = Self::CAMERA_CLASS_BASE.0 + 25,
        SCENE_MODE                  = Self::CAMERA_CLASS_BASE.0 + 26,

        CAMERA_3A_LOCK              = Self::CAMERA_CLASS_BASE.0 + 27,

        AUTO_FOCUS_START            = Self::CAMERA_CLASS_BASE.0 + 28,
        AUTO_FOCUS_STOP             = Self::CAMERA_CLASS_BASE.0 + 29,
        AUTO_FOCUS_STATUS           = Self::CAMERA_CLASS_BASE.0 + 30,
        AUTO_FOCUS_RANGE            = Self::CAMERA_CLASS_BASE.0 + 31,

        PAN_SPEED                   = Self::CAMERA_CLASS_BASE.0 + 32,
        TILT_SPEED                  = Self::CAMERA_CLASS_BASE.0 + 33,

        CAMERA_ORIENTATION          = Self::CAMERA_CLASS_BASE.0 + 34,
        CAMERA_SENSOR_ROTATION      = Self::CAMERA_CLASS_BASE.0 + 35,
    }
}

ffi_enum! {
    pub enum PowerLineFrequency: u32 {
        DISABLED  = 0,
        FREQ_50HZ = 1,
        FREQ_60HZ = 2,
        AUTO      = 3,
    }
}

ffi_enum! {
    pub enum ColorFx: u32 {
        NONE         = 0,
        BW           = 1,
        SEPIA        = 2,
        NEGATIVE     = 3,
        EMBOSS       = 4,
        SKETCH       = 5,
        SKY_BLUE     = 6,
        GRASS_GREEN  = 7,
        SKIN_WHITEN  = 8,
        VIVID        = 9,
        AQUA         = 10,
        ART_FREEZE   = 11,
        SILHOUETTE   = 12,
        SOLARIZATION = 13,
        ANTIQUE      = 14,
        SET_CBCR     = 15,
    }
}

#[repr(C)]
pub struct Control {
    pub id: Cid,
    pub value: i32,
}
