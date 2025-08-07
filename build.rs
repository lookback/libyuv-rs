use std::{env, fs, path::Path};

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-env-changed=./src");

    let output = env::var("OUT_DIR").unwrap();
    let commit_hash = "b7d97d5f3f8f897b88872b6935e4c996b955bc1f";
    let source_dir = Path::new(&output).join("libyuv-source");

    // Clone the repository to the specific commit
    if source_dir.exists() {
        fs::remove_dir_all(&source_dir)?;
    }

    println!("cargo:warning=Cloning libyuv at commit {}", commit_hash);
    let repo = git2::Repository::clone("https://github.com/lemenkov/libyuv", &source_dir)?;
    let oid = git2::Oid::from_str(commit_hash)?;
    let commit = repo.find_commit(oid)?;
    repo.checkout_tree(commit.as_object(), None)?;
    repo.set_head_detached(oid)?;

    // Build with cmake
    let dst = cmake::Config::new(&source_dir).define("CMAKE_BUILD_TYPE", "Release")
                                             .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=yuv");

    Ok(())
}
