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

    let sdk_major = detect_sdk_major_version();
    let has_macos26_sdk = sdk_major.is_some_and(|major| major >= 26);

    // When the SDK is macOS 26+, GKLocalPlayerListener (available since macOS 10.10) was
    // updated to inherit GKGameActivityListener (available since macOS 26.0) without a
    // deployment-target guard.  Swift's conformance checker then requires every optional
    // method implementation for that sub-protocol to be available since the conformance's
    // deployment target (normally macOS 12.0).  But GKGameActivity itself is macOS 26.0+,
    // creating an unsatisfiable constraint.
    //
    // Resolved by overriding the Swift compiler's deployment target to 26.0 (via
    // -Xswiftc -target) when building against the macOS 26+ SDK.  SPM ignores the version
    // component in --triple and always applies its own platform minimum (macOS 12.0 from
    // Package.swift); the -Xswiftc flag bypasses that. The Rust linker flag
    // -mmacosx-version-min=12.0 (set below) still governs the final binary's minimum OS,
    // so the library remains loadable on macOS 12+.  All macOS-26-specific API call sites
    // are behind @available(macOS 26.0, *) guards and are never reached on older systems.
    let swift_arch = match target_arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        other => panic!("gamekit-rs: unsupported target arch '{other}'"),
    };
    let swift_triple = format!("{swift_arch}-apple-macosx");

    let mut swift_args = vec![
        "build".to_owned(),
        "-c".to_owned(),
        "release".to_owned(),
        "--triple".to_owned(),
        swift_triple,
        "--package-path".to_owned(),
        swift_dir.to_owned(),
        "--scratch-path".to_owned(),
        swift_build_dir.clone(),
    ];

    if has_macos26_sdk {
        swift_args.push("-Xswiftc".to_owned());
        swift_args.push("-DGAMEKIT_HAS_MACOS26_SDK".to_owned());
        // Override the deployment target that SPM infers from Package.swift (macOS 12.0) so
        // that @available(macOS 26.0, *) protocol witnesses are accepted by the compiler.
        swift_args.push("-Xswiftc".to_owned());
        swift_args.push("-target".to_owned());
        swift_args.push("-Xswiftc".to_owned());
        swift_args.push(format!("{swift_arch}-apple-macosx26.0"));
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
