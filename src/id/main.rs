use std::{ffi::CStr, process, ptr::null_mut};

use clap::Parser;
use libc::{getgrgid, getpwuid, gid_t, uid_t};

#[derive(Parser, Debug)]
#[command(author, version, name = "id")]
struct Args {
    #[arg(short, long, default_value_t = false)]
    /// Print only the effective user ID
    user: bool,

    #[arg(short, long, default_value_t = false)]
    /// Print the real ID instead of the effective ID
    real: bool,

    #[arg(short, long, default_value_t = false)]
    /// Print only the effective group ID
    group: bool,

    #[arg(short = 'G', long, default_value_t = false)]
    /// Get all group IDs
    groups: bool,

    #[arg(short, long, default_value_t = false)]
    /// Print a name instead of a number
    name: bool,

    #[arg(short, long, default_value_t = false)]
    /// Delimit entries with NUL characters, not whitespace
    zero: bool,

    ids: Vec<String>,
}

struct IdCmd<'a> {
    arg: Args,
    sep: &'a str,
}

impl<'a> IdCmd<'a> {
    fn new(arg: Args) -> IdCmd<'a> {
        let mut sep = " ";
        if arg.zero {
            sep = "\0";
        }

        IdCmd { arg, sep }
    }

    fn run(&self) {
        self.check_args();

        let args = &self.arg;
        if args.ids.len() == 0 {
            if args.user {
                self.print_user_only();
            } else if args.group {
                self.print_group_only();
            } else if args.groups {
                self.print_groups_only();
            } else {
                self.print_user_info();
            }
        }
    }

    fn check_args(&self) {
        let args = &self.arg;
        if (args.real || args.name) && !(args.user || args.group || args.groups) {
            println!("cannot print only names or real IDs in default format");
            process::exit(1);
        }
    }

    fn print_user_only(&self) {
        let uid = IdCmd::getuid(!self.arg.real);
        if self.arg.name {
            match IdCmd::get_username_by_uid(uid) {
                Some(name) => {
                    println!("{}", name);
                }
                None => {
                    println!("(cannot find name for user ID {})", uid);
                    process::exit(1);
                }
            }
            return;
        }
        println!("{}", uid);
    }

    fn print_group_only(&self) {
        let gid = IdCmd::getgid(!self.arg.real);
        if self.arg.name {
            match IdCmd::get_groupname_by_gid(gid) {
                Some(name) => {
                    println!("{}", name);
                }
                None => {
                    println!("(cannot find name for group ID {})", gid);
                    process::exit(1);
                }
            }
            return;
        }
        println!("{}", gid);
    }

    fn print_groups_only(&self) {
        let groups = IdCmd::getgroups();
        for i in 0..groups.len() {
            if self.arg.name {
                match IdCmd::get_groupname_by_gid(groups[i]) {
                    Some(name) => {
                        print!("{}", name);
                    }
                    None => {
                        print!("(cannot find name for group ID {})", groups[i]);
                        process::exit(1);
                    }
                }
            } else {
                print!("{}", groups[i]);
            }
            if i < groups.len() - 1 {
                print!(" ");
            }
        }
    }

    fn print_user_info(&self) {
        let uid = IdCmd::getuid(false);
        let gid = IdCmd::getgid(false);

        unsafe {
            let username = match IdCmd::get_username_by_uid(uid) {
                Some(name) => String::from(name),
                None => {
                    format!("cannot find name for user ID {}", uid)
                }
            };
            print!("uid={}({}){}", uid, username.as_str(), self.sep);

            let group_name = match IdCmd::get_groupname_by_gid(gid) {
                Some(name) => String::from(name),
                None => {
                    format!("cannot find name for group ID {}", gid)
                }
            };
            print!("gid={}({})", gid, group_name);

            let euid = IdCmd::getuid(true);
            if euid != uid {
                let ename = match IdCmd::get_username_by_uid(euid) {
                    Some(name) => String::from(name),
                    None => {
                        format!("cannot find name for user ID {}", euid)
                    }
                };
                print!("{}euid={}({})", self.sep, euid, ename);
            }
            let egid = IdCmd::getgid(true);
            if egid != gid {
                let ename = match IdCmd::get_groupname_by_gid(egid) {
                    Some(name) => String::from(name),
                    None => {
                        format!("cannot find name for user ID {}", egid)
                    }
                };
                print!("{}egid={}({})", self.sep, egid, ename);
            }

            let groups_num = libc::getgroups(0, null_mut::<u32>());
            if groups_num <= 0 {
                return;
            }

            let mut groups = vec![0 as gid_t; groups_num as usize];
            if libc::getgroups(groups_num, groups.as_mut_ptr()) == -1 {
                println!("failed to get groups");
                return;
            }
            print!("{}groups=", self.sep);
            for i in 0..groups.len() {
                print!("{}", groups[i]);

                let group_name = match IdCmd::get_groupname_by_gid(groups[i]) {
                    Some(name) => String::from(name),
                    None => {
                        format!("cannot find name for group ID {}", groups[i])
                    }
                };
                print!("({})", group_name);
                if i != groups.len() - 1 {
                    print!(",");
                }
            }
        }
    }

    fn getuid(effective: bool) -> uid_t {
        unsafe {
            if effective {
                libc::geteuid()
            } else {
                libc::getuid()
            }
        }
    }

    fn getgid(effective: bool) -> gid_t {
        unsafe {
            if effective {
                libc::getegid()
            } else {
                libc::getgid()
            }
        }
    }

    fn getgroups() -> Vec<gid_t> {
        let groups_num;
        unsafe {
            groups_num = libc::getgroups(0, null_mut::<u32>());
            if groups_num <= 0 {
                return vec![0];
            }
        }

        let mut groups = vec![0 as gid_t; groups_num as usize];
        unsafe {
            if libc::getgroups(groups_num, groups.as_mut_ptr()) == -1 {
                return vec![0];
            }
        }
        groups
    }

    fn get_username_by_uid(uid: uid_t) -> Option<&'static str> {
        unsafe {
            let entry = getpwuid(uid);
            if !entry.is_null() {
                let username = CStr::from_ptr((*entry).pw_name).to_str().unwrap();
                return Some(username);
            } else {
                return None;
            }
        }
    }

    fn get_groupname_by_gid(gid: gid_t) -> Option<&'static str> {
        unsafe {
            let entry = getgrgid(gid);
            if !entry.is_null() {
                let groupname = CStr::from_ptr((*entry).gr_name).to_str().unwrap();
                return Some(groupname);
            }
            return None;
        }
    }
}

pub fn main() {
    let args = Args::parse();
    let cmd = IdCmd::new(args);
    cmd.run();
}
