extern crate clap;
extern crate serde_yaml;
#[macro_use]
extern crate log;
extern crate simple_logging;
extern crate swagger_utils;
extern crate ycli;

use std::fs;
use std::io;
use std::str::FromStr;

use clap::{App, Arg};
use swagger_utils::path::Paths;
use swagger_utils::swagger::Swagger;

use ycli::cli::CLI;

const SCHEMA_DIR: &str = "/usr/share/ycli/schema/";

fn read_paths(schema_dir: &str) -> io::Result<Paths> {
    let mut paths = Paths::new();
    for entry in fs::read_dir(schema_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        debug!("Found schema file {:?}", path);
        let file = fs::File::open(path).unwrap();

        let swagger: Swagger = match serde_yaml::from_reader(&file) {
            Ok(swagger) => swagger,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        if swagger.paths.is_none() {
            continue;
        }
        paths.append(&mut swagger.paths.unwrap());
    }
    Ok(paths)
}

fn main() {
    let matches = App::new("ycli")
        .version("0.1.0")
        .author("qiuchengxuan <qiuchengxuan@gmail.com>")
        .about("Yaml based CLI")
        .arg(
            Arg::with_name("schema-dir")
                .short("d")
                .long("schema-dir")
                .takes_value(true)
                .help("Directory that contains swagger schema files"),
        )
        .arg(
            Arg::with_name("config-file")
                .short("f")
                .long("config-file")
                .takes_value(true)
                .help("Directory that contains swagger schema files"),
        )
        .arg(
            Arg::with_name("log-level")
                .long("log-level")
                .takes_value(true)
                .help("Specify log level"),
        )
        .get_matches();
    let schema_dir = matches.value_of("schema-dir").unwrap_or(SCHEMA_DIR);
    let config_file = matches.value_of("config-file").unwrap_or("config.yaml");
    let log_level = matches.value_of("log-level").unwrap_or("INFO");
    let _ = config_file;

    let log_level = match log::LevelFilter::from_str(&log_level.to_uppercase()) {
        Ok(level) => level,
        Err(e) => return println!("{}", e),
    };
    simple_logging::log_to_stderr(log_level);

    match ctrlc::set_handler(move || {}) {
        Ok(()) => {}
        Err(e) => panic!(e),
    };

    let paths = match read_paths(schema_dir) {
        Ok(paths) => paths,
        Err(e) => return println!("{}", e),
    };

    let mut cli = CLI::new(&paths);
    let result = cli.run();
    if result.is_err() {
        println!("{:?}", result.err())
    }
}
