#![feature(slicing_syntax)]

use std::io::Command;
use std::io::fs::PathExtensions;
use std::os;
use std::str;

pub fn target_supported() -> bool {
    os::getenv("HOST") == os::getenv("TARGET") ||
        os::getenv("PKG_CONFIG_ALLOW_CROSS") == Some("1".to_string())
}

pub struct Options {
    pub statik: bool,
    pub atleast_version: Option<String>,
}

pub fn find_library(name: &str) -> Result<(), String> {
    find_library_opts(name, &default_options(name))
}

pub fn find_library_opts(name: &str, options: &Options) -> Result<(), String> {
    if os::getenv(format!("{}_NO_PKG_CONFIG", envify(name)).as_slice()).is_some() {
        return Err(format!("pkg-config requested to be aborted for {}", name))
    } else if !target_supported() {
        return Err("pkg-config doesn't handle cross compilation. Use \
                    PKG_CONFIG_ALLOW_CROSS=1 to override".to_string());
    }
    let mut cmd = Command::new("pkg-config");
    if options.statik {
        cmd.arg("--static");
    }
    cmd.arg("--libs")
       .env("PKG_CONFIG_ALLOW_SYSTEM_LIBS", "1")
       .arg(name);
    match options.atleast_version {
        Some(ref v) => { cmd.arg(format!("--atleast-version={}", v)); }
        None => {}
    }
    let out = try!(cmd.output().map_err(|e| {
        format!("failed to run `{}`: {}", cmd, e)
    }));
    let stdout = str::from_utf8(out.output.as_slice()).unwrap();
    let stderr = str::from_utf8(out.error.as_slice()).unwrap();
    if !out.status.success() {
        let mut msg = format!("`{}` did not exit successfully: {}", cmd,
                              out.status);
        if stdout.len() > 0 {
            msg.push_str("\n--- stdout\n");
            msg.push_str(stdout);
        }
        if stderr.len() > 0 {
            msg.push_str("\n--- stderr\n");
            msg.push_str(stderr);
        }
        return Err(msg)
    }

    let mut dirs = Vec::new();
    let parts = stdout.split(' ').filter(|l| !l.is_empty() && l.len() > 2)
                      .map(|arg| (arg[0..2], arg[2..]))
                      .collect::<Vec<_>>();
    for &(flag, val) in parts.iter() {
        if flag == "-L" {
            println!("cargo:rustc-flags=-L native={}", val);
            dirs.push(Path::new(val));
        }
    }
    for &(flag, val) in parts.iter() {
        if flag == "-l" {
            if options.statik && !is_system_lib(val, dirs[]) {
                println!("cargo:rustc-flags=-l {}:static", val);
            } else {
                println!("cargo:rustc-flags=-l {}", val);
            }
        }
    }
    Ok(())
}

pub fn default_options(name: &str) -> Options {
    let name = envify(name);
    let statik = if os::getenv(format!("{}_STATIC", name).as_slice()).is_some() {
        true
    } else if os::getenv(format!("{}_DYNAMIC", name).as_slice()).is_some() {
        false
    } else if os::getenv("PKG_CONFIG_ALL_STATIC").is_some() {
        true
    } else if os::getenv("PKG_CONFIG_ALL_DYNAMIC").is_some() {
        false
    } else {
        false
    };
    Options { statik: statik, atleast_version: None }
}

fn envify(name: &str) -> String {
    name.chars().map(|c| c.to_uppercase()).map(|c| if c == '-' {'_'} else {c})
        .collect()
}

fn is_system_lib(name: &str, dirs: &[Path]) -> bool {
    let libname = format!("lib{}.a", name);
    let root = Path::new("/usr");
    !dirs.iter().any(|d| {
        !root.is_ancestor_of(d) && d.join(&libname).exists()
    })
}
