use std::io::Command;
use std::os;
use std::str;

pub struct Options {
    pub statik: bool,
}

pub fn find_library(name: &str) -> Result<(), String> {
    find_library_opts(name, &default_options(name))
}

pub fn find_library_opts(name: &str, options: &Options) -> Result<(), String> {
    if os::getenv(format!("{}_NO_PKG_CONFIG", envify(name)).as_slice()).is_some() {
        return Err(format!("pkg-config requested to be aborted for {}", name))
    }
    let mut cmd = Command::new("pkg-config");
    if options.statik {
        cmd.arg("--static");
    }
    cmd.arg("--libs")
       .env("PKG_CONFIG_ALLOW_SYSTEM_LIBS", "1")
       .arg(name);
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

    for arg in stdout.split(' ').filter(|l| !l.is_empty() && l.len() > 2) {
        let val = arg.slice_from(2);
        if arg.starts_with("-l") {
            if options.statik {
                println!("cargo:rustc-flags=-l {}:static", val);
            } else {
                println!("cargo:rustc-flags=-l {}", val);
            }
        } else if arg.starts_with("-L") {
            println!("cargo:rustc-flags=-L {}", val);
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
    Options { statik: statik }
}

fn envify(name: &str) -> String {
    name.chars().map(|c| c.to_uppercase()).map(|c| if c == '-' {'_'} else {c})
        .collect()
}
