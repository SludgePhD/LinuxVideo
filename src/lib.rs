//! Linux video device library.
//!
//! This library provides a (hopefully) convenient and high-level wrapper around the V4L2 ioctls,
//! and allows accessing video devices (capture cards, webcams, etc.) on Linux systems.

#[macro_use]
mod macros;
mod buf_type;
pub mod controls;
pub mod format;
mod pixel_format;
mod raw;
mod shared;
pub mod stream;
pub mod uvc;

use nix::errno::Errno;
use pixel_format::PixelFormat;
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    mem::{self, MaybeUninit},
    os::unix::prelude::*,
    path::{Path, PathBuf},
};

use controls::{ControlDesc, ControlIter, TextMenuIter};
use format::{Format, FormatDescIter, FrameIntervals, FrameSizes, MetaFormat, PixFormat};
use raw::controls::Cid;
use shared::{CaptureParamFlags, Memory, StreamParamCaps};
use stream::{ReadStream, WriteStream, DEFAULT_BUFFER_COUNT};

pub use buf_type::*;
pub use shared::{
    AnalogStd, CapabilityFlags, Fract, InputCapabilities, InputStatus, InputType,
    OutputCapabilities, OutputType,
};

/// Returns an iterator over all connected V4L2 devices.
pub fn list() -> io::Result<impl Iterator<Item = io::Result<Device>>> {
    Ok(fs::read_dir("/dev")?.flat_map(|file| {
        let file = match file {
            Ok(file) => file,
            Err(e) => return Some(Err(e.into())),
        };

        match file.file_type() {
            Ok(ty) => {
                if !ty.is_char_device() {
                    log::debug!("unexpected device file type {:?}", ty);
                    return None;
                }
            }
            Err(e) => return Some(Err(e.into())),
        }

        let name = file.file_name();
        let prefixes: &[&[u8]] = &[
            b"video",
            b"vbi",
            b"radio",
            b"swradio",
            b"v4l-touch",
            b"v4l-subdev",
        ];
        if prefixes.iter().any(|p| name.as_bytes().starts_with(p)) {
            Some(Device::open(&file.path()))
        } else {
            None
        }
    }))
}

/// A V4L2 device.
#[derive(Debug)]
pub struct Device {
    file: File,
    available_capabilities: CapabilityFlags,
}

impl Device {
    /// Opens a V4L2 device file from the given path.
    ///
    /// If the path does not refer to a V4L2 device node, an error will be returned.
    pub fn open<A: AsRef<Path>>(path: A) -> io::Result<Self> {
        Self::open_impl(path.as_ref())
    }

    fn open_impl(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut this = Self {
            file,
            available_capabilities: CapabilityFlags::empty(),
        };
        let caps = this.capabilities()?;
        this.available_capabilities = caps.device_capabilities();

        Ok(this)
    }

    fn fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    /// Returns the path to the V4L2 device.
    pub fn path(&self) -> io::Result<PathBuf> {
        Ok(fs::read_link(format!("/proc/self/fd/{}", self.fd()))?)
    }

    pub fn capabilities(&self) -> io::Result<Capabilities> {
        unsafe {
            let mut caps = MaybeUninit::uninit();
            let res = raw::querycap(self.fd(), caps.as_mut_ptr())?;
            assert_eq!(res, 0);
            Ok(Capabilities(caps.assume_init()))
        }
    }

    pub fn supported_buf_types(&self) -> BufTypes {
        BufTypes::from_capabilities(self.available_capabilities)
    }

