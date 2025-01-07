use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use glob::Pattern;

use colored::Colorize; // If you want colored output for warnings (optional)
                       // cargo add colored = "2" if you choose to use it

// We'll keep track of a global document index for XML output
// In Python code it was a global; in Rust we can pass &mut i32 or hold in struct.
struct Context {
    global_index: usize,
}

pub fn process_files(
    paths: &[String],
    extensions: &[String],
    include_hidden: bool,
    ignore_gitignore: bool,
    ignore_patterns: &[String],
    output_file: Option<&str>,
    claude_xml: bool,
) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context { global_index: 1 };

    // Decide where to print (stdout or a file).
    let mut writer: Box<dyn Write> = if let Some(outfile) = output_file {
        Box::new(fs::File::create(outfile)?)
    } else {
        Box::new(io::stdout())
    };

    // We’ll gather .gitignore rules from each directory as we go, unless ignore_gitignore is true.
    let mut gitignore_rules: Vec<String> = Vec::new();

    if claude_xml {
        // We open the top-level <documents> once if cxml is requested and only if we have at least one path
        if !paths.is_empty() {
            writeln!(writer, "<documents>")?;
        }
    }

    for p in paths {
        let path = Path::new(p);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", p).into());
        }

        // If we’re not ignoring .gitignore, read from the directory containing `p`
        if !ignore_gitignore {
            if let Some(parent) = path.parent() {
                read_gitignore(parent, &mut gitignore_rules);
            }
        }

        if path.is_file() {
            // Single file
            process_single_file(
                &mut writer,
                &mut ctx,
                path,
                extensions,
                include_hidden,
                ignore_gitignore,
                &mut gitignore_rules,
                ignore_patterns,
                claude_xml,
            )?;
        } else if path.is_dir() {
            // Directory recursion
            // We replicate the Python logic with walkdir
            for entry in WalkDir::new(path) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Warning: Skipping entry due to error: {}", e);
                        continue;
                    }
                };

                let fpath = entry.path();

                // If it’s a directory, decide whether to skip it
                if fpath.is_dir() {
                    // Skip hidden directories if we’re not including hidden
                    if !include_hidden && is_hidden_dir(fpath) {
                        continue;
                    }
                    // Possibly read .gitignore in subdirectories
                    if !ignore_gitignore {
                        read_gitignore(fpath, &mut gitignore_rules);
                        if should_ignore(fpath, &gitignore_rules) {
                            // skip entire directory
                            continue;
                        }
                    }
                    continue;
                } else {
                    // It's a file
                    // Possibly skip if hidden
                    if !include_hidden && is_hidden_file(fpath) {
                        continue;
                    }
                    // If we’re ignoring .gitignore, skip if it’s in the .gitignore
                    if !ignore_gitignore && should_ignore(fpath, &gitignore_rules) {
                        continue;
                    }
                    // Skip if matches ignore_patterns
                    if should_ignore_pattern(fpath, ignore_patterns) {
                        continue;
                    }
                    // Skip if extension doesn’t match
                    if !extensions.is_empty() && !has_extension(fpath, extensions) {
                        continue;
                    }

                    process_single_file(
                        &mut writer,
                        &mut ctx,
                        fpath,
                        extensions,
                        include_hidden,
                        ignore_gitignore,
                        &mut gitignore_rules,
                        ignore_patterns,
                        claude_xml,
                    )?;
                }
            }
        }
    }

    if claude_xml && !paths.is_empty() {
        writeln!(writer, "</documents>")?;
    }

    Ok(())
}

fn process_single_file(
    writer: &mut dyn Write,
    ctx: &mut Context,
    path: &Path,
    _extensions: &[String],
    _include_hidden: bool,
    _ignore_gitignore: bool,
    _gitignore_rules: &mut Vec<String>,
    _ignore_patterns: &[String],
    claude_xml: bool,
) -> Result<(), Box<dyn Error>> {
    // Attempt to read text
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            // If it's UnicodeDecodeError in Python, in Rust it might be invalid UTF-8.
            // We mimic a "Skipping" warning here:
            eprintln!(
                "{}",
                format!("Warning: Skipping file {:?} due to {}", path, e).red()
            );
            return Ok(());
        }
    };

    if claude_xml {
        print_as_xml(writer, ctx, path, &content)?;
    } else {
        print_default(writer, path, &content)?;
    }

    Ok(())
}

fn print_default(writer: &mut dyn Write, path: &Path, content: &str) -> io::Result<()> {
    writeln!(writer, "{}", path.display())?;
    writeln!(writer, "---")?;
    writeln!(writer, "{}", content)?;
    writeln!(writer, "---")?;
    Ok(())
}

fn print_as_xml(
    writer: &mut dyn Write,
    ctx: &mut Context,
    path: &Path,
    content: &str,
) -> io::Result<()> {
    writeln!(writer, "<document index=\"{}\">", ctx.global_index)?;
    writeln!(writer, "<source>{}</source>", path.display())?;
    writeln!(writer, "<document_content>")?;
    writeln!(writer, "{}", content)?;
    writeln!(writer, "</document_content>")?;
    writeln!(writer, "</document>")?;
    ctx.global_index += 1;
    Ok(())
}

// Very simplistic .gitignore reading
fn read_gitignore(dir: &Path, gitignore_rules: &mut Vec<String>) {
    let mut ignore_file = dir.to_path_buf();
    ignore_file.push(".gitignore");

    if ignore_file.is_file() {
        if let Ok(contents) = fs::read_to_string(&ignore_file) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                gitignore_rules.push(line.to_string());
            }
        }
    }
}

// Should we ignore based on .gitignore lines
// .gitignore can contain patterns like *.txt or exact filenames. This is simplistic:
fn should_ignore(path: &Path, gitignore_rules: &[String]) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy(),
        None => return false,
    };

    for rule in gitignore_rules {
        // If rule ends with "/", it's a directory pattern
        if rule.ends_with('/') {
            let dir_rule = &rule[..rule.len() - 1];
            if name == dir_rule {
                // directory match
                return true;
            }
        } else {
            // file match
            // We can use glob matching or direct equality
            if Pattern::new(rule).map_or(false, |pat| pat.matches(&name)) {
                return true;
            }
        }
    }
    false
}

// Check user-specified --ignore patterns
fn should_ignore_pattern(path: &Path, ignore_patterns: &[String]) -> bool {
    let name = match path.file_name() {
        Some(n) => n.to_string_lossy(),
        None => return false,
    };

    for pat in ignore_patterns {
        if Pattern::new(pat).map_or(false, |p| p.matches(&name)) {
            return true;
        }
    }
    false
}

fn has_extension(path: &Path, extensions: &[String]) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        for e in extensions {
            if e.to_lowercase() == ext_lower {
                return true;
            }
        }
    }
    false
}

fn is_hidden_dir(path: &Path) -> bool {
    if let Some(fname) = path.file_name() {
        let name = fname.to_string_lossy();
        name.starts_with('.')
    } else {
        false
    }
}

fn is_hidden_file(path: &Path) -> bool {
    if let Some(fname) = path.file_name() {
        let name = fname.to_string_lossy();
        name.starts_with('.')
    } else {
        false
    }
}

