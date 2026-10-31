//! Fallible stdout writes. A closed pipe discards output, not command results.

use std::io::{self, Write};

fn allow_closed_pipe(result: io::Result<()>) -> io::Result<()> {
    match result {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

fn write_to(writer: &mut impl Write, args: std::fmt::Arguments<'_>) -> io::Result<()> {
    allow_closed_pipe(writer.write_fmt(args))?;
    // Stdout can accept a short write into its buffer and fail only on flush.
    allow_closed_pipe(writer.flush())
}

pub(crate) fn write(args: std::fmt::Arguments<'_>) -> Result<(), String> {
    write_to(&mut io::stdout().lock(), args).map_err(|e| format!("cannot write stdout: {e}"))
}

pub(crate) fn line(args: std::fmt::Arguments<'_>) -> Result<(), String> {
    write(format_args!("{args}\n"))
}

pub(crate) fn flush() -> Result<(), String> {
    allow_closed_pipe(io::stdout().lock().flush()).map_err(|e| format!("cannot flush stdout: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Failing {
        write_error: Option<io::ErrorKind>,
        flush_error: Option<io::ErrorKind>,
    }

    impl Write for Failing {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            match self.write_error {
                Some(kind) => Err(kind.into()),
                None => Ok(bytes.len()),
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            match self.flush_error {
                Some(kind) => Err(kind.into()),
                None => Ok(()),
            }
        }
    }

    #[test]
    fn normal_output_is_preserved() {
        let mut bytes = Vec::new();
        write_to(&mut bytes, format_args!("hello {}\n", 42)).unwrap();
        assert_eq!(bytes, b"hello 42\n");
    }

    #[test]
    fn only_broken_pipe_is_ignored_at_write_and_flush() {
        for kind in [io::ErrorKind::BrokenPipe, io::ErrorKind::PermissionDenied] {
            for during_write in [true, false] {
                let mut writer = Failing {
                    write_error: during_write.then_some(kind),
                    flush_error: (!during_write).then_some(kind),
                };
                let result = write_to(&mut writer, format_args!("output"));
                if kind == io::ErrorKind::BrokenPipe {
                    result.unwrap();
                } else {
                    assert_eq!(result.unwrap_err().kind(), kind);
                }
            }
        }
    }
}
