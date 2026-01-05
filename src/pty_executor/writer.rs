use std::sync::{
    Arc, Mutex, MutexGuard,
    mpsc::{Sender, channel},
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
    inner: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
}

impl DetachableWriter {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(writer)),
        }
    }

    pub fn detach(&self) -> std::io::Result<Box<dyn std::io::Write + Send>> {
        self.replace(Box::new(std::io::sink()))
    }

    fn replace(
        &self,
        writer: Box<dyn std::io::Write + Send>,
    ) -> std::io::Result<Box<dyn std::io::Write + Send>> {
        let mut inner = self.lock_inner()?;
        Ok(std::mem::replace(&mut inner, writer))
    }

    fn lock_inner(&self) -> Result<MutexGuard<'_, Box<dyn std::io::Write + Send>>, std::io::Error> {
        self.inner
            .lock()
            .map_err(|e| std::io::Error::other(format!("Mutex poisoned: {e}")))
    }
}

impl std::io::Write for DetachableWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut inner = self.lock_inner()?;
        inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut inner = self.lock_inner()?;
        inner.flush()
    }
}
