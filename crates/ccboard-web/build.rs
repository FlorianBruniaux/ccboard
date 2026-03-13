use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=dist");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist = Path::new(&manifest_dir).join("dist");

    let dist_is_empty = dist
        .read_dir()
        .map(|mut d| d.next().is_none())
        .unwrap_or(true);

    if !dist.exists() || dist_is_empty {
        fs::create_dir_all(&dist).expect("build.rs: failed to create dist/ placeholder");
        fs::write(
            dist.join("index.html"),
            include_str!("build-placeholder.html"),
        )
        .expect("build.rs: failed to write placeholder index.html");
        println!(
            "cargo:warning=dist/ absent — placeholder web UI. Run `trunk build --release` in crates/ccboard-web/ for full frontend."
        );
    }
}
