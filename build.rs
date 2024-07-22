extern crate embed_resource;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    embed_resource::compile("app.rc", embed_resource::NONE);

    cc::Build::new()
        .cpp(true)
        .define("USE_D2D", "1")
        .include("./deps/scintilla/include")
        .include("./deps/scintilla/src/")
        .flag("/std:c++17")
        .file("./deps/scintilla/src/AutoComplete.cxx")
        .file("./deps/scintilla/src/CallTip.cxx")
        .file("./deps/scintilla/src/CaseConvert.cxx")
        .file("./deps/scintilla/src/CaseFolder.cxx")
        .file("./deps/scintilla/src/CellBuffer.cxx")
        .file("./deps/scintilla/src/ChangeHistory.cxx")
        .file("./deps/scintilla/src/CharacterCategoryMap.cxx")
        .file("./deps/scintilla/src/CharacterType.cxx")
        .file("./deps/scintilla/src/CharClassify.cxx")
        .file("./deps/scintilla/src/ContractionState.cxx")
        .file("./deps/scintilla/src/DBCS.cxx")
        .file("./deps/scintilla/src/Decoration.cxx")
        .file("./deps/scintilla/src/Document.cxx")
        .file("./deps/scintilla/src/EditModel.cxx")
        .file("./deps/scintilla/src/Editor.cxx")
        .file("./deps/scintilla/src/EditView.cxx")
        .file("./deps/scintilla/src/Geometry.cxx")
        .file("./deps/scintilla/src/Indicator.cxx")
        .file("./deps/scintilla/src/KeyMap.cxx")
        .file("./deps/scintilla/src/LineMarker.cxx")
        .file("./deps/scintilla/src/MarginView.cxx")
        .file("./deps/scintilla/src/PerLine.cxx")
        .file("./deps/scintilla/src/PositionCache.cxx")
        .file("./deps/scintilla/src/RESearch.cxx")
        .file("./deps/scintilla/src/RunStyles.cxx")
        .file("./deps/scintilla/src/Selection.cxx")
        .file("./deps/scintilla/src/Style.cxx")
        .file("./deps/scintilla/src/UndoHistory.cxx")
        .file("./deps/scintilla/src/UniConversion.cxx")
        .file("./deps/scintilla/src/UniqueString.cxx")
        .file("./deps/scintilla/src/ViewStyle.cxx")
        .file("./deps/scintilla/src/XPM.cxx")
        .file("./deps/scintilla/src/ScintillaBase.cxx")
        .file("./deps/scintilla/win32/HanjaDic.cxx")
        .file("./deps/scintilla/win32/PlatWin.cxx")
        .file("./deps/scintilla/win32/ScintillaWin.cxx")
        .compile("scintilla");

    println!("cargo:rustc-link-lib=dylib=gdi32");
    println!("cargo:rustc-link-lib=dylib=user32");
    println!("cargo:rustc-link-lib=dylib=imm32");
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=uuid");
    println!("cargo:rustc-link-lib=dylib=oleaut32");
    println!("cargo:rustc-link-lib=dylib=advapi32");
}
