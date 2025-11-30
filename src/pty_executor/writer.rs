use std::{
    io, mem,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};

#[derive(Clone)]
pub struct ThreadedWriter {
    sender: Sender<WriterMessage>,
}

enum WriterMessage {
    Data(Vec<u8>),
    Flush,
}

impl ThreadedWriter {
    pub fn new(mut writer: Box<dyn std::io::Write + Send>) -> Self {
        let (sender, receiver) = channel::<WriterMessage>();

        std::thread::spawn(move || {
            while let Ok(msg) = receiver.recv() {
                match msg {
                    WriterMessage::Data(buf) => {
                        if writer.write(&buf).is_err() {
                            break;
                        }
                    }
                    WriterMessage::Flush => {
                        if writer.flush().is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self { sender }
    }
}

impl std::io::Write for ThreadedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sender
            .send(WriterMessage::Data(buf.to_vec()))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::BrokenPipe, err))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.sender
            .send(WriterMessage::Flush)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::BrokenPipe, err))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DetachableWriter {
    inner: Arc<Mutex<Box<dyn io::Write + Send>>>,
}

impl DetachableWriter {
    /// Creates a new detachable writer.
    pub fn new(writer: Box<dyn io::Write + Send>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(writer)),
        }
    }

    /// Detaches the current writer and replaces it with a sink.
    pub fn detach(&self) -> Box<dyn io::Write + Send> {
        self.replace(Box::new(io::sink()))
    }

    /// Replaces the current writer with a new one.
    fn replace(&self, writer: Box<dyn io::Write + Send>) -> Box<dyn io::Write + Send> {
        let mut inner = self.inner.lock().unwrap();
        mem::replace(&mut inner, writer)
    }
}

impl io::Write for DetachableWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}
