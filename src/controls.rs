//! Device control enumeration and access.

use std::{fmt, io, mem};

use nix::errno::Errno;

use crate::shared::CONTROL_FLAGS_NEXT_CTRL;
use crate::{byte_array_to_str, raw, Device};

pub use crate::raw::controls::Cid;
pub use crate::shared::{ControlFlags, CtrlType};

/// Iterator over the control descriptors of a device.
pub struct ControlIter<'a> {
    device: &'a Device,
    next_cid: Cid,
    finished: bool,
    use_ctrl_flag_next_ctrl: bool,
}

impl<'a> ControlIter<'a> {
    pub(crate) fn new(device: &'a Device) -> Self {
        Self {
            device,
            next_cid: Cid::BASE,
            finished: false,
            use_ctrl_flag_next_ctrl: true,
        }
    }
}

impl Iterator for ControlIter<'_> {
    type Item = io::Result<ControlDesc>;

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
                    id |= CONTROL_FLAGS_NEXT_CTRL;
                }
                let mut raw = raw::QueryCtrl {
                    id,
                    ..mem::zeroed()
                };
                match raw::queryctrl(self.device.fd(), &mut raw) {
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

                if raw.flags.contains(ControlFlags::DISABLED) {
                    continue;
                }

                return Some(Ok(ControlDesc(raw)));
            }
        }
    }
}

/// Describes a device control.
pub struct ControlDesc(raw::QueryCtrl);

impl ControlDesc {
    /// The control's identifier.
    #[inline]
    pub fn id(&self) -> Cid {
        Cid(self.0.id)
    }

    /// The user-facing name of this control.
    pub fn name(&self) -> &str {
        byte_array_to_str(&self.0.name)
    }

    /// Returns the type of value this control expects.
    #[inline]
    pub fn control_type(&self) -> CtrlType {
        self.0.type_
    }

    #[inline]
    pub fn minimum(&self) -> i32 {
        self.0.minimum
    }

    #[inline]
    pub fn maximum(&self) -> i32 {
        self.0.maximum
    }

    #[inline]
    pub fn step(&self) -> i32 {
        self.0.step
    }

    #[inline]
    pub fn default_value(&self) -> i32 {
        self.0.default_value
    }

    #[inline]
    pub fn flags(&self) -> ControlFlags {
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

impl<'a> TextMenuIter<'a> {
    pub(crate) fn new(device: &'a Device, ctrl: &ControlDesc) -> Self {
        assert_eq!(ctrl.control_type(), CtrlType::MENU, "menu control required");

        Self {
            device,
            cid: ctrl.id(),
            next_index: ctrl.minimum() as _,
            max_index: ctrl.maximum() as _,
        }
    }
}

impl Iterator for TextMenuIter<'_> {
    type Item = io::Result<TextMenuItem>;

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
                    match raw::querymenu(self.device.fd(), &mut raw) {
                        Ok(_) => return Some(Ok(TextMenuItem { raw })),
                        Err(Errno::EINVAL) => continue,
                        Err(other) => return Some(Err(other.into())),
                    }
                }
            }
        }
    }
}

/// A possible choice for a menu control.
pub struct TextMenuItem {
    raw: raw::QueryMenu,
}

impl TextMenuItem {
    /// The item's index. Setting the menu control to this value will choose this item.
    #[inline]
    pub fn index(&self) -> u32 {
        self.raw.index
    }

    /// The human-readable name of this menu entry.
    pub fn name(&self) -> &str {
        byte_array_to_str(unsafe { &self.raw.name_or_value.name })
    }
}
