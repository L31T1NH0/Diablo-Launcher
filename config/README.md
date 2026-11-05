# Game options

The launcher includes 190 Linux settings from DevilutionX 1.5.5. They cover graphics, sound, language, gameplay, startup screens, demo mode, networking, chat messages, keyboard shortcuts, and controller buttons.

Your current settings always take priority. Extra options found in your game file are also shown. The launcher only writes values you change and keeps a backup before saving.

## Technical reference

- [options.cpp](https://github.com/diasurgical/DevilutionX/blob/1.5.5/Source/options.cpp): option names, defaults, and available values.
- [options.h](https://github.com/diasurgical/DevilutionX/blob/1.5.5/Source/options.h): display and startup modes.
- [sound_defs.hpp](https://github.com/diasurgical/DevilutionX/blob/1.5.5/Source/engine/sound_defs.hpp): sound volume range.

The catalog is stored in `defaults-1.5.5.ini`. It fills gaps in the interface without replacing the user's file. Internal save data (`LastSinglePlayerHero`, `LastMultiplayerHero`, and `SItem`) is left untouched. The launcher controls `GameMode.Game` through its expansion setting. Options for other platforms are outside this Linux catalog. Some languages and optional features require additional game files.
