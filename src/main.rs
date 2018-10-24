use std::fs::{create_dir, read_dir};
use std::fs;
use std::process::Command;
use std::io::{Write};

fn main() {
    // store build artifacts here
    // will be cleared from time to time
    let mut target_dir = dirs::home_dir().unwrap();
    target_dir.push(".clippy_fuzzy_target_dir");
    let target_dir = target_dir; // unmut
    if !target_dir.display().to_string().ends_with("dir") {
        // sanity
        panic!();
    }
    if !target_dir.is_dir() {
        fs::create_dir(&target_dir).unwrap();
    }
    // get crates to check from here
    let mut crate_archives = dirs::home_dir().unwrap();
    crate_archives.push(".cargo");
    crate_archives.push("registry");
    crate_archives.push("cache");
    crate_archives.push("github.com-1ecc6299db9ec823");

    // run clippy inside here
    let mut work_dir = std::path::PathBuf::from("/tmp");
    work_dir.push("clippy_workdir");
    let work_dir = work_dir; // unmut


    let crates = read_dir(crate_archives).unwrap(); // all the creates from the cargo cache

    let mut crate_archives = Vec::new();
    for cr in crates {
        let unwrapped = cr.unwrap();
        let path = unwrapped.path();
        crate_archives.push(path);
    }

    crate_archives.sort(); // sort paths alphabetically

    // every 100 crates, remove the target dir
    let mut target_dir_counter: u32 = 0;

    let mut crate_counter: u32 = 0;

    let mut bad_crates = Vec::new();

    for archive in crate_archives {
        target_dir_counter += 1;
        crate_counter += 1;
        // create workdir if it does not exist
        if !work_dir.is_dir() {
            fs::create_dir(&work_dir).unwrap();
        }

        // copy the crate from cache into work dir
        let copy_source = &archive;
        let mut copy_dest = work_dir.clone();
        // the filename of the crate
        let archive_name = archive.iter().last().unwrap(); // heapsize-0.4.2.crate
        copy_dest.push(&archive_name);
        // println!("coyping {:?} to {:?}", copy_source, copy_dest);
        // copy the crate to workdir
        fs::copy(&copy_source, &copy_dest).unwrap();

        // extract the .crate
        // @TODO make this pure rust
        let tar = Command::new("tar")
            .arg("-xvzf")
            .arg(&copy_dest)
            .current_dir(&work_dir)
            .output();
        let _ = tar.unwrap();

        // the dir with the extracted sources
        let crate_file_name = archive_name.to_string_lossy().to_string();
        //println!("crate name: {}", crate_file_name);

        let crate_name = crate_file_name.replace(".crate", "");

        let mut crate_dir = work_dir.clone();
        crate_dir.push(&crate_name);
    //    println!("CD {:?}", crate_dir);
        print!("{:>4} Checking {}", crate_counter, crate_name,);
        std::io::stdout().flush().unwrap();
        let clippy = std::process::Command::new("cargo")
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
        let stderr = String::from_utf8_lossy(&clippy.stderr);
        let stdout = String::from_utf8_lossy(&clippy.stdout);
        if stderr.contains("internal compiler error") || stderr.contains("query stack during panic") || stdout.contains("internal compiler error") || stdout.contains("query stack during panic") {
            println!(" ERROR");
            bad_crates.push(crate_name);
        } else {
            println!(" ok");
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
    } // for loop
    println!("Bad crates:");
    bad_crates.into_iter().for_each(|c| println!("{}", c));
}
