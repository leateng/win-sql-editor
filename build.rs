extern crate embed_resource;

use cc::Build;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    let _ = embed_resource::compile("app.rc", embed_resource::NONE);

    let mut scintilla_build = cc::Build::new();
    scintilla_build
        .cpp(true)
        .flag("/std:c++17")
        .include("./deps/scintilla/include")
        .include("./deps/scintilla/src/");
    add_dir_files(&mut scintilla_build, "./deps/scintilla/src/", "cxx");
    scintilla_build
        .file("./deps/scintilla/win32/HanjaDic.cxx")
        .file("./deps/scintilla/win32/PlatWin.cxx")
        .file("./deps/scintilla/win32/ListBox.cxx")
        .file("./deps/scintilla/win32/SurfaceGDI.cxx")
        .file("./deps/scintilla/win32/SurfaceD2D.cxx")
        .file("./deps/scintilla/win32/ScintillaWin.cxx")
        .compile("scintilla");

    let mut lexilla_build = cc::Build::new();
    lexilla_build
        .cpp(true)
        .flag("/std:c++17")
        .include("./deps/scintilla/include/")
        .include("./deps/lexilla/include/")
        .include("./deps/lexilla/lexlib/")
        .file("./deps/lexilla/src/Lexilla.cxx");
    add_dir_files(&mut lexilla_build, "./deps/lexilla/lexers/", "cxx");
    add_dir_files(&mut lexilla_build, "./deps/lexilla/lexlib/", "cxx");
    lexilla_build.compile("lexilla");

    println!("cargo:rustc-link-lib=dylib=gdi32");
    println!("cargo:rustc-link-lib=dylib=user32");
    println!("cargo:rustc-link-lib=dylib=imm32");
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=uuid");
    println!("cargo:rustc-link-lib=dylib=oleaut32");
    println!("cargo:rustc-link-lib=dylib=advapi32");
}

fn add_dir_files(build: &mut Build, dir: &str, extension: &str) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    build.file(path);
                }
            }
        }
    }
}
