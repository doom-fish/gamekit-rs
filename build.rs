use std::env;
use std::process::Command;

fn detect_sdk_major_version() -> Option<u32> {
    let output = Command::new("xcrun")
        .args(["--sdk", "macosx", "--show-sdk-version"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout);
    version.trim().split('.').next()?.parse().ok()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
    println!("cargo:rerun-if-env-changed=SDKROOT");

    if env::var("DOCS_RS").is_ok() {
        return;
    }

    println!("cargo:rustc-link-lib=framework=GameKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");

    let swift_dir = "swift-bridge";
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should be set by cargo");
    let swift_build_dir = format!("{out_dir}/swift-build");
    println!("cargo:rerun-if-changed={swift_dir}");

    if let Ok(output) = Command::new("swiftlint")
        .args(["lint"])
        .current_dir(swift_dir)
        .output()
    {
        if !output.status.success() {
            eprintln!(
                "SwiftLint warnings:\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let swift_triple = match target_arch.as_str() {
        "x86_64" => "x86_64-apple-macosx",
        "aarch64" => "arm64-apple-macosx",
        other => panic!("gamekit-rs: unsupported target arch '{other}'"),
    };

    let sdk_major = detect_sdk_major_version();
    let mut swift_args = vec![
        "build".to_owned(),
        "-c".to_owned(),
        "release".to_owned(),
        "--triple".to_owned(),
        swift_triple.to_owned(),
        "--package-path".to_owned(),
        swift_dir.to_owned(),
        "--scratch-path".to_owned(),
        swift_build_dir.clone(),
    ];

    if sdk_major.is_some_and(|major| major >= 26) {
        swift_args.push("-Xswiftc".to_owned());
        swift_args.push("-DGAMEKIT_HAS_MACOS26_SDK".to_owned());
    }

    let output = Command::new("swift")
        .args(&swift_args)
        .output()
        .expect("Failed to build Swift bridge");

    if !output.status.success() {
        eprintln!(
            "Swift build STDOUT:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "Swift build STDERR:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!("Swift build failed with exit code: {:?}", output.status.code());
    }

    println!("cargo:rustc-link-search=native={swift_build_dir}/release");
    println!("cargo:rustc-link-lib=static=GameKitBridge");
    println!("cargo:rustc-link-arg=-mmacosx-version-min=12.0");
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");

    if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
        if output.status.success() {
            let xcode_path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            let swift_lib_path =
                format!("{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx");
            println!("cargo:rustc-link-search=native={swift_lib_path}");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path}");
        }
    }
}
