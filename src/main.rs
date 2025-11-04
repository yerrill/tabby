mod codegen;
mod filetype;
mod state;

use codegen::{CodegenOptions, Generation, JsonSchema};
use filetype::{CsvFileType, CsvOptions, Filetype, JsonFileType};
use state::Subschema;

use clap::{ArgAction, Parser, ValueEnum};
use regex::Regex;
use std::{
    io::{IsTerminal, Read, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(name = "tabby")]
#[command(version)]
#[command(about = "A data tabulation tool", long_about = None)]
pub struct Cli {
    /// Input file path (default: stdin)
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: Option<PathBuf>,

    /// Input data format (default: json, required if reading from stdin)
    #[arg(short = 'd', long = "input-format", value_enum)]
    input_format: Option<InputData>,

    /// Output file path (defaults to stdout if not set)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Optional specify title
    #[arg(long = "title", value_name = "STRING")]
    title: Option<String>,

    /// Optional delimiter for CSV files
    #[arg(long = "delimiter", value_name = "CHAR")]
    delimiter: Option<char>,

    /// Disable use of `enum` keyword
    #[arg(short, long, action = ArgAction::SetFalse, default_value_t = true)]
    no_enum: bool,

    /// Disable use of `const` keyword
    #[arg(short, long, action = ArgAction::SetFalse, default_value_t = true)]
    no_const: bool,

    /// Optional enum percent, field must have less than given percent unique values to be counted as an enum
    #[arg(long = "enum-percent", value_name = "0-100")]
    enum_threshold: Option<u8>,

    /// Optional enum maximum, max number of unique values a field can have to be considered an enum
    #[arg(long = "enum-max", value_name = "0-255")]
    enum_maximum: Option<u8>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum InputData {
    Json,
    Csv,
}

impl InputData {
    fn infer(file_name: &str) -> (String, Option<Self>) {
        let file = Regex::new(r"(?<name>.*)(?:\.(?<ext>.*))$").unwrap();

        let Some(caps) = file.captures(file_name) else {
            return (file_name.to_owned(), None);
        };

        let title = caps["name"].to_owned();

        let format = match &caps["ext"] {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            _ => None,
        };

        (title, format)
    }
}

fn process_file_input(
    file_path: &PathBuf,
    input_format: Option<InputData>,
) -> (String, Option<String>, InputData) {
    let file = std::fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Unable to open file: {}", &file_path.display()));

    let file_name = file_path
        .file_name()
        .expect("Given input path is not a file")
        .to_str()
        .unwrap();

    let (title, file_format) = match (input_format, InputData::infer(file_name)) {
        (Some(input), (title, _)) => (title, input),
        (None, (title, Some(input))) => (title, input),
        _ => panic!(
            "Could not determine input formats. Given argument: {:?}, Given file: {:?}",
            input_format, file_name
        ),
    };

    (file, Some(title), file_format)
}

fn process_stdin_input(input_format: Option<InputData>) -> (String, Option<String>, InputData) {
    let mut buffer = String::new();
    let mut stdin = std::io::stdin();

    if stdin.is_terminal() {
        panic!("No data input provided. Run `tabby --help` for usage.");
    }

    stdin
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");

    let file_format = input_format.expect("Cannot infer input format from stdin");

    (buffer, None, file_format)
}

fn main() {
    let cli = Cli::parse();

    let (file, title, file_format) = match &cli.input {
        Some(file_path) => process_file_input(file_path, cli.input_format),
        None => process_stdin_input(cli.input_format),
    };

    let input_data = match file_format {
        InputData::Csv => {
            let mut csv_options = CsvOptions::new();

            if let Some(delimiter) = cli.delimiter {
                csv_options.delimiter = delimiter;
            }

            CsvFileType::new(file.as_str(), csv_options)
                .expect("Unable to parse csv")
                .to_object()
        }
        InputData::Json => JsonFileType::new(file.as_str())
            .expect("Unable to parse json")
            .to_object(),
    };

    let output_options = {
        let mut options = CodegenOptions::new();

        options.title = match cli.title {
            Some(t) => Some(t),
            None => title,
        };

        options.use_enum = cli.no_enum;
        options.use_const = cli.no_const;

        if let Some(n) = cli.enum_threshold {
            options.enum_threshold = n;
        };

        options.enum_maximum = cli.enum_maximum;

        options
    };

    let output_code = JsonSchema::generate(Subschema::from_data(input_data), output_options);

    match cli.output {
        Some(f) => {
            std::fs::write(f, output_code).expect("Failed to write output file");
        }
        None => {
            std::io::stdout()
                .write_all(output_code.as_bytes())
                .expect("Failed to write to std out");
        }
    };
}
