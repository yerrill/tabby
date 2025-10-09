mod codegen;
mod filetype;
mod state;

use codegen::{CodegenOptions, Generation, JsonSchema, Python};
use filetype::{CsvFileType, CsvOptions, Filetype, JsonFileType};

use clap::{Parser, ValueEnum};
use regex::Regex;
use std::{io::Write, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "tabby")]
#[command(version)]
#[command(about = "A data tabulation tool", long_about = None)]
pub struct Cli {
    /// Input file path
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: PathBuf,

    /// Input data format (e.g. json, csv). If blank, infer based on file extension.
    #[arg(short = 'd', long = "input-format", value_enum)]
    input_format: Option<InputData>,

    /// Output file format (e.g. json-schema, python)
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

fn main() {
    let cli = Cli::parse();

    let file = std::fs::read_to_string(&cli.input)
        .expect(format!("Unable to open file: {}", &cli.input.display()).as_str());

    let file_name = cli
        .input
        .file_name()
        .expect("Given input path is not a file")
        .to_str()
        .unwrap();

    let (title, format) = match (cli.input_format, InputData::infer(file_name)) {
        (Some(input), (title, _)) => (title, input),
        (None, (title, Some(input))) => (title, input),
        _ => panic!(
            "Could not determine input formats. Given argument: {:?}, Given file: {:?}",
            cli.input_format, file_name
        ),
    };

    let input_objects = match format {
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

        options.title = Some(title);
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
