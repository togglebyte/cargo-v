use std::error::Error;
use std::io;
use std::process::Command;
use std::{env, fs};

use cargo_v;

// This function will bail with a given exit code of 1.
// If it is acceptable to build this with Rust 1.61+ then
// we can replace this entire function and have `main` return `std::process::ExitCode`.
fn error_exit(msg: impl AsRef<str>) -> ! {
  eprintln!("ERROR: {}", msg.as_ref());
  std::process::exit(1);
}

fn main() {
  let mut args = env::args().skip(2);

  let new_version_input = match args.next() {
    Some(version_string) => version_string,
    None => error_exit("you must pass the version (patch, minor, major)"),
  };

  let cargo_toml = match fs::read_to_string("./Cargo.toml") {
    Ok(toml) => toml,
    Err(e) => error_exit(format!("failed to read Cargo.toml: {e}")),
  };

  let new_version = match new_version_input.as_str() {
    "patch" => cargo_v::VersionLabel::Patch,
    "minor" => cargo_v::VersionLabel::Minor,
    "major" => cargo_v::VersionLabel::Major,
    v => cargo_v::VersionLabel::NumericVersion(String::from(v)),
  };

  let cargo_toml_updated = match cargo_v::update_version(&cargo_toml, &new_version) {
      Ok(updated) => updated,
      Err(e) => error_exit(format!("failed to update version: {e}")),
  };

  let new_version = cargo_v::get_version(&cargo_toml_updated);
  let new_version = cargo_v::tuple_version_to_string(&new_version);

  if let Err(e) = update_cargo_toml(&cargo_toml_updated) {
    error_exit(format!("failed to write Cargo.toml: {e}"));
  }

  run_build();

  if let Err(e) = git_add() {
    error_exit(format!("failed to call `git add`: {e}"));
  }

  if let Err(e) = git_commit(&new_version) {
    error_exit(format!("failed to call `git commit`: {e}"));
  }

  if let Err(e) = git_tag(&new_version) {
    error_exit(format!("failed to call `git tag`: {e}"));
  }
}

fn update_cargo_toml(new_cargo_toml: &str) -> Result<(), Box<dyn Error>> {
  fs::write("./Cargo.toml", new_cargo_toml)?;
  Ok(())
}

// Build the project to update the `Cargo.lock` file
fn run_build() {
  Command::new("cargo")
    .args(["build", "--release"])
    .output()
    .expect("Failed to build project.");
}

// TODO: make this optional in case there is a
// broken Cargo.toml as a result of the user
// being in the middle of an edit
fn git_add() -> io::Result<()> {
  Command::new("git")
    .args(["add", "Cargo.toml", "Cargo.lock"])
    .output()?;
  Ok(())
}

fn git_commit(version: &str) -> io::Result<()> {
  let version = &format!("v{}", version);
  Command::new("git")
    .args(["commit", "-m", version])
    .output()?;
  Ok(())
}

fn git_tag(version: &str) -> io::Result<()> {
  let version = &format!("v{}", version);
  Command::new("git")
    .args(["tag", "-a", version, "-m", version])
    .output()?;
  Ok(())
}
