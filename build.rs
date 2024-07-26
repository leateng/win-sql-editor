extern crate embed_resource;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    embed_resource::compile("app.rc", embed_resource::NONE);

    cc::Build::new()
        .cpp(true)
        .flag("/std:c++17")
        .include("./deps/scintilla/include")
        .include("./deps/scintilla/src/")
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

    let mut lexilla_builder = cc::Build::new();
    lexilla_builder
        .cpp(true)
        .flag("/std:c++17")
        .include("./deps/scintilla/include/")
        .include("./deps/lexilla/include/")
        .include("./deps/lexilla/lexlib/")
        // .file("./deps/lexilla/lexers/LexSQL.cxx")
        // .file("./deps/lexilla/lexers/LexMSSQL.cxx")
        // .file("./deps/lexilla/lexers/LexJSON.cxx")
        // .file("./deps/lexilla/lexers/LexHTML.cxx")
        // .file("./deps/lexilla/lexers/LexYAML.cxx")
        .file("./deps/lexilla/lexlib/Accessor.cxx")
        .file("./deps/lexilla/lexlib/CharacterCategory.cxx")
        .file("./deps/lexilla/lexlib/CharacterSet.cxx")
        .file("./deps/lexilla/lexlib/DefaultLexer.cxx")
        .file("./deps/lexilla/lexlib/InList.cxx")
        .file("./deps/lexilla/lexlib/LexAccessor.cxx")
        .file("./deps/lexilla/lexlib/LexerBase.cxx")
        .file("./deps/lexilla/lexlib/LexerModule.cxx")
        .file("./deps/lexilla/lexlib/LexerSimple.cxx")
        .file("./deps/lexilla/lexlib/PropSetSimple.cxx")
        .file("./deps/lexilla/lexlib/StyleContext.cxx")
        .file("./deps/lexilla/lexlib/WordList.cxx")
        .file("./deps/lexilla/src/Lexilla.cxx");

    let lexilla_dir = "./deps/lexilla/lexers/";
    for entry in fs::read_dir(lexilla_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "cxx" {
                    lexilla_builder.file(path);
                }
            }
        }
    }

    lexilla_builder.compile("lexilla");

    println!("cargo:rustc-link-lib=dylib=gdi32");
    println!("cargo:rustc-link-lib=dylib=user32");
    println!("cargo:rustc-link-lib=dylib=imm32");
    println!("cargo:rustc-link-lib=dylib=ole32");
    println!("cargo:rustc-link-lib=dylib=uuid");
    println!("cargo:rustc-link-lib=dylib=oleaut32");
    println!("cargo:rustc-link-lib=dylib=advapi32");
}
