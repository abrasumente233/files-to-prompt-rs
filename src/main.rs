use clap::{Arg, ArgAction, Command};
use std::error::Error;

mod process;
use crate::process::process_files;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("files-to-prompt")
        .version("0.1.0")
        .about("Concatenate a directory of files into a single prompt for LLMs")
        .arg(
            Arg::new("paths")
                .help("Paths to files or directories")
                .required(true)
                .num_args(1..)
        )
        .arg(
            Arg::new("extension")
                .short('e')
                .long("extension")
                .help("Only include files with these extensions")
                .action(ArgAction::Append)
                .value_name("EXT")
        )
        .arg(
            Arg::new("include_hidden")
                .long("include-hidden")
                .help("Include files and folders starting with .")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ignore_gitignore")
                .long("ignore-gitignore")
                .help("Ignore .gitignore files and include all files")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ignore_patterns")
                .long("ignore")
                .help("Ignore files matching these patterns")
                .action(ArgAction::Append)
                .value_name("PATTERN")
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output")
                .help("Output to a file instead of stdout")
                .value_name("FILE")
        )
        .arg(
            Arg::new("cxml")
                .short('c')
                .long("cxml")
                .help("Output in XML-ish format suitable for Claude’s long context window")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let paths: Vec<String> = matches
        .get_many::<String>("paths")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let extensions: Vec<String> = matches
        .get_many::<String>("extension")
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    let include_hidden = matches.get_flag("include_hidden");
    let ignore_gitignore = matches.get_flag("ignore_gitignore");
    let ignore_patterns: Vec<String> = matches
        .get_many::<String>("ignore_patterns")
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    let output_file = matches.get_one::<String>("output_file").map(ToString::to_string);
    let claude_xml = matches.get_flag("cxml");

    process_files(
        &paths,
        &extensions,
        include_hidden,
        ignore_gitignore,
        &ignore_patterns,
        output_file.as_deref(),
        claude_xml,
    )?;

    Ok(())
}