    /// Enumerates the supported pixel formats of a stream.
    ///
    /// `buf_type` must be one of `VIDEO_CAPTURE`, `VIDEO_CAPTURE_MPLANE`, `VIDEO_OUTPUT`,
    /// `VIDEO_OUTPUT_MPLANE`, `VIDEO_OVERLAY`, `SDR_CAPTURE`, `SDR_OUTPUT`, `META_CAPTURE`, or
    /// `META_OUTPUT`.
    pub fn formats(&self, buf_type: BufType) -> FormatDescIter<'_> {
        FormatDescIter::new(self, buf_type)
    }

    /// Returns the supported frame sizes for a given pixel format.
    ///
    /// # Errors
    ///
    /// An `ENOTTY` error will be returned if `pixel_format` specifies a format that does not
    /// describe video data (for example, [`PixelFormat::UVC`] or other metadata formats).
    pub fn frame_sizes(&self, pixel_format: PixelFormat) -> io::Result<FrameSizes> {
        FrameSizes::new(self, pixel_format)
    }

    pub fn frame_intervals(
        &self,
        pixel_format: PixelFormat,
        width: u32,
        height: u32,
    ) -> io::Result<FrameIntervals> {
        FrameIntervals::new(self, pixel_format, width, height)
    }

    pub fn inputs(&self) -> InputIter<'_> {
        InputIter {
            device: self,
            next_index: 0,
            finished: false,
        }
    }

    pub fn outputs(&self) -> OutputIter<'_> {
        OutputIter {
            device: self,
            next_index: 0,
            finished: false,
        }
    }

    pub fn controls(&self) -> ControlIter<'_> {
        ControlIter::new(self)
    }

    /// Returns an iterator over the valid values of a menu control.
    pub fn enumerate_menu(&self, ctrl: &ControlDesc) -> TextMenuIter<'_> {
        TextMenuIter::new(self, ctrl)
    }

    pub fn read_control_raw(&self, cid: Cid) -> io::Result<i32> {
        let mut control = raw::controls::Control { id: cid, value: 0 };

        unsafe {
            raw::g_ctrl(self.fd(), &mut control)?;
        }

        Ok(control.value)
    }

    pub fn write_control_raw(&mut self, cid: Cid, value: i32) -> io::Result<()> {
        let mut control = raw::controls::Control { id: cid, value };
        unsafe {
            raw::s_ctrl(self.fd(), &mut control)?;
        }
        Ok(())
    }

    /// Reads the stream format in use by `buf_type`.
    ///
    /// The returned [`Format`] variant will match `buf_type`.
    ///
    /// If no format is set, this returns `EINVAL`.
    ///
    /// # Panics
    ///
    /// This will panic if `buf_type` corresponds to a buffer type that hasn't yet been implemented
    /// in [`Format`].
    pub fn format(&self, buf_type: BufType) -> io::Result<Format> {
        unsafe {
            let mut format = raw::Format {
                type_: buf_type,
                ..mem::zeroed()
            };
            raw::g_fmt(self.fd(), &mut format)?;
            let fmt = Format::from_raw(format)
                .unwrap_or_else(|| todo!("unsupported buffer type {:?}", buf_type));
            Ok(fmt)
        }
    }

    /// Negotiates a stream's format.
    ///
    /// The driver will adjust the values in `format` to the closest values it supports (the variant
    /// will not be changed). The modified `Format` is returned.
    fn set_format_raw(&mut self, format: Format) -> io::Result<Format> {
        unsafe {
            let mut raw_format: raw::Format = mem::zeroed();
            match format {
                Format::VideoCapture(f) => {
                    raw_format.type_ = BufType::VIDEO_CAPTURE;
                    raw_format.fmt.pix = f.to_raw();
                }
                Format::VideoOutput(f) => {
                    raw_format.type_ = BufType::VIDEO_OUTPUT;
                    raw_format.fmt.pix = f.to_raw();
                }
                Format::VideoCaptureMplane(f) => {
                    raw_format.type_ = BufType::VIDEO_CAPTURE_MPLANE;
                    raw_format.fmt.pix_mp = f.to_raw();
                }
                Format::VideoOutputMplane(f) => {
                    raw_format.type_ = BufType::VIDEO_OUTPUT_MPLANE;
                    raw_format.fmt.pix_mp = f.to_raw();
                }
                Format::VideoOverlay(f) => {
                    raw_format.type_ = BufType::VIDEO_OVERLAY;
                    raw_format.fmt.win = f.to_raw();
                }
                Format::MetaCapture(f) => {
                    raw_format.type_ = BufType::META_CAPTURE;
                    raw_format.fmt.meta = f.to_raw();
                }
                Format::MetaOutput(f) => {
                    raw_format.type_ = BufType::META_OUTPUT;
                    raw_format.fmt.meta = f.to_raw();
                }
            }
            raw::s_fmt(self.fd(), &mut raw_format)?;
            let fmt = Format::from_raw(raw_format).unwrap();
            Ok(fmt)
        }
    }

    /// Puts the device into video capture mode and negotiates a pixel format.
    ///
    /// # Format Negotiation
    ///
    /// Generally, the driver is allowed to change most properties of the [`PixFormat`], including
    /// the requested dimensions and the [`PixelFormat`], if the provided value is not supported.
    /// However, it is not required to do so and may instead return `EINVAL` if the parameters are
    /// not supported. One example where this happens is with `v4l2loopback`.
    pub fn video_capture(mut self, format: PixFormat) -> io::Result<VideoCaptureDevice> {
        let format = match self.set_format_raw(Format::VideoCapture(format))? {
            Format::VideoCapture(fmt) => fmt,
            _ => unreachable!(),
        };

        Ok(VideoCaptureDevice {
            file: self.file,
            format,
        })
    }

    /// Puts the device into video output mode and negotiates a pixel format.
    ///
    /// # Format Negotiation
    ///
    /// Generally, the driver is allowed to change most properties of the [`PixFormat`], including
    /// the requested dimensions and the [`PixelFormat`], if the provided value is not supported.
    /// However, it is not required to do so and may instead return `EINVAL` if the parameters are
    /// not supported. One example where this happens is with `v4l2loopback`.
    pub fn video_output(mut self, format: PixFormat) -> io::Result<VideoOutputDevice> {
        let format = match self.set_format_raw(Format::VideoOutput(format))? {
            Format::VideoOutput(fmt) => fmt,
            _ => unreachable!(),
        };

        Ok(VideoOutputDevice {
            file: self.file,
            format,
        })
    }

    /// Puts the device into metadata capture mode and negotiates a data format.
    pub fn meta_capture(mut self, format: MetaFormat) -> io::Result<MetaCaptureDevice> {
        let format = match self.set_format_raw(Format::MetaCapture(format))? {
            Format::MetaCapture(fmt) => fmt,
            _ => unreachable!(),
        };

        Ok(MetaCaptureDevice {
            file: self.file,
            format,
        })
    }
}

