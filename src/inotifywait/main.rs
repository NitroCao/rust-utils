use std::{collections::HashMap, fs, process};

use clap::{ArgAction, Parser};
use inotify::{Inotify, WatchDescriptor, WatchMask};

#[derive(Parser, Debug)]
#[command(name = "inotifywait", author, about)]
struct Args {
    #[arg(short, long = "event", action = ArgAction::Append)]
    /// Listen for specific event(s). If omitted, all events are listened for.
    events: Vec<String>,

    #[arg(short, default_value_t = false)]
    /// Keep listening for events forever or until --timeout expires.
    /// Without this option, inotifywait will exit after one event is received.
    monitor: bool,

    #[arg(short, long, default_value_t = false)]
    /// Watch directories recursively.
    recursive: bool,

    files: Vec<String>,
}

struct InotifywaitCmd {
    arg: Args,
    wd_map: HashMap<WatchDescriptor, String>,
    inotify: Option<Inotify>,
}

impl InotifywaitCmd {
    fn new(arg: Args) -> InotifywaitCmd {
        InotifywaitCmd {
            arg,
            wd_map: HashMap::new(),
            inotify: None,
        }
    }

    fn add_watch(&mut self, filename: String, flags: WatchMask) {
        let metadata = match fs::metadata(&filename) {
            Ok(data) => data,
            Err(err) => {
                println!(
                    "error when getting metadata of {}: {}",
                    &filename,
                    err.to_string()
                );
                return;
            }
        };
        if metadata.is_dir() {
            let dir_iter = match fs::read_dir(&filename) {
                Ok(iter) => iter,
                Err(err) => {
                    println!(
                        "error when reading directory {}: {}",
                        filename,
                        err.to_string()
                    );
                    process::exit(1);
                }
            };

            let iter = dir_iter.filter(|path| match path {
                Ok(entry) => match entry.file_type() {
                    Ok(file_type) => file_type.is_dir(),
                    Err(err) => {
                        println!(
                            "error when getting metadata of {}: {}",
                            entry.file_name().as_os_str().to_str().unwrap_or(""),
                            err.to_string()
                        );
                        process::exit(1);
                    }
                },
                Err(err) => {
                    println!(
                        "error when reading directory {}: {}",
                        filename,
                        err.to_string()
                    );
                    process::exit(1);
                }
            });
            for entry in iter {
                match entry {
                    Ok(dir_entry) => {
                        let new_filename = dir_entry.path().to_string_lossy().to_string();
                        self.add_watch(new_filename, flags);
                    }
                    Err(err) => {
                        println!(
                            "error when reading directory {}: {}",
                            filename,
                            err.to_string()
                        );
                        process::exit(1);
                    }
                }
            }
        }
        self.wd_map.insert(
            match self
                .inotify
                .as_mut()
                .expect("unexpected null inotify instance")
                .add_watch(&filename, flags)
            {
                Ok(wd) => wd,
                Err(err) => {
                    println!(
                        "failed to add inotify watch for {}: {}",
                        filename,
                        err.to_string()
                    );
                    return;
                }
            },
            filename,
        );
    }

    fn run(&mut self) {
        let flags = self.setup_flags();

        println!("Setting up watches.");
        if self.arg.files.len() == 0 {
            println!("No files specified to watch!");
            return;
        }

        self.inotify = Some(Inotify::init().unwrap_or_else(|err| {
            println!("failed to initialize inotify: {}", err.to_string());
            process::exit(1);
        }));

        for i in 0..self.arg.files.len() {
            let filename = self.arg.files.remove(i);
            self.add_watch(filename, flags);
        }
        println!("Watches established.");

        let mut buff = [0u8; 4096];
        'outer: loop {
            let events = self
                .inotify
                .as_mut()
                .expect("unexpected null inotify instance")
                .read_events_blocking(&mut buff)
                .expect("failed to read inotify event");
            for event in events {
                let filename = event.name.map_or("", |n| n.to_str().map_or("", |n| n));
                println!(
                    "{}\t{:?} {}",
                    self.wd_map.get(&event.wd).map_or("", |path| path),
                    event.mask,
                    filename
                );
                if !self.arg.monitor {
                    break 'outer;
                }
            }
        }
    }

    fn setup_flags(&self) -> WatchMask {
        let args = &self.arg;
        match args.events.len() {
            0 => {
                WatchMask::ACCESS
                    | WatchMask::ATTRIB
                    | WatchMask::CLOSE_NOWRITE
                    | WatchMask::CLOSE_WRITE
                    | WatchMask::CREATE
                    | WatchMask::DELETE
                    | WatchMask::DELETE_SELF
                    | WatchMask::MODIFY
                    | WatchMask::MOVED_FROM
                    | WatchMask::MOVED_TO
                    | WatchMask::MOVE_SELF
                    | WatchMask::OPEN
            }
            _ => {
                let mut flags: WatchMask = WatchMask::empty();
                for each in &args.events {
                    flags |= match each.as_str() {
                        "access" => WatchMask::ACCESS,
                        "modify" => WatchMask::MODIFY,
                        "attrib" => WatchMask::ATTRIB,
                        "close_write" => WatchMask::CLOSE_WRITE,
                        "close_nowrite" => WatchMask::CLOSE_NOWRITE,
                        "close" => WatchMask::CLOSE,
                        "open" => WatchMask::OPEN,
                        "moved_to" => WatchMask::MOVED_TO,
                        "moved_from" => WatchMask::MOVED_FROM,
                        "move" => WatchMask::MOVE,
                        "moved_self" => WatchMask::MOVE_SELF,
                        "create" => WatchMask::CREATE,
                        "delete" => WatchMask::DELETE,
                        "delete_self" => WatchMask::DELETE_SELF,
                        _ => {
                            println!("unknown event type: {}", each);
                            process::exit(1);
                        }
                    }
                }
                flags
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let mut cmd = InotifywaitCmd::new(args);
    cmd.run();
}
