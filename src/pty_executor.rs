use anyhow::{anyhow, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use std::io::{BufReader, Read, Write};

pub struct PtyProcess {
    pub child: Box<dyn Child + Send + Sync>,
    pub reader: BufReader<Box<dyn Read + Send>>,
    pub writer: Box<dyn Write + Send>,
}

pub struct PtyOptions {
    pub rows: u16,
    pub cols: u16,
}

pub struct PtyExecutor {
    pair: PtyPair,
}

impl PtyExecutor {
    pub fn new(pty_options: &PtyOptions) -> Result<Self> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows: pty_options.rows,
            cols: pty_options.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        Ok(Self { pair })
    }

    pub fn run_command(&self, command: &[String]) -> Result<PtyProcess> {
        if command.is_empty() {
            return Err(anyhow!("Empty command provided"));
        }

        let cmd_name = &command[0];
        let args = &command[1..];

        println!("Executing command in PTY: {:?}, {:?}", cmd_name, args);

        let mut cmd = CommandBuilder::new(cmd_name);
        cmd.args(args);

        if cmd.get_cwd().is_none() {
            cmd.cwd(".");
        }

        let reader = BufReader::new(self.pair.master.try_clone_reader()?);
        let child = self.pair.slave.spawn_command(cmd)?;
        let writer = self.pair.master.take_writer()?;

        Ok(PtyProcess {
            child,
            reader,
            writer,
        })
    }
}