/// A video device configured for video capture.
pub struct VideoCaptureDevice {
    file: File,
    format: PixFormat,
}

impl VideoCaptureDevice {
    /// Returns the pixel format the driver chose for capturing.
    ///
    /// This may (and usually will) differ from the format passed to [`Device::video_capture`].
    pub fn format(&self) -> &PixFormat {
        &self.format
    }

    /// Requests a change to the frame interval.
    ///
    /// Returns the actual frame interval chosen by the driver.
    ///
    /// Supported frame intervals depend on the pixel format and video resolution and can be
    /// enumerated with [`Device::frame_intervals`].
    pub fn set_frame_interval(&self, interval: Fract) -> io::Result<Fract> {
        unsafe {
            let mut parm = raw::StreamParm {
                type_: BufType::VIDEO_CAPTURE,
                union: raw::StreamParmUnion {
                    capture: raw::CaptureParm {
                        timeperframe: interval,
                        capability: StreamParamCaps::TIMEPERFRAME,
                        capturemode: CaptureParamFlags::empty(),
                        extendedmode: 0,
                        readbuffers: 0,
                        reserved: [0; 4],
                    },
                },
            };
            raw::s_parm(self.file.as_raw_fd(), &mut parm)?;
            Ok(parm.union.capture.timeperframe)
        }
    }

    /// Initializes streaming I/O mode with the given number of buffers.
    pub fn into_stream(self) -> io::Result<ReadStream> {
        Ok(ReadStream::new(
            self.file,
            BufType::VIDEO_CAPTURE,
            Memory::MMAP,
            DEFAULT_BUFFER_COUNT,
        )?)
    }
}

/// Performs a direct `read()` from the video device.
///
/// This will only succeed if the device advertises the `READWRITE` capability, otherwise an
/// error will be returned and you have to use the streaming API instead.
impl Read for VideoCaptureDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

