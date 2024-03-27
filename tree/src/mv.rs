//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//
// TODO:
// - preserve file attributes during copy
// - do not repeatedly stat(2) the target, for each source
//

extern crate clap;
extern crate libc;
extern crate plib;

use clap::Parser;
use gettextrs::{bind_textdomain_codeset, gettext, textdomain};
use plib::PROJECT_NAME;
use std::path::Path;
use std::{fs, io};

/// mv - move files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Do not prompt for confirmation if the destination path exists
    #[arg(short, long)]
    force: bool,

    /// Prompt for confirmation if the destination path exists.
    #[arg(short, long)]
    interactive: bool,

    /// Source(s) and target of move(s)
    files: Vec<String>,
}

struct Config {
    force: bool,
    interactive: bool,
    is_terminal: bool,
}

impl Config {
    fn new(args: &Args) -> Self {
        Config {
            force: args.force,
            interactive: args.interactive,
            is_terminal: unsafe { libc::isatty(libc::STDIN_FILENO) != 0 },
        }
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn prompt_user(prompt: &str) -> bool {
    eprint!("{} ", prompt);
    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    response.to_lowercase().starts_with('y')
}

fn move_file(cfg: &Config, source: &str, target: &str) -> io::Result<()> {
    // 1. If the destination path exists, conditionally prompt user
    let target_md = match fs::metadata(target) {
        Ok(md) => Some(md),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                None
            } else {
                eprintln!("{}: {}", target, e);
                return Err(e.into());
            }
        }
    };
    let target_exists = target_md.is_some();
    let target_is_dir = match target_md {
        Some(md) => md.is_dir(),
        None => false,
    };
    if target_exists && !cfg.force && (cfg.is_terminal || cfg.interactive) {
        let is_affirm = prompt_user(&format!("{}: {}", target, gettext("overwrite?")));
        if !is_affirm {
            return Ok(());
        }
    }

    // 2. source and target are same dirent:  we assume rename handles this case

    // 3. call rename(2) to move source to target
    match fs::rename(source, target) {
        Ok(_) => return Ok(()),
        Err(e) => {
            // use ErrorKind::CrossesDevices in the future, when it is stable
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap();
            if errno != libc::EXDEV {
                eprintln!("{}: {}", source, e);
                return Err(e.into());
            }
        }
    }

    // Fall through: source and target are on different filesystems; must copy.

    // 4. handle source/target dir mismatch
    let source_md = match fs::metadata(source) {
        Ok(md) => Some(md),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                None
            } else {
                eprintln!("{}: {}", source, e);
                return Err(e.into());
            }
        }
    };
    let source_is_dir = match source_md {
        Some(md) => md.is_dir(),
        None => false,
    };

    if (target_exists && target_is_dir && !source_is_dir)
        || (target_exists && !target_is_dir && source_is_dir)
    {
        eprintln!(
            "{}: {}",
            target,
            gettext("cannot overwrite directory with non-directory")
        );
        return Ok(());
    }

    // 5. remove destination path
    if target_exists {
        if target_is_dir {
            fs::remove_dir_all(target)?;
        } else {
            fs::remove_file(target)?;
        }
    }

    // 6. copy source file hierarchy to target
    if source_is_dir {
        copy_dir_all(source, target)?;
    } else {
        fs::copy(source, target)?;
    }

    // 7. Remove source file hierarchy
    assert!(source_is_dir);
    fs::remove_dir_all(source)?;

    Ok(())
}

fn move_files(cfg: &Config, sources: &[String], target: &str) -> io::Result<()> {
    // loop through sources, moving each to target
    for source in sources {
        move_file(cfg, source, target)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let args = Args::parse();

    // Initialize translation system
    textdomain(PROJECT_NAME)?;
    bind_textdomain_codeset(PROJECT_NAME, "UTF-8")?;

    if args.files.len() < 2 {
        eprintln!("{}", gettext("Must supply a source and target for move"));
        std::process::exit(1);
    }

    // split sources and target
    let sources = &args.files[0..args.files.len() - 1];
    let target = &args.files[args.files.len() - 1];

    // choose mode based on whether target is a directory
    let dir_exists = {
        match fs::metadata(target) {
            Ok(md) => md.is_dir(),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    false
                } else {
                    eprintln!("{}: {}", target, e);
                    std::process::exit(1);
                }
            }
        }
    };

    let cfg = Config::new(&args);
    if dir_exists {
        let _ = move_files(&cfg, sources, target);
    } else {
        let _ = move_file(&cfg, &sources[0], target);
    }

    Ok(())
}
