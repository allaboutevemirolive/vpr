use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Buffer<W: Write> {
    pub bufwr: io::BufWriter<W>,
}

impl<W: Write> Buffer<W> {
    pub fn new(writer: W) -> anyhow::Result<Self> {
        tracing::info!("Initializing Buffer");

        let bufwr = io::BufWriter::new(writer);
        Ok(Buffer { bufwr })
    }

    pub fn write_message(&mut self, message: &str) -> io::Result<()> {
        self.bufwr.write_all(message.as_bytes())
    }

    pub fn write_os_string(&mut self, message: String) -> io::Result<()> {
        self.bufwr.write_all(message.as_bytes())
    }
}

impl<W: Write> Buffer<W> {
    pub fn newline(&mut self) -> io::Result<()> {
        self.bufwr.write_all("\n".as_bytes())
    }

    pub fn write_space(&mut self) -> io::Result<()> {
        self.bufwr.write_all(" ".as_bytes())
    }
}
