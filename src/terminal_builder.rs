use anyhow::Result;
use std::io::BufRead;
use termwiz::escape::parser::Parser;
use termwiz::surface::Surface;

use crate::pty_executor::PtyProcess;
use crate::terminal_builder::action::process_action;

mod action;
mod constants;
mod utils;

pub struct TerminalBuilder {
    pty_process: PtyProcess,
    surface: Surface,
}

impl TerminalBuilder {
    pub fn run(pty_process: PtyProcess, rows: u16, cols: u16) -> Result<Surface> {
        let mut terminal = Self {
            pty_process,
            surface: Surface::new(cols.into(), rows.into()),
        };
        terminal.run_loop()
    }

    fn run_loop(&mut self) -> Result<Surface> {
        let child = &mut self.pty_process.child;
        let reader = &mut self.pty_process.reader;
        let writer = &mut self.pty_process.writer;
        let surface = &mut self.surface;

        let mut parser = Parser::new();

        loop {
            let buf = reader.fill_buf().expect("error reading PTY");
            if buf.is_empty() {
                break;
            }

            let mut actions = Vec::new();
            parser.parse(buf, |action| action.append_to(&mut actions));

            for action in actions {
                process_action(surface, writer, action);
            }

            let len = buf.len();
            reader.consume(len);

            if let Some(status) = child.try_wait()? {
                println!("\nProcess exited: {:?}", status);
                break;
            }
        }

        let screen = self.surface.screen_cells();
        for line in screen {
            for cell in line {
                print!("{}", cell.str());
            }
            println!();
        }

        Ok(self.surface.clone())
    }
}
