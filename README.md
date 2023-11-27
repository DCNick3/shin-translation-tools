
A future home for translation-focused tools for the shin engine.

I've already made a few tools for this engine:
- [shin](https://github.com/DCNick3/shin) - a reimplementation of the umineko version of shin engine and `sdu` - a companion tool for extracting game data
- [ShinDataUtil](https://github.com/DCNick3/ShinDataUtil) - a comprehensive tool for extracting and repacking the game data 

However, they target relatively modern versions of the engine (the former - switch's umineko, the latter - switch's higurashi).

The aim of this project is to provide tools that cover a wider range of versions.

As a starting point I'm targeting the Astral Air version of the engine.

## Tools

### shin-tl

A tool for extracting and injecting translated strings from/to the game script. Unlike what `ShinDataUtil` or `sdu` does, it does not attempt to fully disassemble the scripts, but instead focuses on the most important part of translation - strings.

Hopefully this will allow it to easier support more versions of the engine.

See [README.md](shin-tl/README.md) for more information.

### ...more to come

I plan to make at least a tool for working with ROM files, and maybe multi-version tools for `PIC` and `TXA` files.

## License

Licensed under Mozilla Public License 2.0. See [LICENSE](LICENSE) for more information.