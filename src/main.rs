use env_logger;
use rustwide::cmd::Command;
use rustwide::{cmd::SandboxBuilder, Crate, Toolchain, WorkspaceBuilder};
use std::error::Error;
use std::fs;
use std::fs::{create_dir, read_dir};
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir;
#[derive(Debug)]
struct Krate {
    name: String,
    version: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logs();

    let sandbox = SandboxBuilder::new()
        .memory_limit(Some(1024 * 1024 * 1024 * 3))
        .enable_networking(false);

    // Create a new workspace in .workspaces/docs-builder
    let workspace =
        WorkspaceBuilder::new(Path::new(".workspaces/crashfinder"), "crashfinder").init()?;

    // Get nightly toolchain
    let toolchain = Toolchain::Dist {
        name: "nightly".into(),
    };
    toolchain.install(&workspace)?;

    // get clippy
    match toolchain.add_component(&workspace, "clippy") {
        Ok(_) => {}
        // if we can't install clippy component, try building from git
        Err(_) => {
            let install_clippy = Command::new(&workspace, toolchain.cargo()).args(&[
                "install",
                "--force",
                "--git",
                "https://github.com/rust-lang/rust-clippy",
                "clippy",
            ]);
            install_clippy.run()?;
        }
    }

/*
    // get rustfmt
    match toolchain.add_component(&workspace, "rustfmt") {
        Ok(_) => {}
        // if we can't install clippy component, try building from git
        Err(_) => {
            let install_rustfmt = Command::new(&workspace, toolchain.cargo()).args(&[
                "install",
                "--git",
                "https://github.com/rust-lang/rustfmt",
                "--force",
            ]);
            install_rustfmt.run()?;
        }
    }
    */

    // install cargo-cache
    let _ = Command::new(&workspace, toolchain.cargo())
        .args(&["install", "cargo-cache"])
        .run();

    let mut build_nr = 0_u32;

    let mut krates =
        std::fs::read_dir("/home/matthias/.cargo/registry/cache/github.com-1ecc6299db9ec823/")
            .unwrap()
            .filter(|f| f.is_ok())
            .map(|f| f.unwrap().path())
            .map(|f| f.file_name().map(|f| f.to_os_string()))
            .filter(|f| f.is_some())
            .map(|f| f.unwrap().into_string())
            .filter(|f| f.is_ok())
            .map(|f| f.unwrap())
            .map(|name| name.to_string())
            .map(|name| name.replace(".crate", ""))
            .map(|name| {
                let split = name.chars().rev().collect::<String>();
                let split = split.split("-").collect::<Vec<_>>();
                let version = split[0].chars().rev().collect::<String>();
                let name = split[1..].join("").chars().rev().collect::<String>();

                Krate {
                    version: version,
                    name: name,
                }
            })
            .collect::<Vec<Krate>>();
    krates.sort_by_key(|k| format!("{}-{}", k.name, k.version));

    for mykrate in &krates {
        build_nr += 1;

        println!(
            "{}  CHECKING: {} {}",
            build_nr, mykrate.name, mykrate.version
        );
        let krate = Crate::crates_io(&mykrate.name, &mykrate.version);
        // dont error if the crate has been canked in the meanstime
        if krate.fetch(&workspace).is_err() {
            continue;
        }

        let mut build_dir = workspace.build_dir("clippy");
        build_dir
            .build(&toolchain, &krate, sandbox.clone())
            .run(|build| {
                let output = build
                    .cargo()
                    .args(&[
                        "clippy",
                        "--all-targets",
                        "--all-features",
                        "-vvvv",
                        "--",
                        "--cap-lints=warn",
                    ])
                    .env("CARGO_INCREMENTAL", "0")
                    .env("RUST_BACKTRACE", "full")
                    .log_output(true)
                    .run_capture();
                match output {
                    Err(_) => {}
                    Ok(output) => {
                        let stdout: String = output.stdout_lines().join("\n");
                        let stderr: String = output.stderr_lines().join("\n");

                        for output in &[&stdout, &stderr] {
                            if output.contains("internal compiler error:")
                                || output.contains("query stack during panic:")
                            {
                                eprintln!("CRASH: {:?}", mykrate);
                                eprintln!("stdout:\n{}", stdout);
                                eprintln!("stderr:\n{}", stderr);
                                std::process::exit(1);
                            }
                        }
                    }
                };
                Ok(())
                // Ok(())
            });
        // for

        // we may need to clean the cargo cache from time to time, do this every 1000 builds:
        if build_nr % 500 == 0 {
            println!("500th build, cleaning cargo cache!");
            let _ = Command::new(&workspace, toolchain.cargo()).args(&["cache", "--autoclean"]);
        }
        // if the target dir gets too big, clear it, only check every 50 crates
        //  let target_dir_path = build_dir.host_target_dir();
        if cumulative_dir_size(&PathBuf::from(".workspaces/crashfinder/builds/")) >= 5_000_000_000
            && build_nr % 50 == 0
        {
            println!("Purging build dirs");
            workspace.purge_all_build_dirs()?
        }
    }
    Ok(())
}

fn setup_logs() {
    let mut env = env_logger::Builder::new();
    env.filter_module("rustwide", log::LevelFilter::Info); // ..Filter::Info
    if let Ok(content) = std::env::var("RUST_LOG") {
        env.parse_filters(&content);
    }
    rustwide::logging::init_with(env.build());
}

fn cumulative_dir_size(dir: &PathBuf) -> u64 {
    if !dir.is_dir() {
        return 0;
    }

    let walkdir_start = dir.display().to_string();

    walkdir::WalkDir::new(&walkdir_start)
        .into_iter()
        .map(|e| e.unwrap().path().to_owned())
        .filter(|f| f.exists()) // avoid broken symlinks
        .map(|f| {
            fs::metadata(f)
                .expect("failed to get metadata of file when getting size of dir")
                .len()
        })
        .sum()
}
