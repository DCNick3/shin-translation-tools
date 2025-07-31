# Version 0.10.2

- Adds support for Umineko no Naku Koro ni Saku ~Nekobako to Musou no Koukyoukyoku~ (`01006A300BA2C000`, released on
  2021-01-28)
- Adds support for Higurashi no Naku Koro ni Hou (v1 and v2) (`0100F6A00A684000`, released on 2018-07-26) scenarios
    - `shin-tl` can't produce scripts byte-for-byte equal to the original ones for now. This is due to inconsistent
      usage of fixup encoding by the game. This is intended to be fixed. This should not leave to any noticeable in-game
      behaviour differences, just makes the tool harder to validate.
    - The 2.0.0 update changed the format of the scenario, so this game corresponds to two engine versions in `shin-tl`:
      `higurashi-hou` and `higurashi-hou-v2`

# Version 0.10.1

- Adds support for Gensou Rougoku no Kaleidoscope 2 scenarios (`0100451020714000`, released on 2025-03-14)

# Version 0.10.0

This version adds support for text reflowing, allowing to bake-in linebreaks that are more natural for western
languages. See [this README section](README.md#text-reflowing-with-shin-translation-tools) for usage documentation

This is only implemented for Higurashi no Naku Koro ni Sui (`PCSG00517`) for now.

# Version 0.9.0

This version includes features that make it easier to translate older versions of the engines using unescaped commands,
like for Higurashi no Naku Koro ni Sui (`PCSG00517`) and ALIA's CARNIVAL (`PCSG00628`)

## Breaking changes

- Add an optional linter that checks for invalid commands in messages. If you wish to disable the linting and generate
  the file despite that, pass the `--no-lint` flag.
- Rewriter will now ignore the value in the untraslated `s` column, only reading the contents of the `translated`
  column. If there's no value there, the string will be left as it is in the SNR file.
  The old behaviour of checking the `translated` column and falling back to the `s` column can be enabled by passing
  `--replacement-mode translated-or-original` flag.

## Features

- Change the way fixups encoding is implemented; Make the usage of fixup encoding contextual by parsing the layout
  commands. This allows the rewriting to produce SNR files that are byte-for-byte equal for all supported engine
  versions, making me much more confident in the validity of the tool.
- Adds support for transforming layout command format between old unescaped format and the new escaped one.
  This makes translations to languages that use ASCII characters much easier.
  See [this README section](README.md#dealing-with-ascii-characters-in-older-games) for usage documentation

# Version 0.8.0

- Adds support for Higurashi no Naku Koro ni Sui ps vita scenarios (`PCSG00517`, released on 2015-01-28)

# Version 0.7.0

- Adds support for Konosuba switch scenarios (`01004920105FC000`, released on 2020-08-27)

# Version 0.6.0

- Adds support for ALIA's CARNIVAL ps vita scenarios (`PCSG00628`, released on 2015-10-29)
    - English translations won't directly work, see https://github.com/DCNick3/shin-translation-tools/issues/3

# Version 0.5.1

- Make DC4 MSGSET use the fixup encoding, resulting in slightly smaller file size

# Version 0.5.0

- Adds support for DC4 switch scenarios (`0100D8500EE14000`, released on 2019-12-19)

# Version 0.4.0

- Adds ROM file support
- Updates CLI, now making the snr actions a subcommand
