use std::{
    fs::File,
    io::{stdin, BufRead, BufReader, Read, Result},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    /// Number all output lines
    number: bool,

    #[arg(short = 'b', long, default_value_t = false)]
    /// Number nonempty output lines, overrides -n
    number_nonblack: bool,

    #[arg(short = 'T', long, default_value_t = false)]
    /// Display TAB characters as ^I
    show_tabs: bool,

    #[arg(short = 'E', long, default_value_t = false)]
    /// Display $ at the end of each line
    show_ends: bool,

    #[arg(short, long, default_value_t = false)]
    /// suppress repeated empty outline lines
    squeeze_blank: bool,

    #[arg(short = 'v', long, default_value_t = false)]
    /// use ^ and M- notation, except for LFD and TAB
    show_nonprinting: bool,

    #[arg(short = 'e', default_value_t = false)]
    /// equivalent to -vE
    show_noprinting_and_ends: bool,

    #[arg(short = 't', default_value_t = false)]
    /// equivalent to -vT
    show_noprinting_and_tabs: bool,

    files: Vec<String>,
}

struct CatCmd {
    arg: Args,
}

impl CatCmd {
    fn new(arg: Args) -> CatCmd {
        CatCmd { arg }
    }

    fn run(&mut self) {
        self.check_args();
        self.process_file();
    }

    fn check_args(&mut self) {
        let args = &mut self.arg;
        if args.number_nonblack {
            args.number = false;
        }
        if args.show_noprinting_and_ends {
            args.show_nonprinting = true;
            args.show_ends = true;
        }
        if args.show_noprinting_and_tabs {
            args.show_nonprinting = true;
            args.show_tabs = true;
        }
    }

    fn open_file(filename: &str) -> Result<Box<dyn Read>> {
        match filename {
            "-" => Ok(Box::new(stdin())),
            _ => Ok(Box::new(File::open(&filename)?)),
        }
    }

    fn is_emptyline(line: &String) -> bool {
        if (line.len() == 1 && line == "\n") || (line.len() == 2 && line == "\r\n") {
            return true;
        }
        false
    }

    fn process_file(&self) {
        let args = &self.arg;
        for idx in 0..args.files.len() {
            let filename = &args.files[idx];
            let input = match CatCmd::open_file(filename.as_str()) {
                Ok(file) => file,
                Err(err) => {
                    println!("{}: {}", filename, err.to_string());
                    continue;
                }
            };
            let mut reader = BufReader::new(input);

            let mut line_num = 1;
            let mut buff = String::new();
            let mut has_emptyline = false;
            while let Ok(n) = reader.read_line(&mut buff) {
                if n == 0 {
                    break;
                }

                let line_num_str = format!("{}", line_num);
                let padding = " ".repeat(6 - line_num_str.len());
                if args.number || (args.number_nonblack && !CatCmd::is_emptyline(&buff)) {
                    print!("{}{:>}\t", padding, line_num_str);
                    line_num += 1;
                }

                if CatCmd::is_emptyline(&buff) && args.squeeze_blank {
                    if has_emptyline {
                        buff.clear();
                        continue;
                    } else {
                        has_emptyline = true;
                    }
                } else {
                    has_emptyline = false;
                }

                for ch in buff.chars() {
                    match ch {
                        '\t' => {
                            if args.show_tabs {
                                print!("^I");
                                continue;
                            }
                            print!("{}", ch);
                        }
                        '\n' => {
                            if args.show_ends {
                                print!("$");
                            }
                            print!("{}", ch);
                        }
                        other => {
                            if args.show_nonprinting && (other as u8) < 0x20 {
                                print!("^{}", char::from(other as u8 + 64));
                            } else {
                                print!("{}", other);
                            }
                        }
                    }
                }
                buff.clear();
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let mut cmd = CatCmd::new(args);
    cmd.run();
}
