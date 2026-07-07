#![no_std]
#![forbid(unsafe_code)]
//#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
// That's just a bad lint, in many cases I want two ifs for readability
#![allow(clippy::collapsible_if)]

#[macro_export]
macro_rules! awrite {
    ($dst:expr, $($arg:tt)*) => {
        $dst.write_fmt_async(::core::format_args!($($arg)*))
    };
    ($($arg:tt)*) => {
        compile_error!("requires a destination and format arguments, like `awrite!(dest, \"format string\", args...)`")
    };
}

#[macro_export]
macro_rules! awriteln {
    ($dst:expr $(,)?) => {
        $crate::awrite!($dst, "\n")
    };
    ($dst:expr, $fmt:literal $(, $($arg:tt)*)?) => {
        $dst.write_fmt_async(
            ::core::format_args!(
                concat!($fmt, "\n")
                $(, $($arg)*)?
            )
        )
    };
    ($($arg:tt)*) => {
        compile_error!("requires a destination and format arguments, like `awriteln!(dest, \"format string\", args...)`")
    };
}

pub struct FormatWriter<'a, T: AsyncWriteTarget> {
    writer: T,
    buffer: &'a mut [u8],
}

impl<'a, T: AsyncWriteTarget> FormatWriter<'a, T> {
    pub async fn write_fmt_async(
        &mut self,
        args: core::fmt::Arguments<'_>,
    ) -> Result<(), T::Error> {
        use core::fmt::Write as _;
        // The actual logic of this crate.
        // Format the string repeatedly in chunks until it is successfully processed.
        // This assumes that formatting the string multiple times yields the same result every time.
        // I know there are cornercases where this might be wrong, but the problem is impossible to solve otherwise.

        struct SkippingCursor<'a> {
            leftover_skips: usize,
            offset: usize,
            buffer: &'a mut [u8],
        }

        impl core::fmt::Write for SkippingCursor<'_> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let s = s.as_bytes();

                let num_skipped = s.len().min(self.leftover_skips);

                self.leftover_skips -= num_skipped;

                let s = &s[num_skipped..];
                if s.is_empty() {
                    return Ok(());
                }

                let write_len = s.len().min(self.buffer.len() - self.offset);
                self.buffer[self.offset..self.offset + write_len].copy_from_slice(&s[..write_len]);
                self.offset += write_len;

                if write_len != s.len() {
                    Err(core::fmt::Error)
                } else {
                    Ok(())
                }
            }
        }

        let mut num_processed = 0;

        loop {
            let mut cursor = SkippingCursor {
                leftover_skips: num_processed,
                offset: 0,
                buffer: self.buffer,
            };

            let write_result = cursor.write_fmt(args);

            if cursor.offset != 0 {
                self.writer.write(&cursor.buffer[..cursor.offset]).await?;
                num_processed += cursor.offset;
            }

            if write_result.is_ok() {
                break Ok(());
            }
        }
    }
}

pub trait AsyncWriteTarget: Sized {
    type Error: core::error::Error;

    fn flush(&mut self) -> impl core::future::Future<Output = Result<(), Self::Error>>;
    fn write(&mut self, buf: &[u8]) -> impl core::future::Future<Output = Result<(), Self::Error>>;

    fn into_format_writer<'a, T: AsMut<[u8]>>(self, buf: &'a mut T) -> FormatWriter<'a, Self> {
        FormatWriter {
            writer: self,
            buffer: buf.as_mut(),
        }
    }
}

impl<T> AsyncWriteTarget for T
where
    T: embedded_io_async::Write,
{
    type Error = T::Error;

    fn flush(&mut self) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        T::flush(self)
    }

    fn write(&mut self, buf: &[u8]) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        T::write_all(self, buf)
    }
}
