use std::{
    f64::consts::PI,
    io::{Stdout, Write as _, stdout},
    time::Duration,
};

use async_format::{AsyncWriteTarget as _, awriteln};

struct SlowWriter(Stdout);

impl embedded_io_async::ErrorType for SlowWriter {
    type Error = std::io::Error;
}

impl embedded_io_async::Write for SlowWriter {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let num_written = self.0.write(buf)?;
        self.0.flush()?;
        Ok(num_written)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

#[tokio::main]
async fn main() {
    let mut buf = [0u8; 8];

    let mut writer = SlowWriter(stdout()).format_writer(&mut buf);

    awriteln!(
        writer,
        "This was written asynchronously: {:?}. Not bad, eh?",
        Some(PI)
    )
    .await
    .unwrap();
}
