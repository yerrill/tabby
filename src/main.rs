mod codegen;
mod filetype;
mod state;

use codegen::{CodegenOptions, Generation, JsonSchema, Python};
use filetype::{CsvFileType, CsvOptions, Filetype, JsonFileType};

use clap::{Parser, ValueEnum};
use regex::Regex;
use std::{
    io::{Read, Write},
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

    /// Output file format (default: json-schema)
    #[arg(short = 'f', long = "output-format", value_enum)]
    output_format: Option<OutputFormat>,

    /// Output file path (defaults to stdout if not set)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Optional delimiter for CSV files
    #[arg(long = "delimiter", value_name = "CHAR")]
    delimiter: Option<char>,
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

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    JsonSchema,
    Python,
}

impl OutputFormat {
    fn resolve(arg: Option<Self>) -> Self {
        match arg {
            Some(f) => f,
            None => Self::JsonSchema,
        }
    }
}

fn process_file_input(
    file_path: &PathBuf,
    input_format: Option<InputData>,
) -> (String, Option<String>, InputData) {
    let file = std::fs::read_to_string(file_path)
        .expect(format!("Unable to open file: {}", &file_path.display()).as_str());

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
    std::io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");

    let file_format = input_format.expect("Cannot infer input format from stdin");

    (buffer, None, file_format)
}

fn main() {
    let cli = Cli::parse();

    let (file, title, file_format) = if let Some(file_path) = &cli.input {
        process_file_input(file_path, cli.input_format)
    } else {
        process_stdin_input(cli.input_format)
    };

    let input_objects = match file_format {
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

        options.title = title;
        options
    };

    let output_code = match OutputFormat::resolve(cli.output_format) {
        OutputFormat::JsonSchema => JsonSchema::generate(input_objects, output_options),
        OutputFormat::Python => Python::generate(input_objects, output_options),
    };

    match cli.output {
        Some(f) => {
            let _ = std::fs::write(f, output_code).expect("Failed to write output file");
        }
        None => {
            let _ = std::io::stdout()
                .write_all(output_code.as_bytes())
                .expect("Failed to write to std out");
        }
    };
}
