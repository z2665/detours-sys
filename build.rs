use std::path::{Path, PathBuf};

fn build_detours() {
    cc::Build::new()
        .include("dep/Detours/src/")
        .static_crt(true)
        .flag("/MT")
        .flag("/W4")
        .flag("/WX")
        .flag("/Gy")
        .flag("/Gm-")
        .flag("/Zl")
        .flag("/Od")
        .define("WIN32_LEAN_AND_MEAN", "1")
        .define("_WIN32_WINNT", "0x501")
        .define("DETOURS_VERSION", "0x4c0c1")
        .define("_AMD64_", "1")
        .file("dep/Detours/src/detours.cpp")
        .file("dep/Detours/src/modules.cpp")
        .file("dep/Detours/src/disasm.cpp")
        .file("dep/Detours/src/image.cpp")
        .file("dep/Detours/src/creatwth.cpp")
        .compile("detours");
}

#[cfg(feature = "build_bind")]
fn get_windows_kits_dir() -> Result<PathBuf, std::io::Error> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";

    let dir: String = hklm.open_subkey(key)?.get_value("KitsRoot10")?;
    println!("{}",dir);
    Ok(dir.into())
}
#[cfg(feature = "build_bind")]
fn get_include_dir(windows_kits_dir: &PathBuf,dirname: &str) -> Result<PathBuf, std::io::Error> {
    let readdir = Path::new(windows_kits_dir).join("Include").read_dir()?;

    let max_libdir = readdir
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map(|c| c.starts_with("10.") && dir.join(dirname).is_dir())
                .unwrap_or(false)
        }).max()
        .expect(&format!("Can not find a valid {:?} dir in `{:?}`",dirname, windows_kits_dir));

    Ok(max_libdir.join(dirname))
}
#[cfg(feature = "build_bind")]
fn generate_bindings() {
    use std::{env, fs};
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy("dep/Detours/src/detours.h", out_path.join("detours.h")).unwrap();
    let wkit = get_windows_kits_dir().expect("not found windows kits");
    //
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", out_path.to_str().expect("OUTDIR is weird")))
        .clang_arg("-fms-compatibility")
        .clang_arg("-fms-extensions")
        .clang_arg(format!("-I{}",get_include_dir(&wkit,"ucrt").unwrap().to_str().unwrap()))
        .clang_arg(format!("-I{}",get_include_dir(&wkit,"um").unwrap().to_str().unwrap()))
        .clang_arg(format!("-I{}",get_include_dir(&wkit,"shared").unwrap().to_str().unwrap()))
        // Detouring APIs
        .allowlist_function("DetourTransactionBegin")
        .allowlist_function("DetourUpdateThread")
        .allowlist_function("DetourAttach")
        .allowlist_function("DetourAttachEx")
        .allowlist_function("DetourDetach")
        .allowlist_function("DetourSetIgnoreTooSmall")
        .allowlist_function("DetourSetRetainRegions")
        .allowlist_function("DetourSetSystemRegionLowerBound")
        .allowlist_function("DetourSetSystemRegionUpperBound")
        .allowlist_function("DetourTransactionAbort")
        .allowlist_function("DetourTransactionCommit")
        .allowlist_function("DetourTransactionCommitEx")
        // Targeting APIs
        .allowlist_function("DetourFindFunction")
        .allowlist_function("DetourCodeFromPointer")
        // Binary and Payload access APIs
        .allowlist_function("DetourEnumerateModules")
        .allowlist_function("DetourGetEntryPoint")
        .allowlist_function("DetourGetModuleSize")
        .allowlist_function("DetourEnumerateExports")
        .allowlist_function("DetourEnumerateImports")
        .allowlist_function("DetourEnumerateImportsEx")
        .allowlist_function("DetourFindPayload")
        .allowlist_function("DetourGetContainingModule")
        .allowlist_function("DetourGetSizeOfPayloads")
        // Binary Modifcation APIs
        .allowlist_function("DetourBinaryOpen")
        .allowlist_function("DetourBinaryEnumeratePayloads")
        .allowlist_function("DetourBinaryFindPayload")
        .allowlist_function("DetourBinarySetPayload")
        .allowlist_function("DetourBinaryDeletePayload")
        .allowlist_function("DetourBinaryPurgePayloads")
        .allowlist_function("DetourBinaryEditImports")
        .allowlist_function("DetourBinaryResetImports")
        .allowlist_function("DetourBinaryWrite")
        .allowlist_function("DetourBinaryClose")
        // Injection APIs
        .allowlist_function("DetourCreateProcessWithDllW")
        .allowlist_function("DetourCreateProcessWithDllExW")
        .allowlist_function("DetourCreateProcessWithDllsW")
        .allowlist_function("DetourCreateProcessWithDllA")
        .allowlist_function("DetourCreateProcessWithDllExA")
        .allowlist_function("DetourCreateProcessWithDllsA")
        .allowlist_function("DetourCopyPayloadToProcess")
        .allowlist_function("DetourFinishHelperProcess")
        .allowlist_function("DetourIsHelperProcess")
        .allowlist_function("DetourRestoreAfterWith")
        //
        .header("build/wrapper.h")
        .generate()
        .expect("Unable to generate bindings");
    //
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(not(feature = "build_bind"))]
fn generate_bindings() {}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_detours();
    generate_bindings();
}