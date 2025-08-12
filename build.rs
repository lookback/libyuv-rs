use std::{env, fs, path::Path};

fn main() -> anyhow::Result<()> {
    println!("cargo::rerun-if-env-changed=./src");

    let output = env::var("OUT_DIR").unwrap();
    let commit_hash = "b7d97d5f3f8f897b88872b6935e4c996b955bc1f";
    let source_dir = Path::new(&output).join("libyuv-source");

    // Download the repository as a tarball to avoid git authentication issues
    if source_dir.exists() {
        fs::remove_dir_all(&source_dir)?;
    }

    println!(
        "cargo::warning=Downloading libyuv at commit {}",
        commit_hash
    );

    // Download tarball from GitHub
    let tarball_url = format!(
        "https://github.com/lemenkov/libyuv/archive/{}.tar.gz",
        commit_hash
    );

    let response = ureq::get(&tarball_url).call()?;
    if response.status() != 200 {
        return Err(anyhow::anyhow!(
            "Failed to download tarball: HTTP {}",
            response.status()
        ));
    }

    let reader = response.into_body().into_reader();
    let gz_decoder = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(gz_decoder);

    // Extract to a temporary directory first
    let temp_extract_dir = Path::new(&output).join("temp_extract");
    if temp_extract_dir.exists() {
        fs::remove_dir_all(&temp_extract_dir)?;
    }
    fs::create_dir_all(&temp_extract_dir)?;

    archive.unpack(&temp_extract_dir)?;

    // Find the extracted directory (should be libyuv-{commit_hash})
    let extracted_dir = temp_extract_dir.join(format!("libyuv-{}", commit_hash));
    if !extracted_dir.exists() {
        return Err(anyhow::anyhow!(
            "Expected directory {} not found after extraction",
            extracted_dir.display()
        ));
    }

    // Move it to our expected location
    fs::rename(&extracted_dir, &source_dir)?;

    // Clean up temporary directory
    fs::remove_dir_all(&temp_extract_dir)?;

    // Build with cmake
    let dst = cmake::Config::new(&source_dir)
        .define("CMAKE_BUILD_TYPE", "Release")
        .build();

    println!("cargo::rustc-link-search=native={}/lib", dst.display());
    println!("cargo::rustc-link-lib=static=yuv");

    Ok(())
}
