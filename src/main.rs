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
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Input data format (default: json)
    #[arg(short = 'f', long = "input-format", value_enum)]
    input_format: Option<InputData>,

    /// Output file path (default: stdout)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Optional specify title
    #[arg(short = 't', long = "title", value_name = "STRING")]
    title: Option<String>,

    /// Optional delimiter for CSV files
    #[arg(long = "delimiter", value_name = "CHAR")]
    delimiter: Option<char>,

    /// Disable use of `enum` keyword
    #[arg(long = "no-enum", action = ArgAction::SetFalse, default_value_t = true)]
    no_enum: bool,

    /// Disable use of `const` keyword
    #[arg(long = "no-const", action = ArgAction::SetFalse, default_value_t = true)]
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

fn resolve_title(cli: &Cli) -> Option<String> {
    if let Some(title) = &cli.title {
        Some(title.to_owned())
    } else if let Some(file_path) = &cli.input {
        let (title, _) = InputData::infer(
            file_path
                .file_name()
                .expect("Given input path is not a file")
                .to_str()
                .unwrap(),
        );
        Some(title)
    } else {
        None
    }
}

fn read_data(cli: &Cli) -> String {
    if let Some(file_path) = &cli.input {
        std::fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Unable to open file: {}", &file_path.display()))
    } else {
        let mut buffer = String::new();
        let mut stdin = std::io::stdin();

        if stdin.is_terminal() {
            panic!("No data input provided. Run `tabby --help` for usage.");
        }

        stdin
            .read_to_string(&mut buffer)
            .expect("Unable to read from stdin");

        buffer
    }
}

fn resolve_format(cli: &Cli) -> InputData {
    let resolved = if let Some(input_format) = &cli.input_format {
        Some(*input_format)
    } else if let Some(file_path) = &cli.input {
        let (_, input_format) = InputData::infer(
            file_path
                .file_name()
                .expect("Given input path is not a file")
                .to_str()
                .unwrap(),
        );
        input_format
    } else {
        None
    };

    resolved.unwrap_or(InputData::Json)
}

fn main() {
    let cli = Cli::parse();

    let title = resolve_title(&cli);
    let file = read_data(&cli);
    let file_format = resolve_format(&cli);

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

        options.title = title;
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
