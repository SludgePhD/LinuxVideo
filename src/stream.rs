//! Streaming I/O.

use core::slice;
use std::ffi::c_void;
use std::fs::File;
use std::ops::Deref;
use std::os::unix::prelude::{AsRawFd, RawFd};
use std::{mem, ptr};

use nix::sys::mman::{mmap, munmap, MapFlags, ProtFlags};

use crate::buf_type::BufType;
use crate::shared::{BufFlag, Memory};
use crate::{raw, Result};

enum AllocType {
    /// The buffer was `mmap`ped into our address space, use `munmap` to free it.
    Mmap,
}

struct Buffer {
    /// Pointer in our address space where this buffer is mapped or allocated.
    ptr: *mut c_void,
    /// Size of the buffer in bytes.
    length: u32,
}

/// Owns all buffers allocated or mapped for a device stream.
struct Buffers {
    ty: AllocType,
    /// The buffer index equals its index in this vector.
    buffers: Vec<Buffer>,
}

impl Drop for Buffers {
    fn drop(&mut self) {
        for buffer in &self.buffers {
            match self.ty {
                AllocType::Mmap => unsafe {
                    munmap(buffer.ptr, buffer.length as usize).ok();
                },
            }
        }
    }
}

/// A stream that reads data from a V4L2 device.
pub struct ReadStream {
    file: File,
    buffers: Buffers,
    buf_type: BufType,
    mem_type: Memory,
}

impl ReadStream {
    pub(crate) fn new(
        file: File,
        buf_type: BufType,
        mem_type: Memory,
        buffer_count: u32,
    ) -> Result<Self> {
        let fd = file.as_raw_fd();
        assert_eq!(
            mem_type,
            Memory::MMAP,
            "only `mmap` memory type is current supported"
        );

        let mut req_bufs: raw::RequestBuffers = unsafe { mem::zeroed() };
        req_bufs.count = buffer_count;
        req_bufs.type_ = buf_type;
        req_bufs.memory = mem_type;

        unsafe {
            raw::reqbufs(fd, &mut req_bufs)?;
        }

        log::debug!("{:?}", req_bufs);

        if req_bufs.count < buffer_count {
            return Err(format!(
                "failed to allocate {} buffers (driver only allocated {})",
                buffer_count, req_bufs.count
            )
            .into());
        }

        // Query the buffer locations and map them into our process.
        let mut buffers = Vec::with_capacity(buffer_count as usize);
        for i in 0..buffer_count {
            let mut buf: raw::Buffer = unsafe { mem::zeroed() };
            buf.type_ = buf_type;
            buf.memory = mem_type;
            buf.index = i;

            unsafe {
                raw::querybuf(fd, &mut buf)?;
            }

            // NB: buffer sizes are usually `PixFormat::size_image(_)` rounded up to whole pages
            let ptr = unsafe {
                mmap(
                    ptr::null_mut(),
                    buf.length as usize,
                    ProtFlags::PROT_READ,
                    MapFlags::MAP_SHARED,
                    fd,
                    buf.m.offset as _,
                )?
            };

            assert_eq!(buf.index, i);
            assert_eq!(buf.index as usize, buffers.len());

            buffers.push(Buffer {
                ptr,
                length: buf.length,
            });
        }

        let mut this = Self {
            file,
            buffers: Buffers {
                ty: AllocType::Mmap,
                buffers,
            },
            buf_type,
            mem_type,
        };
        this.enqueue_all()?;

        Ok(this)
    }

    fn enqueue(&mut self, index: u32) -> Result<()> {
        let mut buf: raw::Buffer = unsafe { mem::zeroed() };
        buf.type_ = self.buf_type;
        buf.memory = self.mem_type;
        buf.index = index;

        unsafe {
            raw::qbuf(self.file.as_raw_fd(), &mut buf)?;
        }

        Ok(())
    }

    fn enqueue_all(&mut self) -> Result<()> {
        for i in 0..self.buffers.buffers.len() {
            self.enqueue(i as u32)?;
        }
        Ok(())
    }

    /// Starts streaming.
    ///
    /// This function can potentially block for a noticeable amount of time.
    pub fn stream_on(&mut self) -> Result<()> {
        unsafe {
            raw::streamon(self.file.as_raw_fd(), &self.buf_type)?;
        }

        Ok(())
    }

    // XXX to publicly expose this, we have to handle the fact that it dequeues all buffers
    fn stream_off(&mut self) -> Result<()> {
        unsafe {
            raw::streamoff(self.file.as_raw_fd(), &self.buf_type)?;
        }

        Ok(())
    }

    /// Dequeues a buffer, passes it to `cb`, then enqueues it again.
    ///
    /// If `cb` returns an error, this function will still try to enqueue the buffer again. If that
    /// fails, the error that occurred during enqueuing will be returned, if it succeeds, the error
    /// returned by `cb` will be returned.
    pub fn dequeue(&mut self, cb: impl FnOnce(ReadBufferView<'_>) -> Result<()>) -> Result<()> {
        let mut buf: raw::Buffer = unsafe { mem::zeroed() };
        buf.type_ = self.buf_type;
        buf.memory = self.mem_type;

        unsafe {
            raw::dqbuf(self.file.as_raw_fd(), &mut buf)?;
        }

        let buffer = &self.buffers.buffers[buf.index as usize];
        let data =
            unsafe { slice::from_raw_parts(buffer.ptr as *const u8, buffer.length as usize) };
        let view = ReadBufferView {
            flags: buf.flags,
            data,
        };

        let res = cb(view);
        // XXX not sure if we should short-circuit here

        self.enqueue(buf.index)?;

        res
    }
}

impl Drop for ReadStream {
    fn drop(&mut self) {
        // Turn off the stream to dequeue all buffers.
        self.stream_off().ok();
    }
}

impl AsRawFd for ReadStream {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

/// Immutable view into a dequeued (filled) read buffer.
///
/// Dereferences to a byte slice.
pub struct ReadBufferView<'a> {
    flags: BufFlag,
    data: &'a [u8],
}

impl ReadBufferView<'_> {
    /// Returns whether the error flag for this buffer is set.
    ///
    /// If this returns `true`, the application should expect data corruption in the buffer data.
    pub fn is_error(&self) -> bool {
        self.flags.contains(BufFlag::ERROR)
    }
}

impl Deref for ReadBufferView<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
