//! Image and pixel formats.

use std::{fmt, io, mem};

use nix::errno::Errno;

use crate::shared::{FrmIvalType, FrmSizeType};
use crate::{byte_array_to_str, raw, BufType, Device, Fract};

pub use crate::pixel_format::PixelFormat;
pub use crate::shared::FormatFlags;

/// Formats of all possible buffer types.
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

/// Pixel format of a [`VIDEO_OUTPUT`][BufType::VIDEO_OUTPUT] or
/// [`VIDEO_CAPTURE`][BufType::VIDEO_CAPTURE] buffer.
pub struct PixFormat(raw::PixFormat);

pub struct PixFormatMplane(raw::PixFormatMplane);

pub struct Window(raw::Window);

pub struct PlanePixFormat(raw::PlanePixFormat);

/// Metadata format of a [`META_CAPTURE`][BufType::META_CAPTURE] or
/// [`META_OUTPUT`][BufType::META_OUTPUT] buffer.
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
    pub fn new(width: u32, height: u32, pixel_format: PixelFormat) -> Self {
        Self(raw::PixFormat {
            width,
            height,
            pixel_format,
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

    pub fn pixel_format(&self) -> PixelFormat {
        self.0.pixel_format
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
    pub fn new(format: PixelFormat) -> Self {
        Self(raw::MetaFormat {
            dataformat: format,
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
            .field("pixel_format", &self.0.pixel_format)
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
            .field("pixel_format", &{ self.0.pixel_format })
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

/// Iterator over a device's supported [`FormatDesc`]s.
pub struct FormatDescIter<'a> {
    device: &'a Device,
    buf_type: BufType,
    next_index: u32,
    finished: bool,
}

impl<'a> FormatDescIter<'a> {
    pub(crate) fn new(device: &'a Device, buf_type: BufType) -> Self {
        Self {
            device,
            buf_type,
            next_index: 0,
            finished: false,
        }
    }
}

impl Iterator for FormatDescIter<'_> {
    type Item = io::Result<FormatDesc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        unsafe {
            let mut desc = raw::FmtDesc {
                index: self.next_index,
                type_: self.buf_type,
                mbus_code: 0,
                ..mem::zeroed()
            };
            match raw::enum_fmt(self.device.fd(), &mut desc) {
                Ok(_) => {}
                Err(e) => {
                    self.finished = true;
                    match e {
                        Errno::EINVAL => return None,
                        e => return Some(Err(e.into())),
                    }
                }
            }

            self.next_index += 1;

            Some(Ok(FormatDesc(desc)))
        }
    }
}

pub struct FormatDesc(raw::FmtDesc);

impl FormatDesc {
    pub fn flags(&self) -> FormatFlags {
        self.0.flags
    }

    pub fn description(&self) -> &str {
        byte_array_to_str(&self.0.description)
    }

    pub fn pixel_format(&self) -> PixelFormat {
        self.0.pixel_format
    }
}

impl fmt::Debug for FormatDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Format")
            .field("index", &self.0.index)
            .field("type", &self.0.type_)
            .field("flags", &self.0.flags)
            .field("description", &self.description())
            .field("pixel_format", &self.0.pixel_format)
            .finish()
    }
}

pub enum FrameSizes {
    Discrete(Vec<DiscreteFrameSize>),
    Stepwise(StepwiseFrameSizes),
    Continuous(StepwiseFrameSizes),
}

impl FrameSizes {
    pub(crate) fn new(device: &Device, pixel_format: PixelFormat) -> io::Result<Self> {
        unsafe {
            let mut desc = raw::FrmSizeEnum {
                index: 0,
                pixel_format,
                ..mem::zeroed()
            };
            raw::enum_framesizes(device.fd(), &mut desc)?;

            match desc.type_ {
                FrmSizeType::DISCRETE => {
                    let mut sizes = vec![DiscreteFrameSize {
                        raw: desc.union.discrete,
                        index: 0,
                    }];
                    for index in 1.. {
                        let mut desc = raw::FrmSizeEnum {
                            index,
                            pixel_format,
                            ..mem::zeroed()
                        };
                        match raw::enum_framesizes(device.fd(), &mut desc) {
                            Ok(_) => {
                                assert_eq!(desc.type_, FrmSizeType::DISCRETE);
                                sizes.push(DiscreteFrameSize {
                                    raw: desc.union.discrete,
                                    index,
                                });
                            }
                            Err(Errno::EINVAL) => break,
                            Err(e) => return Err(e.into()),
                        }
                    }

                    Ok(FrameSizes::Discrete(sizes))
                }
                FrmSizeType::CONTINUOUS => Ok(FrameSizes::Continuous(StepwiseFrameSizes(
                    desc.union.stepwise,
                ))),
                FrmSizeType::STEPWISE => Ok(FrameSizes::Stepwise(StepwiseFrameSizes(
                    desc.union.stepwise,
                ))),
                _ => unreachable!("unknown frame size type {:?}", desc.type_),
            }
        }
    }

    pub fn min_width(&self) -> u32 {
        match self {
            FrameSizes::Discrete(sizes) => sizes.iter().map(|size| size.width()).min().unwrap(),
            FrameSizes::Stepwise(sizes) | FrameSizes::Continuous(sizes) => sizes.min_width(),
        }
    }

    pub fn min_height(&self) -> u32 {
        match self {
            FrameSizes::Discrete(sizes) => sizes.iter().map(|size| size.height()).min().unwrap(),
            FrameSizes::Stepwise(sizes) | FrameSizes::Continuous(sizes) => sizes.min_height(),
        }
    }

    pub fn max_width(&self) -> u32 {
        match self {
            FrameSizes::Discrete(sizes) => sizes.iter().map(|size| size.width()).max().unwrap(),
            FrameSizes::Stepwise(sizes) | FrameSizes::Continuous(sizes) => sizes.max_width(),
        }
    }

    pub fn max_height(&self) -> u32 {
        match self {
            FrameSizes::Discrete(sizes) => sizes.iter().map(|size| size.height()).max().unwrap(),
            FrameSizes::Stepwise(sizes) | FrameSizes::Continuous(sizes) => sizes.max_height(),
        }
    }
}

pub struct StepwiseFrameSizes(raw::FrmSizeStepwise);

pub struct DiscreteFrameSize {
    raw: raw::FrmSizeDiscrete,
    index: u32,
}

impl StepwiseFrameSizes {
    pub fn min_width(&self) -> u32 {
        self.0.min_width
    }

    pub fn min_height(&self) -> u32 {
        self.0.min_height
    }

    pub fn max_width(&self) -> u32 {
        self.0.max_width
    }

    pub fn max_height(&self) -> u32 {
        self.0.max_height
    }

    pub fn step_width(&self) -> u32 {
        self.0.step_width
    }

    pub fn step_height(&self) -> u32 {
        self.0.step_height
    }
}

impl DiscreteFrameSize {
    pub fn width(&self) -> u32 {
        self.raw.width
    }

    pub fn height(&self) -> u32 {
        self.raw.height
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub enum FrameIntervals {
    Discrete(Vec<DiscreteFrameInterval>),
    Stepwise(StepwiseFrameIntervals),
    Continuous(StepwiseFrameIntervals),
}

impl FrameIntervals {
    pub(crate) fn new(
        device: &Device,
        pixel_format: PixelFormat,
        width: u32,
        height: u32,
    ) -> io::Result<Self> {
        unsafe {
            let mut desc = raw::FrmIvalEnum {
                index: 0,
                pixel_format,
                width,
                height,
                ..mem::zeroed()
            };
            raw::enum_frameintervals(device.fd(), &mut desc)?;

            match desc.type_ {
                FrmIvalType::DISCRETE => {
                    let mut ivals = vec![DiscreteFrameInterval {
                        raw: desc.union.discrete,
                        index: 0,
                    }];
                    for index in 1.. {
                        let mut desc = raw::FrmIvalEnum {
                            index,
                            pixel_format,
                            width,
                            height,
                            ..mem::zeroed()
                        };
                        match raw::enum_frameintervals(device.fd(), &mut desc) {
                            Ok(_) => {
                                assert_eq!(desc.type_, FrmIvalType::DISCRETE);
                                ivals.push(DiscreteFrameInterval {
                                    raw: desc.union.discrete,
                                    index,
                                });
                            }
                            Err(Errno::EINVAL) => break,
                            Err(e) => return Err(e.into()),
                        }
                    }

                    Ok(FrameIntervals::Discrete(ivals))
                }
                FrmIvalType::CONTINUOUS => Ok(FrameIntervals::Continuous(StepwiseFrameIntervals(
                    desc.union.stepwise,
                ))),
                FrmIvalType::STEPWISE => Ok(FrameIntervals::Stepwise(StepwiseFrameIntervals(
                    desc.union.stepwise,
                ))),
                _ => {
                    // sadly, this can happen with a v4l2loopback device that no application is outputting data to
                    log::error!("driver bug: unknown frame interval type {:?}", desc.type_);
                    Ok(FrameIntervals::Continuous(StepwiseFrameIntervals(
                        raw::FrmIvalStepwise {
                            min: Fract::new(1, 600),
                            max: Fract::new(1, 1),
                            // FIXME: docs say that `step` is "1", but surely "1 second" is not the right step size?
                            // https://kernel.org/doc/html/v5.17-rc3/userspace-api/media/v4l/vidioc-enum-frameintervals.html
                            step: Fract::new(1, 1),
                        },
                    )))
                }
            }
        }
    }

    pub fn min(&self) -> Fract {
        match self {
            FrameIntervals::Discrete(list) => list.iter().map(|ival| ival.raw).min().unwrap(),
            FrameIntervals::Stepwise(ivals) | FrameIntervals::Continuous(ivals) => *ivals.min(),
        }
    }

    pub fn max(&self) -> Fract {
        match self {
            FrameIntervals::Discrete(list) => list.iter().map(|ival| ival.raw).max().unwrap(),
            FrameIntervals::Stepwise(ivals) | FrameIntervals::Continuous(ivals) => *ivals.max(),
        }
    }
}

impl fmt::Display for FrameIntervals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameIntervals::Discrete(list) => {
                for (i, ival) in list.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }

                    write!(f, "{}", ival.fract())?;
                }

                Ok(())
            }
            FrameIntervals::Stepwise(ival) => {
                write!(f, "{}-{} (step {})", ival.min(), ival.max(), ival.step())
            }
            FrameIntervals::Continuous(ival) => {
                write!(f, "{}-{}", ival.min(), ival.max())
            }
        }
    }
}

pub struct DiscreteFrameInterval {
    index: u32,
    raw: Fract,
}

impl DiscreteFrameInterval {
    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn fract(&self) -> &Fract {
        &self.raw
    }
}

pub struct StepwiseFrameIntervals(raw::FrmIvalStepwise);

impl StepwiseFrameIntervals {
    pub fn min(&self) -> &Fract {
        &self.0.min
    }

    pub fn max(&self) -> &Fract {
        &self.0.max
    }

    pub fn step(&self) -> &Fract {
        &self.0.step
    }
}
