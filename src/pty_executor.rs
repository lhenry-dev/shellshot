use anyhow::{anyhow, Result};
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, PtySize};
use std::{
    io::{BufReader, BufWriter, Read, Write},
    panic, thread,
    time::Duration,
};
use termwiz::surface::Surface;

use crate::{
    constants::{MAX_HEIGHT, MAX_WIDTH},
    pty_executor::{
        dimension::Dimension,
        writer::{DetachableWriter, ThreadedWriter},
    },
    terminal_builder::TerminalBuilder,
};

pub mod dimension;
mod writer;

pub struct PtyIO {
    pub reader: BufReader<Box<dyn Read + Send>>,
    pub writer: DetachableWriter,
}

#[derive(Clone)]
pub struct PtyOptions {
    pub cols: Dimension,
    pub rows: Dimension,
}

pub struct PtyExecutor {}

impl PtyExecutor {
    pub fn run_command(pty_options: &PtyOptions, command: &[String]) -> Result<Surface> {
        if command.is_empty() {
            return Err(anyhow!("Empty command provided"));
        }

        let cmd_name = &command[0];
        let args = &command[1..];

        println!("Executing command in PTY: {:?}, {:?}", cmd_name, args);

        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            cols: pty_options.cols.to_u16(MAX_WIDTH),
            rows: pty_options.rows.to_u16(MAX_HEIGHT),
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(cmd_name);
        cmd.args(args);

        if cmd.get_cwd().is_none() {
            cmd.cwd(".");
        }

        let reader = BufReader::new(pair.master.try_clone_reader()?);
        let mut child = pair.slave.spawn_command(cmd)?;
        let killer = child.clone_killer();

        let writer = pair.master.take_writer()?;
        let writer = ThreadedWriter::new(Box::new(BufWriter::new(writer)));
        let writer = DetachableWriter::new(Box::new(BufWriter::new(writer)));

        let pty_process = PtyIO {
            reader,
            writer: writer.clone(),
        };
        let cols = &pty_options.cols;
        let rows = &pty_options.rows;

        thread::scope(|s| -> anyhow::Result<Surface> {
            let handle = s.spawn(|| TerminalBuilder::run(pty_process, cols, rows));

            with_timeout(None, killer, s, || child.wait())?;

            writer.detach().flush()?;
            drop(child);
            drop(pair);

            let surface = handle.join().unwrap()?;

            Ok(surface)
        })
    }
}

fn with_timeout<'scope, R, F>(
    timeout: Option<Duration>,
    mut killer: Box<dyn ChildKiller + Send + Sync>,
    s: &'scope thread::Scope<'scope, '_>,
    f: F,
) -> R
where
    F: FnOnce() -> R,
{
    if let Some(timeout) = timeout {
        let t = s.spawn(move || {
            thread::park_timeout(timeout);
            let _ = killer.kill();
        });
        let result = panic::catch_unwind(panic::AssertUnwindSafe(f));
        t.thread().unpark();
        t.join().unwrap();
        result.unwrap()
    } else {
        f()
    }
}
