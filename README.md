# The Beans scripting language
*insert some cute kitty toe beans logo here*

## This document is still WIP!

## Introduction
Beans is a simple scripting language made because i was curious about the inner mechanisms of interpreters and parsers, and wanted to improve my Rust skills.
I decided that the best way to learn all that stuff was by implementing my personal scripting language.

Beans is a static scoped language with a dynamic type checker, with the syntax resembling Lua's:
```lua
function factorial(n)
    if n == 1 or n == 0 then
        return 1;
    else
        return n * factorial(n - 1);
    end
end

print("The factorial of 5 is:", factorial(5));
```

notice the ending `;` on every statement.

## Running the included REPL shell
Simply clone the repo and do `cargo run` inside the repository root, and the shell will be run in interactive mode.
By passing some files to `cargo run`, the shell will (try to) open and execute each of them.

## Project organization

* `src/main.rs` is a simple program meant to run language scripts (by passing some script), or to act as a (WIP) REPL interactive interpreter, by passing no files.

* `beans_lang` is the directory containing the actual language implementation:
  * `grammar.txt` is the formal grammar of the language;
  * `generate_tokens.py` is an helper script that parser a grammar.txt and generates a Rust source file with an enum containing
  all the tokens necessary for parsing the language;
  * `beans_lang/src` contains all the source files of the reference interpreter for the language.

* The `tests` folder is contains some Beans programs used to test the reference language interpreter.