/// A video device configured for video output.
pub struct VideoOutputDevice {
    file: File,
    format: PixFormat,
}

impl VideoOutputDevice {
    /// Returns the video format chosen by the driver.
    pub fn format(&self) -> &PixFormat {
        &self.format
    }

    /// Initializes streaming I/O mode.
    pub fn into_stream(self) -> io::Result<WriteStream> {
        Ok(WriteStream::new(
            self.file,
            BufType::VIDEO_CAPTURE,
            Memory::MMAP,
            DEFAULT_BUFFER_COUNT,
        )?)
    }
}

/// Performs a direct `write()` on the video device file, writing a video frame to it.
///
/// This will only succeed if the device advertises the `READWRITE` capability, otherwise an
/// error will be returned and you have to use the streaming API instead.
///
/// Note that some applications, like guvcview, do not support the read/write methods, so using this
/// on a v4l2loopback device will not work with such applications.
impl Write for VideoOutputDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// A device configured for metadata capture.
pub struct MetaCaptureDevice {
    file: File,
    format: MetaFormat,
}

impl MetaCaptureDevice {
    /// Returns the metadata format the driver chose.
    pub fn format(&self) -> &MetaFormat {
        &self.format
    }

    /// Initializes streaming I/O mode.
    pub fn into_stream(self) -> io::Result<ReadStream> {
        Ok(ReadStream::new(
            self.file,
            BufType::META_CAPTURE,
            Memory::MMAP,
            DEFAULT_BUFFER_COUNT,
        )?)
    }
}

/// Performs a direct `read()` from the video device.
///
/// This will only succeed if the device advertises the `READWRITE` capability, otherwise an
/// error will be returned and you have to use the streaming API instead.
impl Read for MetaCaptureDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

/// Stores generic device information.
///
/// Returned by [`Device::capabilities`].
pub struct Capabilities(raw::Capabilities);

impl Capabilities {
    /// Returns the identifier of the V4L2 driver that provides this device.
    ///
    /// Examples:
    /// - `uvcvideo`
    /// - `v4l2 loopback`
    pub fn driver(&self) -> &str {
        byte_array_to_str(&self.0.driver)
    }

    /// Returns the card or device name.
    ///
    /// For `v4l2loopback` devices, the reported card name can be configured by passing the
    /// `card_label` parameter when loading the module (or via `modprobe.d`).
    pub fn card(&self) -> &str {
        byte_array_to_str(&self.0.card)
    }

    /// Returns a description of where on the system the device is attached.
    ///
    /// Examples:
    /// - `usb-0000:0a:00.3-2.1`
    /// - `platform:v4l2loopback-002`
    pub fn bus_info(&self) -> &str {
        byte_array_to_str(&self.0.bus_info)
    }

    /// Returns all capabilities the underlying hardware device exposes.
    ///
    /// Some capabilities might be inaccessible through the opened device node and require opening a
    /// different one.
    pub fn all_capabilities(&self) -> CapabilityFlags {
        self.0.capabilities
    }

    /// Returns the capabilities available through the currently opened device node.
    pub fn device_capabilities(&self) -> CapabilityFlags {
        if self.0.capabilities.contains(CapabilityFlags::DEVICE_CAPS) {
            self.0.device_caps
        } else {
            self.0.capabilities
        }
    }
}

impl fmt::Debug for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Capabilities")
            .field("driver", &self.driver())
            .field("card", &self.card())
            .field("bus_info", &self.bus_info())
            .field("capabilities", &self.0.capabilities)
            .field("device_caps", &self.0.device_caps)
            .finish()
    }
}

/// Iterator over the [`Output`]s of a [`Device`].
pub struct OutputIter<'a> {
    device: &'a Device,
    next_index: u32,
    finished: bool,
}

impl Iterator for OutputIter<'_> {
    type Item = io::Result<Output>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        unsafe {
            let mut raw = raw::Output {
                index: self.next_index,
                ..mem::zeroed()
            };
            match raw::enumoutput(self.device.fd(), &mut raw) {
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

            Some(Ok(Output(raw)))
        }
    }
}

