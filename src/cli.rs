use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Arguments {
    ///The path to the file to compile
    pub path_to_file: String,
    ///Whether to save the LLVM IR as a file instead of compiling to an executable
    #[arg(short, long)]
    pub save_as_ir: bool,
}
