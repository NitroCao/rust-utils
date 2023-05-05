use std::{collections::HashMap, process};

use clap::{ArgAction, Parser};
use inotify::{Inotify, WatchMask};

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

    files: Vec<String>,
}

struct InotifywaitCmd {
    arg: Args,
}

impl InotifywaitCmd {
    fn new(arg: Args) -> InotifywaitCmd {
        InotifywaitCmd { arg }
    }

    fn run(&self) {
        let flags = self.setup_flags();

        let mut inotify = Inotify::init().unwrap_or_else(|err| {
            println!("failed to initialize inotify: {}", err.to_string());
            process::exit(1);
        });

        println!("Setting up watches.");
        let args = &self.arg;
        if args.files.len() == 0 {
            println!("No files specified to watch!");
            return;
        }
        let mut wd_map = HashMap::with_capacity(args.files.len());
        for filename in &args.files {
            wd_map.insert(
                match inotify.add_watch(&filename, flags) {
                    Ok(wd) => wd,
                    Err(err) => {
                        println!(
                            "failed to add inotify watch for {}: {}",
                            &filename,
                            err.to_string()
                        );
                        continue;
                    }
                },
                filename,
            );
        }
        println!("Watches established.");

        let mut buff = [0u8; 4096];
        'outer: loop {
            let events = inotify
                .read_events_blocking(&mut buff)
                .expect("failed to read inotify event");
            for event in events {
                let filename = event.name.map_or("", |n| n.to_str().map_or("", |n| n));
                println!(
                    "{}\t{:?} {}",
                    wd_map.get(&event.wd).map_or("", |path| path),
                    event.mask,
                    filename
                );
                if !args.monitor {
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
    let cmd = InotifywaitCmd::new(args);
    cmd.run();
}
