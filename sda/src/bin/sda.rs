use std::io::Write;

use clap::Parser;
use sda::assemble_text;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputType {
    Binary,
    C,
    Hex,
}

/// SProc Assembler
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Filename of the input
    #[clap(value_parser)]
    input: String,

    /// Determine output format
    #[clap(short, long, value_enum, default_value_t = OutputType::Hex)]
    format: OutputType,

    /// Filename of the output, if desired
    #[clap(short, long, value_parser)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let text = match std::fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Unable to read input file {}", args.input);
            std::process::exit(1);
        }
    };

    let result = match assemble_text(&text) {
        Ok(v) => v.into_iter().map(|v| v.get()).collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Unable to assemble {} - {}", args.input, e);
            std::process::exit(1);
        }
    };

    let byte_result = match args.format {
        OutputType::Binary => result
            .iter()
            .flat_map(|v| [(v & 0xF) as u8, ((v & 0xF0) >> 8) as u8])
            .collect::<Vec<_>>(),
        OutputType::Hex => result
            .iter()
            .map(|v| format!("0x{:04X}", v))
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes(),
        OutputType::C => {
            let mut inner: Vec<String> = Vec::new();
            inner.push(format!("size_t data_size = {};", result.len()));
            inner.push("uint16_t data[] = {{".to_string());
            inner.extend(result.iter().enumerate().map(|(i, v)| {
                format!(
                    "    0x{:04X}{}",
                    v,
                    if i + 1 == result.len() { "" } else { "," }
                )
            }));
            inner.push("}};\n".to_string());
            inner.join("\n").into_bytes()
        }
    };

    if let Some(output_file) = args.output {
        match std::fs::write(&output_file, byte_result) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Unable to write to {} - {}", output_file, e);
                std::process::exit(1);
            }
        }
    } else {
        match std::io::stdout().write_all(&byte_result) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Unable to write to stdout - {}", e);
                std::process::exit(1);
            }
        }
    }
}
