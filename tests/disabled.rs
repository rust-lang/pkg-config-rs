#![feature(env, std_misc, old_path, old_io)]

extern crate "pkg-config" as pkg_config;

use std::env;
use std::sync::mpsc::channel;
use std::sync::{StaticMutex, MUTEX_INIT};
use std::thread;
use std::old_io::ChanWriter;

static LOCK: StaticMutex = MUTEX_INIT;

fn reset() {
    for (k, _) in env::vars() {
        if k.contains("PKG_CONFIG") || k.contains("DYNAMIC") ||
           k.contains("STATIC") {
            env::remove_var(&k);
        }
    }
    env::remove_var("TARGET");
    env::remove_var("HOST");
    env::set_var("PKG_CONFIG_PATH", &env::current_dir().unwrap().join("tests"));
}

fn find(name: &str) -> Result<pkg_config::Library, String> {
    let (tx, rx) = channel();
    let (tx2, rx2) = channel();
    let name = name.to_string();
    let _t = thread::scoped(move || {
        std::old_io::stdio::set_stdout(Box::new(ChanWriter::new(tx)));
        tx2.send(pkg_config::find_library(&name)).unwrap();
    });
    let ret = rx2.recv().unwrap();
    let mut output = Vec::new();
    for msg in rx.iter() {
        output.extend(msg.into_iter());
    }
    ret
}

#[test]
fn cross_disabled() {
    let _g = LOCK.lock();
    reset();
    env::set_var("TARGET", "foo");
    env::set_var("HOST", "bar");
    find("foo").unwrap_err();
}

#[test]
fn cross_enabled() {
    let _g = LOCK.lock();
    reset();
    env::set_var("TARGET", "foo");
    env::set_var("HOST", "bar");
    env::set_var("PKG_CONFIG_ALLOW_CROSS", "1");
    find("foo").unwrap();
}

#[test]
fn package_disabled() {
    let _g = LOCK.lock();
    reset();
    env::set_var("FOO_NO_PKG_CONFIG", "1");
    find("foo").unwrap_err();
}

#[test]
fn output_ok() {
    let _g = LOCK.lock();
    reset();
    let lib = find("foo").unwrap();
    assert!(lib.libs.contains(&"gcc".to_string()));
    assert!(lib.libs.contains(&"coregrind-amd64-linux".to_string()));
    assert!(lib.link_paths.contains(&Path::new("/usr/lib/valgrind")));
}

#[test]
fn framework() {
    let _g = LOCK.lock();
    reset();
    let lib = find("framework").unwrap();
    assert!(lib.frameworks.contains(&"foo".to_string()));
    assert!(lib.framework_paths.contains(&Path::new("/usr/lib")));
}
