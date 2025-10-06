mod codegen;
mod filetype;
mod state;

use codegen::{Generation, Python};
use filetype::{CsvFileType, CsvOptions, Filetype, JsonFileType};

use clap::{Parser, ValueEnum};
use std::{io::Write, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "tabby")]
#[command(version)]
#[command(about = "A data tabulation tool", long_about = None)]
pub struct Cli {
    /// Input file path
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: PathBuf,

    /// Input format (e.g. json, csv)
    #[arg(long = "from", value_enum)]
    input_format: InputFormat,

    /// Output format (e.g. python)
    #[arg(long = "to", value_enum)]
    output_format: OutputFormat,

    /// Output file path (defaults to stdout if not set)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Optional delimiter for CSV files
    #[arg(long = "delimiter", value_name = "CHAR")]
    delimiter: Option<char>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum InputFormat {
    Json,
    Csv,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Python,
}

fn main() {
    let cli = Cli::parse();

    let file = std::fs::read_to_string(&cli.input)
        .expect(format!("Unable to open file: {}", &cli.input.display()).as_str());

    let input_objects = match cli.input_format {
        InputFormat::Csv => {
            let mut csv_options = CsvOptions::new();

            if let Some(delimiter) = cli.delimiter {
                csv_options.delimiter = delimiter;
            }

            CsvFileType::new(file.as_str(), csv_options)
                .expect("Unable to parse csv")
                .to_object()
        }
        InputFormat::Json => JsonFileType::new(file.as_str())
            .expect("Unable to parse json")
            .to_object(),
    };

    let output_code = match cli.output_format {
        OutputFormat::Python => Python::generate(input_objects),
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
