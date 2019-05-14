use bindgen;
use cc;
use failure;
use git2;
use std::env;
use std::fs;
use std::path::PathBuf;

fn major_version() -> u8 {
    env::var("CARGO_PKG_VERSION_MAJOR")
        .unwrap()
        .parse()
        .unwrap()
}

fn minor_version() -> u8 {
    env::var("CARGO_PKG_VERSION_MAJOR")
        .unwrap()
        .parse()
        .unwrap()
}

fn version() -> String {
    format!("{}.{}", major_version(), minor_version())
}

fn output() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn main() -> Result<(), failure::Error> {
    let url = "https://github.com/MusicPlayerDaemon/libmpdclient";
    let repo = git2::Repository::clone(url, &output())?;
    let mut build = cc::Build::new();
    let mut bindgen = bindgen::Builder::default();

    if !cfg!(feature = "latest") {
        repo.set_head(&format!("refs/tags/v{}", version()))?;
    }

    fs::write(output().join("config.h"), format!(r#"
        #pragma once
        #define DEFAULT_HOST "localhost"
        #define DEFAULT_PORT 6600
        #define DEFAULT_SOCKET "/var/run/mpd/socket"
        #define ENABLE_TCP
        #define HAVE_GETADDRINFO
        #define HAVE_STRNDUP
        #define PACKAGE "libmpdclient"
        #define VERSION "{}"
        "#, version()))?;
    fs::write(output().join("include/mpd/version.h"), format!(r#"
        #ifndef MPD_VERSION_H
        #define MPD_VERSION_H

        #define LIBMPDCLIENT_MAJOR_VERSION {}
        #define LIBMPDCLIENT_MINOR_VERSION {}
        #define LIBMPDCLIENT_PATCH_VERSION 0

        #define LIBMPDCLIENT_CHECK_VERSION(major, minor, patch) \
                ((major) < LIBMPDCLIENT_MAJOR_VERSION || \
                ((major) == LIBMPDCLIENT_MAJOR_VERSION && \
                ((minor) < LIBMPDCLIENT_MINOR_VERSION || \
                ((minor) == LIBMPDCLIENT_MINOR_VERSION && \
                    (patch) <= LIBMPDCLIENT_PATCH_VERSION))))
        #endif
        "#, major_version(), minor_version()))?;

    build.include(output().join("include"))
         .include(output())
         .include(output().join("src"))
         .shared_flag(false);

    for entry in fs::read_dir(output().join("src"))? {
        let entry = entry?;

        if let Some(extension) = entry.path().extension() {
            if extension == "c" && entry.file_name() != "example.c" {
                build.file(entry.path());
            }
        }
    }

    for entry in fs::read_dir(output().join("include/mpd"))? {
        let entry = entry?;

        if let Some(extension) = entry.path().extension() {
            if extension == "h" {
                bindgen = bindgen.header(entry.path().to_str().unwrap());
            }
        }
    }

    build.compile("mpdclient");

    let bindings = bindgen.generate().expect("Could not generate bindings.");

    bindings.write_to_file(output().join("libmpdclient.rs"))?;
    Ok(())
}