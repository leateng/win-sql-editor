# bindgen

bindgen deps\scintilla\include\Scintilla.h -o src\scintilla\bindings.rs
bindgen deps\lexilla\include\Lexilla.h -o src\lexilla\bindings.rs  
bindgen deps\lexilla\include\SciLexer.h -o src\lexilla\sci_lexer.rs

# todo

- editor
  - keywords highlight
  - formatter
  - autocomplete
  - theme
- layout
- grid
- execute sql
