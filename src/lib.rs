//! **Li**nux **Vid**eo device library.

#![forbid(unaligned_references)] // can't believe this isn't default

#[macro_use]
mod macros;
mod buf_type;
pub mod format;
mod pixelformat;
mod raw;
mod shared;
pub mod stream;
pub mod uvc;

use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    mem::{self, MaybeUninit},
    os::unix::prelude::*,
    path::{Path, PathBuf},
};

use format::{Format, MetaFormat, PixFormat};
use nix::errno::Errno;
use raw::controls::Cid;

pub use buf_type::*;
pub use pixelformat::Pixelformat;
pub use shared::*;
use stream::{ReadStream, WriteStream};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn list() -> Result<impl Iterator<Item = Result<Device>>> {
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
    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut this = Self {
            file,
            available_capabilities: CapabilityFlags::empty(),
        };
        let caps = this.capabilities()?;
        this.available_capabilities = caps.device_capabilities();

        Ok(this)
    }

    pub fn path(&self) -> Result<PathBuf> {
        Ok(fs::read_link(format!(
            "/proc/self/fd/{}",
            self.file.as_raw_fd()
        ))?)
    }

    pub fn capabilities(&self) -> Result<Capabilities> {
        unsafe {
            let mut caps = MaybeUninit::uninit();
            let res = raw::querycap(self.file.as_raw_fd(), caps.as_mut_ptr())?;
            assert_eq!(res, 0);
            Ok(Capabilities(caps.assume_init()))
        }
    }

    pub fn supported_buf_types(&self) -> BufTypes {
        BufTypes::from_capabilities(self.available_capabilities)
    }

    /// Enumerates the supported formats of a stream.
    ///
    /// `buf_type` must be one of `VIDEO_CAPTURE`, `VIDEO_CAPTURE_MPLANE`, `VIDEO_OUTPUT`,
    /// `VIDEO_OUTPUT_MPLANE`, `VIDEO_OVERLAY`, `SDR_CAPTURE`, `SDR_OUTPUT`, `META_CAPTURE`, or
    /// `META_OUTPUT`.
    pub fn formats(&self, buf_type: BufType) -> FormatDescIter<'_> {
        FormatDescIter {
            device: self,
            buf_type,
            next_index: 0,
            finished: false,
        }
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
        ControlIter {
            device: self,
            next_cid: Cid::BASE,
            finished: false,
            use_ctrl_flag_next_ctrl: true,
        }
    }

    /// Returns an iterator over the valid values of a menu control.
    pub fn enumerate_menu(&self, ctrl: &ControlDesc) -> TextMenuIter<'_> {
        assert_eq!(
            ctrl.control_type(),
            CtrlType::MENU,
            "`enumerate_menu` requires a menu control"
        );

        TextMenuIter {
            device: self,
            cid: ctrl.id(),
            next_index: ctrl.minimum() as _,
            max_index: ctrl.maximum() as _,
        }
    }

    pub fn read_control(&self, cid: Cid) -> Result<i32> {
        let mut control = raw::controls::Control { id: cid, value: 0 };

        unsafe {
            raw::g_ctrl(self.file.as_raw_fd(), &mut control)?;
        }

        Ok(control.value)
    }

    pub fn write_control(&mut self, cid: Cid, value: i32) -> Result<()> {
        let mut control = raw::controls::Control { id: cid, value };
        unsafe {
            raw::s_ctrl(self.file.as_raw_fd(), &mut control)?;
        }
        Ok(())
    }

    /// Reads the stream format in use by `buf_type`.
    ///
    /// The returned `Format` variant will match `buf_type`.
    ///
    /// If no format is set, this returns `EINVAL`.
    pub fn format(&self, buf_type: BufType) -> Result<Format> {
        unsafe {
            let mut format = raw::Format {
                type_: buf_type,
                ..mem::zeroed()
            };
            raw::g_fmt(self.file.as_raw_fd(), &mut format)?;
            let fmt = Format::from_raw(format)
                .ok_or_else(|| format!("unsupported buffer type {:?}", buf_type))?;
            Ok(fmt)
        }
    }

    /// Negotiates a stream's format.
    ///
    /// The driver will adjust the values in `format` to the closest values it supports (the variant
    /// will not be changed). The modified `Format` is returned.
    fn set_format_raw(&mut self, format: Format) -> Result<Format> {
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
            raw::s_fmt(self.file.as_raw_fd(), &mut raw_format)?;
            let fmt = Format::from_raw(raw_format).unwrap();
            Ok(fmt)
        }
    }

    /// Puts the device into video capture mode and negotiates a pixel format.
    ///
    /// # Format Negotiation
    ///
    /// Generally, the driver is allowed to change most properties of the [`PixFormat`], including
    /// the requested dimensions and the [`Pixelformat`], if the provided value is not supported.
    /// However, it is not required to do so and may instead return `EINVAL` if the parameters are
    /// not supported. One example where this happens is with `v4l2loopback`.
    pub fn video_capture(mut self, format: PixFormat) -> Result<VideoCaptureDevice> {
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
    /// the requested dimensions and the [`Pixelformat`], if the provided value is not supported.
    /// However, it is not required to do so and may instead return `EINVAL` if the parameters are
    /// not supported. One example where this happens is with `v4l2loopback`.
    pub fn video_output(mut self, format: PixFormat) -> Result<VideoOutputDevice> {
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
    pub fn meta_capture(mut self, format: MetaFormat) -> Result<MetaCaptureDevice> {
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

/// A V4L2 device configured for video capture ([`BufType::VIDEO_CAPTURE`]).
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

    /// Initializes streaming I/O mode with the given number of buffers.
    ///
    /// Note that some drivers may fail to allocate even low buffer counts. For example v4l2loopback
    /// seems to be limited to 2 buffers.
    pub fn into_stream(self, buffer_count: u32) -> Result<ReadStream> {
        Ok(ReadStream::new(
            self.file,
            BufType::VIDEO_CAPTURE,
            Memory::MMAP,
            buffer_count,
        )?)
    }
}

/// Performs a direct `read()` from the video device.
///
/// This will only succeed if the device advertises the `READWRITE` capability, otherwise an
/// error will be returned and you have to use the streaming API instead.
impl Read for VideoCaptureDevice {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

pub struct VideoOutputDevice {
    file: File,
    format: PixFormat,
}

impl VideoOutputDevice {
    pub fn format(&self) -> &PixFormat {
        &self.format
    }

    /// Initializes streaming I/O mode with the given number of buffers.
    ///
    /// Note that some drivers may fail to allocate even low buffer counts. For example v4l2loopback
    /// seems to be limited to 2 buffers.
    pub fn into_stream(self, buffer_count: u32) -> Result<WriteStream> {
        Ok(WriteStream::new(
            self.file,
            BufType::VIDEO_CAPTURE,
            Memory::MMAP,
            buffer_count,
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
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

pub struct MetaCaptureDevice {
    file: File,
    format: MetaFormat,
}

impl MetaCaptureDevice {
    /// Returns the metadata format the driver chose.
    pub fn format(&self) -> &MetaFormat {
        &self.format
    }

    /// Initializes streaming I/O mode with the given number of buffers.
    pub fn into_stream(self, buffer_count: u32) -> Result<ReadStream> {
        Ok(ReadStream::new(
            self.file,
            BufType::META_CAPTURE,
            Memory::MMAP,
            buffer_count,
        )?)
    }

    // FIXME: footgun: `into_stream` not starting the stream leaves user (me) dumbfounded
}

/// Performs a direct `read()` from the video device.
///
/// This will only succeed if the device advertises the `READWRITE` capability, otherwise an
/// error will be returned and you have to use the streaming API instead.
impl Read for MetaCaptureDevice {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

pub struct Capabilities(raw::Capabilities);

impl Capabilities {
    /// Returns the identifier of the V4L2 driver that provides this device.
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

/// Iterator over a device's supported [`FormatDesc`]s.
pub struct FormatDescIter<'a> {
    device: &'a Device,
    buf_type: BufType,
    next_index: u32,
    finished: bool,
}

impl Iterator for FormatDescIter<'_> {
    type Item = Result<FormatDesc>;

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
            match raw::enum_fmt(self.device.file.as_raw_fd(), &mut desc) {
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
    pub fn flags(&self) -> FmtFlags {
        self.0.flags
    }

    pub fn description(&self) -> &str {
        byte_array_to_str(&self.0.description)
    }

    pub fn pixelformat(&self) -> Pixelformat {
        self.0.pixelformat
    }
}

impl fmt::Debug for FormatDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Format")
            .field("index", &self.0.index)
            .field("type", &self.0.type_)
            .field("flags", &self.0.flags)
            .field("description", &self.description())
            .field("pixelformat", &self.0.pixelformat)
            .finish()
    }
}

pub struct OutputIter<'a> {
    device: &'a Device,
    next_index: u32,
    finished: bool,
}

impl Iterator for OutputIter<'_> {
    type Item = Result<Output>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        unsafe {
            let mut raw = raw::Output {
                index: self.next_index,
                ..mem::zeroed()
            };
            match raw::enumoutput(self.device.file.as_raw_fd(), &mut raw) {
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

pub struct InputIter<'a> {
    device: &'a Device,
    next_index: u32,
    finished: bool,
}

impl Iterator for InputIter<'_> {
    type Item = Result<Input>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        unsafe {
            let mut raw = raw::Input {
                index: self.next_index,
                ..mem::zeroed()
            };
            match raw::enuminput(self.device.file.as_raw_fd(), &mut raw) {
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

pub struct ControlIter<'a> {
    device: &'a Device,
    next_cid: Cid,
    finished: bool,
    use_ctrl_flag_next_ctrl: bool,
}

impl Iterator for ControlIter<'_> {
    type Item = Result<ControlDesc>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.finished {
                return None;
            }

            if self.next_cid.0 >= Cid::LASTP1.0 && !self.use_ctrl_flag_next_ctrl {
                return None;
            }

            unsafe {
                let mut id = self.next_cid.0;
                if self.use_ctrl_flag_next_ctrl {
                    id |= CtrlFlags::NEXT_CTRL.bits();
                }
                let mut raw = raw::QueryCtrl {
                    id,
                    ..mem::zeroed()
                };
                match raw::queryctrl(self.device.file.as_raw_fd(), &mut raw) {
                    Ok(_) => {
                        if self.use_ctrl_flag_next_ctrl {
                            self.next_cid.0 = raw.id;
                        } else {
                            self.next_cid.0 += 1;
                        }
                    }
                    Err(e) => {
                        match e {
                            Errno::EINVAL => {
                                self.use_ctrl_flag_next_ctrl = false;
                                self.next_cid.0 += 1;
                                continue; // continue, because there might be gaps
                            }
                            e => {
                                self.finished = true;
                                return Some(Err(e.into()));
                            }
                        }
                    }
                }

                if raw.flags.contains(CtrlFlags::DISABLED) {
                    continue;
                }

                return Some(Ok(ControlDesc(raw)));
            }
        }
    }
}

pub struct ControlDesc(raw::QueryCtrl);

impl ControlDesc {
    pub fn id(&self) -> Cid {
        Cid(self.0.id)
    }

    pub fn name(&self) -> &str {
        byte_array_to_str(&self.0.name)
    }

    pub fn control_type(&self) -> CtrlType {
        self.0.type_
    }

    pub fn minimum(&self) -> i32 {
        self.0.minimum
    }

    pub fn maximum(&self) -> i32 {
        self.0.maximum
    }

    pub fn step(&self) -> i32 {
        self.0.step
    }

    pub fn default_value(&self) -> i32 {
        self.0.default_value
    }

    pub fn flags(&self) -> CtrlFlags {
        self.0.flags
    }
}

impl fmt::Debug for ControlDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlDesc")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("control_type", &self.control_type())
            .field("minimum", &self.minimum())
            .field("maximum", &self.maximum())
            .field("step", &self.step())
            .field("default_value", &self.default_value())
            .field("flags", &self.flags())
            .finish()
    }
}

/// An iterator over a menu control's valid choices.
///
/// Note that the returned [`TextMenuItem`]s might not have contiguous indices, since this iterator
/// automatically skips invalid indices.
pub struct TextMenuIter<'a> {
    device: &'a Device,
    cid: Cid,
    next_index: u32,
    /// Highest allowed index.
    max_index: u32,
}

impl Iterator for TextMenuIter<'_> {
    type Item = Result<TextMenuItem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index > self.max_index {
            None
        } else {
            loop {
                unsafe {
                    let mut raw = raw::QueryMenu {
                        id: self.cid.0,
                        index: self.next_index,
                        ..mem::zeroed()
                    };

                    self.next_index += 1;
                    match raw::querymenu(self.device.file.as_raw_fd(), &mut raw) {
                        Ok(_) => return Some(Ok(TextMenuItem { raw })),
                        Err(Errno::EINVAL) => continue,
                        Err(other) => return Some(Err(other.into())),
                    }
                }
            }
        }
    }
}