/// Iterator over the [`Input`]s of a [`Device`].
pub struct InputIter<'a> {
    device: &'a Device,
    next_index: u32,
    finished: bool,
}

impl Iterator for InputIter<'_> {
    type Item = io::Result<Input>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        unsafe {
            let mut raw = raw::Input {
                index: self.next_index,
                ..mem::zeroed()
            };
            match raw::enuminput(self.device.fd(), &mut raw) {
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

            Some(Ok(Input(raw)))
        }
    }
}

/// Information about a device output.
pub struct Output(raw::Output);

impl Output {
    /// Returns the output's name.
    ///
    /// Examples:
    /// - `loopback in`
    pub fn name(&self) -> &str {
        byte_array_to_str(&self.0.name)
    }

    /// Returns what kind of device this output is.
    #[inline]
    pub fn output_type(&self) -> OutputType {
        self.0.type_
    }

    /// Returns the set of selectable audio sources when this output is active.
    ///
    /// This may return 0 even if the device supports audio inputs to indicate that the application
    /// cannot choose an audio input.
    #[inline]
    pub fn audioset(&self) -> u32 {
        self.0.audioset
    }

    /// Returns the modulator index if this input is of type [`OutputType::MODULATOR`].
    ///
    /// For non-modulator outputs, this value should be ignored.
    #[inline]
    pub fn modulator(&self) -> u32 {
        self.0.modulator
    }

    /// Returns the set of supported analog video standards.
    #[inline]
    pub fn std(&self) -> AnalogStd {
        self.0.std
    }

    /// Returns the capability flags of this output.
    #[inline]
    pub fn capabilities(&self) -> OutputCapabilities {
        self.0.capabilities
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("index", &self.0.index)
            .field("name", &self.name())
            .field("output_type", &self.output_type())
            .field("audioset", &self.0.audioset)
            .field("modulator", &self.0.modulator)
            .field("std", &self.0.std)
            .field("capabilities", &self.0.capabilities)
            .finish()
    }
}

/// Information about a device input.
pub struct Input(raw::Input);

impl Input {
    /// Returns the name of the input.
    ///
    /// Examples:
    /// - `Camera 1`
    /// - `loopback`
    pub fn name(&self) -> &str {
        byte_array_to_str(&self.0.name)
    }

    /// Returns what kind of device this input is.
    #[inline]
    pub fn input_type(&self) -> InputType {
        self.0.type_
    }

    /// Returns the set of selectable audio sources when this input is active.
    ///
    /// This may return 0 even if the device supports audio inputs to indicate that the application
    /// cannot choose an audio input.
    #[inline]
    pub fn audioset(&self) -> u32 {
        self.0.audioset
    }

    /// Returns the tuner index if this input is of type [`InputType::TUNER`].
    ///
    /// For non-tuner inputs, this value should be ignored.
    #[inline]
    pub fn tuner(&self) -> u32 {
        self.0.tuner
    }

    /// Returns the set of supported analog video standards for this input.
    #[inline]
    pub fn std(&self) -> AnalogStd {
        self.0.std
    }

    /// Returns the current status of the input.
    ///
    /// Note that the input needs to be selected as the active input for most fields in this value
    /// to be valid.
    #[inline]
    pub fn status(&self) -> InputStatus {
        self.0.status
    }

    /// Returns the capability flags of this input.
    #[inline]
    pub fn capabilities(&self) -> InputCapabilities {
        self.0.capabilities
    }
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Input")
            .field("index", &self.0.index)
            .field("name", &self.name())
            .field("input_type", &self.input_type())
            .field("audioset", &self.0.audioset)
            .field("tuner", &self.0.tuner)
            .field("std", &self.0.std)
            .field("status", &self.0.status)
            .field("capabilities", &self.0.capabilities)
            .finish()
    }
}

/// Turns a zero-padded byte array containing UTF-8 or ASCII data into a `&str`.
fn byte_array_to_str(bytes: &[u8]) -> &str {
    let len = bytes
        .iter()
        .position(|b| *b == 0)
        .expect("missing NUL terminator");
    std::str::from_utf8(&bytes[..len]).unwrap()
}
