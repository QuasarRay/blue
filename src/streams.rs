//! Source counterparts for memory streams and their inherited contracts.
#![allow(non_snake_case, non_camel_case_types)]

use std::io;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

/// Carbon core boundary. Native ownership replaces reference-count machinery.
pub trait IRoot: Send + Sync {}

/// Operations inherited from Carbon core, with slices replacing raw buffers.
/// The dependency's complete interface inventory is still pending.
pub trait ICcpStream: Send + Sync {
    fn Read(&mut self, dest: &mut [u8], count: isize) -> io::Result<usize>;
    fn Write(&mut self, source: &[u8]) -> io::Result<usize>;
    fn Seek(&mut self, distance: isize, method: SeekOrigin) -> io::Result<usize>;
    fn GetPosition(&self) -> usize;
    fn GetSize(&self) -> usize;
}

pub trait IBlueStream: IRoot + ICcpStream {
    fn LockData(&mut self, size: usize) -> io::Result<RwLockWriteGuard<'_, Vec<u8>>>;
    fn UnlockData(&mut self) -> bool;
}

pub trait IBlueMemStream: IBlueStream {
    fn SetBuffer(&mut self, buf: Option<Arc<RwLock<Vec<u8>>>>, size: usize) -> bool;
}

pub trait ICacheable: IRoot {
    fn IsMemoryUsageKnown(&self) -> bool;
    fn GetMemoryUsage(&self) -> usize;
}

#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum SeekOrigin {
    SO_BEGIN = 0,
    SO_CURRENT = 1,
    SO_END = 2,
}

/// `src-cpp/BlueMemStream.h::MemStream`. Supplied storage uses shared Rust
/// ownership; its contents are never mirrored in a Python-side stream.
pub struct MemStream {
    mData: Arc<RwLock<Vec<u8>>>,
    mSize: usize,
    mAllocSize: usize,
    mPosition: usize,
    mLocked: bool,
}

impl MemStream {
    pub fn new() -> Self {
        Self {
            mData: Arc::new(RwLock::new(Vec::new())),
            mSize: 0,
            mAllocSize: 0,
            mPosition: 0,
            mLocked: false,
        }
    }

    fn Grow(&mut self, reqSize: usize) -> io::Result<()> {
        if reqSize > self.mAllocSize {
            let size = reqSize.max(self.mAllocSize.saturating_mul(2));
            let mut data = self
                .mData
                .write()
                .map_err(|_| io::Error::other("stream buffer poisoned"))?;
            let extra = size.saturating_sub(data.len());
            data.try_reserve_exact(extra).map_err(io::Error::other)?;
            // Initialize formerly indeterminate C++ allocation bytes.
            data.resize(size, 0);
            self.mAllocSize = size;
        }
        Ok(())
    }

    pub fn SetSize(&mut self, newsize: usize) -> bool {
        if self.mLocked || (newsize > self.mSize && self.Grow(newsize).is_err()) {
            return false;
        }
        self.mSize = newsize;
        self.mPosition = self.mPosition.min(newsize);
        true
    }

    /// The source returns -1 without accessing either stream. This retains its
    /// defined behavior; it is not a migration placeholder.
    pub fn CopyFrom(&mut self, _source: &mut dyn IBlueStream, _count: usize) -> isize {
        -1
    }
}

impl Default for MemStream {
    fn default() -> Self {
        Self::new()
    }
}
impl IRoot for MemStream {}

impl ICcpStream for MemStream {
    fn Read(&mut self, dest: &mut [u8], count: isize) -> io::Result<usize> {
        let available = self.mSize - self.mPosition;
        let requested = if count < 0 {
            available
        } else {
            (count as usize).min(available)
        };
        let count = requested.min(dest.len());
        let data = self
            .mData
            .read()
            .map_err(|_| io::Error::other("stream buffer poisoned"))?;
        dest[..count].copy_from_slice(&data[self.mPosition..self.mPosition + count]);
        self.mPosition += count;
        Ok(count)
    }

    fn Write(&mut self, source: &[u8]) -> io::Result<usize> {
        let pos = self
            .mPosition
            .checked_add(source.len())
            .ok_or_else(|| io::Error::other("stream size overflow"))?;
        if pos > self.mSize {
            self.Grow(pos)?;
        }
        let mut data = self
            .mData
            .write()
            .map_err(|_| io::Error::other("stream buffer poisoned"))?;
        data[self.mPosition..pos].copy_from_slice(source);
        self.mPosition = pos;
        self.mSize = self.mSize.max(pos);
        Ok(source.len())
    }

    fn Seek(&mut self, distance: isize, method: SeekOrigin) -> io::Result<usize> {
        // The source uses size - distance for SO_END.
        let pos = match method {
            SeekOrigin::SO_BEGIN => usize::try_from(distance).ok(),
            SeekOrigin::SO_CURRENT => self.mPosition.checked_add_signed(distance),
            SeekOrigin::SO_END => distance
                .checked_neg()
                .and_then(|n| self.mSize.checked_add_signed(n)),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid stream position"))?;
        if pos > self.mSize {
            self.Grow(pos)?;
            self.mSize = pos;
        }
        self.mPosition = pos;
        Ok(pos)
    }

    fn GetPosition(&self) -> usize {
        self.mPosition
    }
    fn GetSize(&self) -> usize {
        self.mSize
    }
}

impl IBlueStream for MemStream {
    fn LockData(&mut self, _size: usize) -> io::Result<RwLockWriteGuard<'_, Vec<u8>>> {
        self.mLocked = true;
        self.mData
            .write()
            .map_err(|_| io::Error::other("stream buffer poisoned"))
    }
    fn UnlockData(&mut self) -> bool {
        self.mLocked = false;
        true
    }
}

impl IBlueMemStream for MemStream {
    fn SetBuffer(&mut self, buf: Option<Arc<RwLock<Vec<u8>>>>, size: usize) -> bool {
        let supplied = buf.is_some();
        let buffer = buf.unwrap_or_else(|| Arc::new(RwLock::new(Vec::new())));
        {
            let Ok(mut data) = buffer.write() else {
                return false;
            };
            if supplied && data.len() < size {
                return false;
            }
            if !supplied {
                let extra = size.saturating_sub(data.len());
                if data.try_reserve_exact(extra).is_err() {
                    return false;
                }
                data.resize(size, 0);
            }
        }
        self.mData = buffer;
        self.mAllocSize = if supplied { 0 } else { size };
        self.mSize = size;
        self.mPosition = 0;
        self.mLocked = false;
        true
    }
}

impl ICacheable for MemStream {
    fn IsMemoryUsageKnown(&self) -> bool {
        true
    }
    fn GetMemoryUsage(&self) -> usize {
        self.mAllocSize
    }
}
