use dirs::home_dir;
use std::fs::{copy, create_dir, read_dir};
use std::process::Command;

fn main() {
    // store build artifacts here
    let mut target_dir = dirs::home_dir().unwrap();
    target_dir.push(".clippy_fuzzy_target_dir");
    if !target_dir.display().to_string().ends_with("dir") {
        // sanity
        panic!();
    }
    if !target_dir.is_dir() {
        create_dir(&target_dir).unwrap();
    }
    // get crates from here
    let mut crate_archives = dirs::home_dir().unwrap();
    crate_archives.push(".cargo");
    crate_archives.push("registry");
    crate_archives.push("cache");
    crate_archives.push("github.com-1ecc6299db9ec823");

    // run clippy inside here
    let mut work_dir = std::path::PathBuf::from("/tmp");
    work_dir.push("clippy_workdir");
    let work_dir = work_dir; // unmut


    let crates = read_dir(crate_archives).unwrap();

    let mut crate_archives = Vec::new();
    for c in crates {
        let u = c.unwrap();
        let p = u.path();
        crate_archives.push(p);
    }

    crate_archives.sort();

    // every 100 crates, remove the target dir
    let mut target_dir_counter = 0;

    for entry in crate_archives {
        target_dir_counter += 1;
        // create workdir if it does not exist
        if !work_dir.is_dir() {
            create_dir(&work_dir).unwrap();
        }

        let archive = entry;

        let copy_from = archive.clone();
        let mut copy_to = work_dir.clone();
        let archive_name = archive.iter().last().unwrap(); // heapsize-0.4.2.crate
        copy_to.push(&archive_name);
        //println!("coyping {:?} to {:?}", copy_from, copy_to);
        // copy crate to workdir
        copy(&copy_from, &copy_to).unwrap();

        // extract the .crate
        // @TODO make this pure rust
        let tar = std::process::Command::new("tar")
            .arg("-xvzf")
            .arg(&copy_to)
            .current_dir(&work_dir)
            .output();
        let _ = tar.unwrap();
        let mut crate_dir = archive_name.to_string_lossy().to_string();
        // the dir with the extracted sources
        let crate_name = crate_dir.replace(".crate", "");
        let name = crate_name.clone();
        let mut crate_dir = work_dir.clone();
        crate_dir.push(crate_name);
    //    println!("CD {:?}", crate_dir);
        println!("{}", name);
        let CLIPPY = std::process::Command::new("cargo")
            .arg("clippy")

            .arg("--all-targets")
            .arg("--all-features")
            .arg("-vvvv")
            .args(&["--"
            ,"--cap-lints", "warn"
            ,"-Wclippy::internal"
            ,"-Wclippy::pedantic"
            ,"-Wclippy::nursery"
            ,"-Wabsolute-paths-not-starting-with-crate"
            ,"-Wbare-trait-objects"
            ,"-Wbox-pointers"
            ,"-Welided-lifetimes-in-paths"
            ,"-Wellipsis-inclusive-range-patterns"
            ,"-Wkeyword-idents"
            ,"-Wmacro-use-extern-crate"
            ,"-Wmissing-copy-implementations"
            ,"-Wmissing-debug-implementations"
            ,"-Wmissing-docs"
            ,"-Wquestion-mark-macro-sep"
            ,"-Wsingle-use-lifetimes"
            ,"-Wtrivial-casts"
            ,"-Wtrivial-numeric-casts"
            ,"-Wunreachable-pub"
            ,"-Wunsafe-code"
            ,"-Wunstable-features"
            ,"-Wunused-extern-crates"
            ,"-Wunused-import-braces"
            ,"-Wunused-labels"
            ,"-Wunused-lifetimes"
            ,"-Wunused-qualifications"
            ,"-Wunused-results"
            ,"-Wvariant-size-differences"])
            .current_dir(&crate_dir)
            .env("CARGO_INCREMENTAL", "0")
            .env("RUST_BACKTRACE", "full")
            .env("CARGO_TARGET_DIR", &target_dir)
            .output().unwrap();
        //println!("crate_dir: {}, cargo_target_dir {}", crate_dir, target_dir.display());
        //println!("output: {:?}", CLIPPY);
        let stderr = String::from_utf8_lossy(&CLIPPY.stderr);
        let stdout = String::from_utf8_lossy(&CLIPPY.stdout);
        if stderr.contains("internal compiler error") || stderr.contains("query stack during panic") || stdout.contains("internal compiler error") || stdout.contains("query stack during panic") {
            println!("{} ERROR ERROR ERROR!", name);
        } else {
            println!("{} ok", name);
        }

        //println!("{}", stderr);
        // remove the workdir
        if work_dir.is_dir() {
            std::fs::remove_dir_all(&work_dir).unwrap();
        }
        if target_dir_counter >= 100 {
            // clear target dir
            println!("CLEARING TARGET DIR");
            std::fs::remove_dir_all(&target_dir).unwrap();
            create_dir(&target_dir).unwrap();
            target_dir_counter = 0;
        }
        //break;
    }
}
