
A future home for translation-focused tools for the shin engine.

I've already made a few tools for this engine:
- [shin](https://github.com/DCNick3/shin) - a reimplementation of the umineko version of shin engine and `sdu` - a companion tool for extracting game data
- [ShinDataUtil](https://github.com/DCNick3/ShinDataUtil) - a comprehensive tool for extracting and repacking the game data 

However, they target relatively modern versions of the engine (the former - switch's umineko, the latter - switch's higurashi).

The aim of this project is to provide tools that cover a wider range of versions.

As a starting point I'm targeting the "AstralAir no Shiroki Towa -White Eternity-" version of the engine.

## Features

- Extraction and compilation of ROM files compatible with, AFAIK, all* versions of the engine
- Translation of the game script (SNR files). For now, only for `white-eternity` version of the engine, more to come

\* except for PS2 versions, which embed rom files directly onto the disc

In the future I plan to implement more engine versions for the SNR translation tool, and maybe make a multi-version tool for working with graphics (PIC and TXA).

## Workflow

### Preparation

First of all, you would need the game files. How you obtain it differs by the platform.

On Switch, you would need to first dump the game with [nxdumptool](https://github.com/DarkMatterCore/nxdumptool) and then extract the romfs files with [hactoolnet](https://github.com/Thealexbarney/LibHac). 

shin engine usually packages its files into one or two `.rom` files: `data.rom` and sometimes `patch.rom`. You would need to extract them first.

After you have obtained the rom files, `shin-tl` can already handle them

### Extracting the rom files

To extract the rom files, use a command like this:

```bash
shin-tl rom extract <data.rom> <rom-dir>
```

This will create the `rom-dir` directory and extract the rom files into it.

The game stores its data in multitude of formats. The ones that are the most interesting for translation are:
- `SNR` - the game script
- `PIC` - pictures (mostly CGs)
- `TXA` - texture archives (mostly UI elements)
- `FNT` - fonts

As of now, shin-tl can only be used to translate the game script (`SNR` files).

### Translating the SNR files

1. Extract the strings

To extract strings from the snr file into a csv file, use a command like this:

```bash
shin-tl snr read <engine-version> <main.snr> <strings.csv>
```

The format of the SNR files varies greatly with the engine version, and it does not contain any indicators as to which version it is. Thus, you need to supply the engine version to the tool.

As of now, only AstralAir no Shiroki Towa -White Eternity- is supported, so it should be `white-eternity`.

The `strings.csv` file will contain the extracted strings. It can be edited with a spreadsheet editor like Excel or LibreOffice Calc.

Here's how it will look like:

```csv
index,offset,source,source_subindex,s,translated
<...>
53,0x00037e62,saveinfo,0,共通ルート,
54,0x00037e71,saveinfo,0,プロローグ,
55,0x00037f15,msgset,1,@rここは自由に駆け回れる庭だった。,
56,0x00037f3a,msgset,2,@r好きに生きることを許された世界だった。,
57,0x00037f62,msgset,3,@rそれ以上に求めるものはない。,
58,0x00037f82,msgset,4,@rやわらかい雪の上に、想うだけ足跡をつけたなら、この心は簡単に満たされる。,
<...>
```

The `index` column is used to later inject the translated strings back into the SNR file, while `offset`, `source` and `source_subindex` provide information about where the string comes from.

If you are using a spreadsheet editor, take care to avoid conversion of the columns to numbers, as it will break the tool.

In case of Google Spreadsheets, make sure to uncheck the "Convert text to numbers, dates and formulas" option when importing the csv:

![](googledoc_import.png)

2. Inject the translated strings back

Create a translation csv by either putting your translation into the `translated` column, or modifying the `s` column directly. The `translated` column will take precedence over the `s` column.

To inject the translated strings back into the snr file, use a command like this:

```bash
shin-tl snr rewrite <engine-version> <main.snr> <translation.csv> <main_translated.snr>
```

This will read the translation csv, replace the strings in the snr file and write the result to `main_translated.snr`.

### Rebuild the rom file

After touching all the files you wanted to translate, you would need to package them back into a `.rom` file.

All games I saw so far load the `patch.rom` file on top of `data.rom` (even if in original distribution there's no `patch.rom`). It also tends to be smaller, so you almost definitely want to put your translated files into `patch.rom`.

To rebuild the rom file, use a command like this:

```bash
shin-tl rom create --rom-version <rom-or-game-version> <rom-dir> <patch.rom> 
```

This will package all files and directories inside `rom-dir` into a `patch.rom` file.

Note that the rom format varies from game to game, so you need to supply either the rom format (`rom1-v2-1`, `rom2-v1-0` or `rom2-v1-1`) or the engine version to the tool. You can see the correspondence in [this spreadsheet](https://docs.google.com/spreadsheets/d/1wGX9FOQq_iXcWMnY9qITCAV7hq1R7_gpWwjkT4_tKDI).

After that, you can put the `patch.rom` file back into the game (however it is done on the platform you are working with).

On Switch you would use LayeredFS mods to do that.

## Alternatives

There are alternatives to this project, which support different versions of the engine:

- [ShinDataUtil](https://github.com/DCNick3/ShinDataUtil) (by me) has full support for (almost?) all formats used by switch's Higurashi Hou and Konosuba
- [sdu](https://github.com/DCNick3/shin#what-else-is-in-the-box) (also by me) has support for extraction (but not creation) of most formats used by switch's Umineko
- [enter_extractor](https://github.com/07th-mod/enter_extractor) (by 07th-mod) has support for extraction and patching of most formats across several versions of the engine
- [kaleido](https://gitlab.com/Neurochitin/kaleido/) (by Neurochitin) has support for scenario translation of vita's umineko and vita and switch's Kaleido 

## License

Licensed under Mozilla Public License 2.0. See [LICENSE](LICENSE) for more information.