pub struct TextMenuItem {
    raw: raw::QueryMenu,
}

impl TextMenuItem {
    /// The item's index. Setting the menu control to this value will choose this item.
    pub fn index(&self) -> u32 {
        self.raw.index
    }

    /// The human-readable name of this menu entry.
    pub fn name(&self) -> &str {
        byte_array_to_str(unsafe { &self.raw.name_or_value.name })
    }
}

pub struct Output(raw::Output);

impl Output {
    pub fn name(&self) -> &str {
        byte_array_to_str(&self.0.name)
    }

    pub fn output_type(&self) -> OutputType {
        self.0.type_
    }

    pub fn audioset(&self) -> u32 {
        self.0.audioset
    }

    pub fn modulator(&self) -> u32 {
        self.0.modulator
    }

    pub fn std(&self) -> AnalogStd {
        self.0.std
    }

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

pub struct Input(raw::Input);

impl Input {
    pub fn name(&self) -> &str {
        byte_array_to_str(&self.0.name)
    }

    pub fn input_type(&self) -> InputType {
        self.0.type_
    }

    pub fn audioset(&self) -> u32 {
        self.0.audioset
    }

    pub fn tuner(&self) -> u32 {
        self.0.tuner
    }

    pub fn std(&self) -> AnalogStd {
        self.0.std
    }

    pub fn status(&self) -> InputStatus {
        self.0.status
    }

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
