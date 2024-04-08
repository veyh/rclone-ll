use anyhow::Result;
use clap::Parser;
use colorful::{Color, Colorful, Style};
use serde::Deserialize;
use std::print;
use subprocess::{Exec, Redirection};

#[derive(Parser, Debug)]
pub struct AppArgs {
    /// Reverse the list.
    #[arg(long, short, default_value_t = false)]
    pub reverse: bool,

    /// Sort by time.
    #[arg(short = 't', default_value_t = false)]
    pub sort_by_time: bool,

    #[arg(default_value = ".")]
    pub path: String,
}

#[derive(Debug)]
pub struct App {
    args: AppArgs,
}

const KIB: i64 = 1024;
const MIB: i64 = 1024 * 1024;
const GIB: i64 = 1024 * 1024 * 1024;
const TIB: i64 = 1024 * 1024 * 1024 * 1024;

impl App {
    pub fn run() -> Result<()> {
        let args = AppArgs::parse();
        let mut app = App { args };
        app.run_internal()
    }

    fn run_internal(&mut self) -> Result<()> {
        let out = Exec::cmd("rclone")
            .arg("lsjson")
            .arg("--max-depth")
            .arg("1")
            .arg(&self.args.path)
            .stdout(Redirection::Pipe)
            .capture()?
            .stdout_str();

        let mut entries: Vec<DirEntry> = serde_json::from_str(&out)?;

        if self.args.sort_by_time {
            entries.sort_by(|a, b| b.mod_time.cmp(&a.mod_time));
        }

        if self.args.reverse {
            entries.reverse();
        }

        for entry in entries.iter() {
            self.print_time(entry.mod_time.clone());
            self.print_size(entry.size);
            self.print_path(&entry);
        }

        Ok(())
    }

    fn print_time(&self, time: String) {
        print!("{}\t", format!("[{}]", time).color(Color::Green));
    }

    fn print_size(&self, size: i64) {
        let size_formatted = match size {
            _ if size <= 0 => {
                "".to_string()
            }

            _ if size < KIB => {
                format!("{:6.}B", size)
            }

            _ if size < MIB => {
                let value = (size as f64) / (KIB as f64);
                format!("{:6.1}K", value)
            }

            _ if size < GIB => {
                let value = (size as f64) / (MIB as f64);
                format!("{:6.1}M", value)
            }

            _ if size < TIB => {
                let value = (size as f64) / (GIB as f64);
                format!("{:6.1}G", value)
            }

            _ => {
                let value = (size as f64) / (TIB as f64);
                format!("{:6.1}T", value)
            }
        };

        print!("{}\t", size_formatted.color(Color::Yellow));
    }

    fn print_path(&self, entry: &DirEntry) {
        if entry.is_dir {
            let value = format!("{}/", entry.path)
                .color(Color::Cyan1)
                .style(Style::Bold);

            print!("{}\n", value);
        }

        else {
            print!("{}\n", entry.path);
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DirEntry {
    path: String,
    name: String,
    size: i64,
    mod_time: String,

    #[serde(default)]
    mime_type: String,

    #[serde(default)]
    is_dir: bool,

    #[serde(default)]
    is_bucket: bool,
}
