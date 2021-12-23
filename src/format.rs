use std::{fmt, mem};

use crate::{raw, BufType, Pixelformat};

#[derive(Debug)]
#[non_exhaustive]
pub enum Format {
    VideoCapture(PixFormat),
    VideoOutput(PixFormat),
    VideoCaptureMplane(PixFormatMplane),
    VideoOutputMplane(PixFormatMplane),
    VideoOverlay(Window),
    MetaCapture(MetaFormat),
    MetaOutput(MetaFormat),
    // TODO...
}

pub struct PixFormat(raw::PixFormat);

pub struct PixFormatMplane(raw::PixFormatMplane);

pub struct Window(raw::Window);

pub struct PlanePixFormat(raw::PlanePixFormat);

pub struct MetaFormat(raw::MetaFormat);

impl Format {
    pub(crate) unsafe fn from_raw(raw: raw::Format) -> Option<Self> {
        Some(match raw.type_ {
            BufType::VIDEO_CAPTURE => Self::VideoCapture(PixFormat(raw.fmt.pix)),
            BufType::VIDEO_OUTPUT => Self::VideoOutput(PixFormat(raw.fmt.pix)),
            BufType::VIDEO_CAPTURE_MPLANE => {
                Self::VideoCaptureMplane(PixFormatMplane(raw.fmt.pix_mp))
            }
            BufType::VIDEO_OUTPUT_MPLANE => {
                Self::VideoOutputMplane(PixFormatMplane(raw.fmt.pix_mp))
            }
            BufType::VIDEO_OVERLAY => Self::VideoOverlay(Window(raw.fmt.win)),
            BufType::META_CAPTURE => Self::MetaCapture(MetaFormat(raw.fmt.meta)),
            _ => return None,
        })
    }
}

impl PixFormat {
    pub fn new(width: u32, height: u32, pixelformat: Pixelformat) -> Self {
        Self(raw::PixFormat {
            width,
            height,
            pixelformat,
            ..unsafe { mem::zeroed() }
        })
    }

    pub(crate) fn to_raw(self) -> raw::PixFormat {
        self.0
    }

    pub fn width(&self) -> u32 {
        self.0.width
    }

    pub fn height(&self) -> u32 {
        self.0.height
    }

    pub fn pixelformat(&self) -> Pixelformat {
        self.0.pixelformat
    }

    pub fn bytes_per_line(&self) -> u32 {
        self.0.bytesperline
    }

    pub fn size_image(&self) -> u32 {
        self.0.sizeimage
    }
}

impl PixFormatMplane {
    pub(crate) fn to_raw(self) -> raw::PixFormatMplane {
        self.0
    }

    pub fn num_planes(&self) -> usize {
        self.0.num_planes.into()
    }

    pub fn plane_formats(&self) -> impl Iterator<Item = PlanePixFormat> + '_ {
        // NB: this cannot return `&[PlanePixFormat]` because the underlying data is unaligned
        (0..self.num_planes()).map(move |i| PlanePixFormat(self.0.plane_fmt[i]))
    }
}

impl PlanePixFormat {
    pub fn size_image(&self) -> u32 {
        self.0.sizeimage
    }

    pub fn bytes_per_line(&self) -> u32 {
        self.0.bytesperline
    }
}

impl Window {
    pub(crate) fn to_raw(self) -> raw::Window {
        self.0
    }
}

impl MetaFormat {
    pub fn new(pixelformat: Pixelformat) -> Self {
        Self(raw::MetaFormat {
            dataformat: pixelformat,
            buffersize: 0, // set by driver during `S_FMT`
        })
    }

    pub fn buffer_size(&self) -> u32 {
        self.0.buffersize
    }

    pub(crate) fn to_raw(self) -> raw::MetaFormat {
        self.0
    }
}

impl fmt::Debug for PixFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixFormat")
            .field("width", &self.0.width)
            .field("height", &self.0.height)
            .field("pixelformat", &self.0.pixelformat)
            .field("field", &self.0.field)
            .field("bytesperline", &self.0.bytesperline)
            .field("sizeimage", &self.0.sizeimage)
            .field("colorspace", &self.0.colorspace)
            .field("flags", &self.0.flags)
            .field("enc", &self.0.enc)
            .field("quantization", &self.0.quantization)
            .field("xfer_func", &self.0.xfer_func)
            .finish()
    }
}

impl fmt::Debug for PixFormatMplane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixFormatMplane")
            .field("width", &{ self.0.width })
            .field("height", &{ self.0.height })
            .field("pixelformat", &{ self.0.pixelformat })
            .field("field", &{ self.0.field })
            .field("colorspace", &{ self.0.colorspace })
            .field("plane_fmt", &self.plane_formats().collect::<Vec<_>>())
            .field("num_planes", &self.0.num_planes)
            .finish()
    }
}

impl fmt::Debug for PlanePixFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlanePixFormat")
            .field("sizeimage", &{ self.0.sizeimage })
            .field("bytesperline", &{ self.0.bytesperline })
            .finish()
    }
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("rect", &self.0.w)
            .field("field", &self.0.field)
            .field("chromakey", &self.0.chromakey)
            .field("clips", &self.0.clips)
            .field("clipcount", &self.0.clipcount)
            .field("bitmap", &self.0.bitmap)
            .field("global_alpha", &self.0.global_alpha)
            .finish()
    }
}

impl fmt::Debug for MetaFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaFormat")
            .field("dataformat", &{ self.0.dataformat })
            .field("buffersize", &{ self.0.buffersize })
            .finish()
    }
}
