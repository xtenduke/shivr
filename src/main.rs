use std::{env, thread};
mod files;
use files::{filter_files_to_packages, get_packages};
mod git;
use git::get_changed_files;
mod config;
use config::load_config;
use argh::{FromArgs};

extern crate run_shell;
use run_shell::*;

fn get_env_dir() -> String {
    let root_dir_path = match env::current_dir() {
        Ok(root_dir) => root_dir,
        Err(e) => panic!("Can't get current dir {}", e),
    };

    return String::from(root_dir_path.to_str().unwrap());
}

/// Arguments
#[derive(FromArgs)]
struct Args {
    /// if shiv should run the command on all packages, or just those changed against main
    #[argh(switch)]
    detect_changes: bool,

    /// main branch name, default "main"
    #[argh(option, default = "String::from(\"main\")")]
    main_branch: String,

    /// root dir to run in
    #[argh(option, default = "get_env_dir()")]
    root_dir: String,

    /// package directory, default "packages"
    #[argh(option, default = "String::from(\"packages\")")]
    package_dir: String,

    /// command to run on packages
    #[argh(option)]
    command: String,
}


fn main() {
    let args: Args = argh::from_env();

    run(
        args.detect_changes,
        &args.package_dir,
        &args.main_branch,
        &args.root_dir,
        &args.command,
    );
}

fn run(
    detect_changes: bool,
    package_dir: &String,
    main_branch: &String,
    root_dir: &String,
    command: &String,
) {
    println!(
        "Running--- detect_changes: {}, root_dir: {}, command: {}",
        &detect_changes, &root_dir, &command
    );

    let mut packages = get_packages(&String::from(root_dir), package_dir);

    if detect_changes {
        println!("Detecting changes");
        let changed_files = get_changed_files(root_dir, main_branch);
        packages = filter_files_to_packages(packages, changed_files);
    }

    let mut handles = Vec::<std::thread::JoinHandle<()>>::new();
    for package in packages {
        let command = command.clone();
        let root_dir = root_dir.clone();
        println!("Detected package change: {}", package);
        let handle = thread::spawn(move || {
            run_command(command, root_dir, package);
        });
        handles.push(handle);
    }

    for handle in handles.into_iter() {
        handle.join().unwrap();
    }
}

fn run_command(command: String, root_dir: String, package: String) {
    // look for shiv.json file in package
    // process fork here
    let mut path = root_dir.clone();
    path.push_str("/");
    path.push_str(&package);

    assert!(env::set_current_dir(&path).is_ok());

    let mut config_path = path.clone();
    config_path.push_str("/");
    config_path.push_str("shiv.json");

    let config = load_config(&config_path);

    let mut package_command: Option<String> = None;
    for script in config.scripts {
        if command == script.name {
            package_command = Some(script.run);
        }
    }

    if package_command.is_none() {
        println!("Found no entries for {} in {}", command, package);
        // process::exit(0x0100);
    } else {
        println!("Running {} in {}", command, package);
        // change cwd
        println!("Running {} in {}", package_command.clone().unwrap(), &path);
        //Command::new(package_command.unwrap()).spawn().expect("command failed");
        cmd!(&package_command.unwrap(), &path).run().unwrap();
        println!("Successfully ran {}", &path);
    }

}