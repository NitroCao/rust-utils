use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// echo - display a line of text
struct Args {
    #[arg(short, required = false, default_value_t = false)]
    /// Do not output the trailing newline
    no_newline: bool,

    /// Enable interpretation of backslash escapes
    #[arg(short, required = false, default_value_t = false)]
    escape_sequence: bool,

    /// strings
    input: Vec<String>,
}

fn convert_number(input: &mut String, base: u32, idx: usize) {
    let sep_begin = idx + 2;
    if sep_begin >= input.len() {
        input.replace_range(idx..idx + 2, "");
        return;
    }
    let mut sep_end = sep_begin;
    let num_len = match base {
        8 => 3,
        16 => 2,
        _ => {
            return;
        }
    };
    for i in 1..num_len {
        match input.chars().nth(sep_begin + i) {
            Some(ch) => {
                if ch == '\\' {
                    break;
                }
                sep_end = sep_begin + i;
            }
            None => {}
        }
    }
    sep_end += 1;

    match u32::from_str_radix(&input[sep_begin..sep_end], base) {
        Ok(n) => match char::from_u32(n) {
            Some(ch) => {
                if ch.is_ascii() {
                    input.replace_range(idx..sep_end, &ch.to_string());
                }
            }
            None => input.replace_range(idx..sep_end, ""),
        },
        Err(_) => {}
    }
}

fn process_escapes(input: &mut String) {
    *input = input
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\a", "\x07")
        .replace("\\b", "\x08")
        .replace("\\c", "\x00")
        .replace("\\e", "\x1b")
        .replace("\\f", "\x0c")
        .replace("\\r", "\r")
        .replace("\\v", "\x0b");

    match input.find('\x00') {
        Some(idx) => {
            *input = input.chars().take(idx).collect();
        }
        None => {}
    }

    loop {
        match input.find("\\x") {
            Some(idx) => {
                convert_number(input, 16, idx);
            }
            None => break,
        }
    }

    loop {
        match input.find("\\0") {
            Some(idx) => {
                convert_number(input, 8, idx);
            }
            None => {
                break;
            }
        }
    }
}

pub fn main() {
    let mut args = Args::parse();

    let input_count = args.input.len();
    for idx in 0..(input_count) {
        let s = &mut args.input[idx];
        if args.escape_sequence {
            process_escapes(s);
        }

        if idx == input_count - 1 {
            print!("{}", s);
        } else {
            print!("{} ", s);
        }
    }

    if !args.no_newline {
        println!("");
    }
}
