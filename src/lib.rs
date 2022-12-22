use clap::{value_parser, Arg, ArgAction, Command};
use std::io::{BufRead, BufReader, Read};

pub fn run(config: Config) -> anyhow::Result<()> {
    let num_files = config.files.len();

    for (file_num, file_name) in config.files.iter().enumerate() {
        match open(&file_name) {
            Err(err) => eprintln!("{}: {}", file_name, err),
            Ok(mut file) => {
                if config.verbose && !config.quiet {
                    if num_files > 1 {
                        println!(
                            "{}==> {} <==",
                            if file_num > 0 { "\n" } else { "" },
                            file_name
                        );
                    }
                }

                if let Some(num_bytes) = config.bytes {
                    if num_bytes >= 0 {
                        let bytes: Result<Vec<_>, _> =
                            file.bytes().take(num_bytes as usize).collect();
                        print!("{}", String::from_utf8_lossy(&bytes?));
                    } else {
                        let mut bytes = file.bytes().collect::<Result<Vec<_>, _>>()?;
                        let n = bytes.len() as i32;
                        bytes = bytes.into_iter().skip((n + num_bytes) as usize).collect();
                        print!("{}", String::from_utf8_lossy(&bytes));
                    }
                } else {
                    let mut line = String::new();
                    if config.lines >= 0 {
                        for _ in 0..config.lines {
                            let bytes = file.read_line(&mut line)?;
                            if bytes == 0 {
                                break;
                            }
                            print!("{}", line);
                            line.clear();
                        }
                    } else {
                        let bytes = file.bytes().collect::<Result<Vec<_>, _>>()?;
                        let count = BufReader::new(&*bytes.clone()).lines().count();
                        let mut file = BufReader::new(&*bytes);
                        for _ in 0..(count as i32 + config.lines) {
                            file.read_line(&mut line)?;
                        }
                        line.clear();
                        for _ in 0..(-config.lines) {
                            let bytes = file.read_line(&mut line)?;
                            if bytes == 0 {
                                break;
                            }
                            print!("{}", line);
                            line.clear();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: i32,
    bytes: Option<i32>,
    quiet: bool,
    verbose: bool,
}

pub fn get_args() -> anyhow::Result<Config> {
    let matches = Command::new("headr")
        .version("0.1.0")
        .author("LovesAsuna <qq625924077@gmail.com>")
        .about("Rust head")
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Input file(s)")
                .action(ArgAction::Append)
                .default_value("-"),
        )
        .arg(
            Arg::new("bytes")
                .short('c')
                .long("bytes")
                .help("print the first K bytes of each file;with the leading '-', print all but the last K bytes of each file")
                .value_parser(value_parser!(i32))
                .conflicts_with("lines")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("lines")
                .short('n')
                .long("lines")
                .help("print the first K lines instead of the first 10;with the leading '-', print all but the last K lines of each file")
                .value_parser(value_parser!(i32))
                .action(ArgAction::Set)
                .default_value("10"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .visible_alias("silent")
                .help("never print headers giving file names")
                .action(ArgAction::SetTrue)
                .conflicts_with("verbose")
                .default_value("false"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .help("always print headers giving file names")
                .action(ArgAction::SetTrue)
                .default_value("true"),
        )
        .get_matches();
    Ok(Config {
        files: matches
            .get_many::<String>("files")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
        quiet: matches.get_flag("quiet"),
        verbose: matches.get_flag("verbose"),
        lines: *matches.get_one::<i32>("lines").unwrap(),
        bytes: matches.get_one("bytes").map(|p| *p),
    })
}

fn open(filename: &str) -> anyhow::Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(filename)?))),
    }
}
