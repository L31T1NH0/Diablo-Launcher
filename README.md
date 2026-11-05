<p align="center">
  <img src="assets/devilutionx-wordmark.png" width="340" alt="DevilutionX">
</p>

# Devilution Launcher

A simple way to play Diablo I on Linux. Start the game, change your settings, and return to Tristram.

![The launcher home screen](assets/launcher.png)

## Make it yours

- Adjust graphics, sound, gameplay, keyboard shortcuts, and controller buttons.
- Browse 190 game options, grouped into clear categories.
- Save changes with a backup of your previous settings.
- Enable Hellfire under **Settings → Advanced** when you want it.

![Game settings in the launcher](assets/settings.png)

## Get started

First, install [DevilutionX](https://github.com/diasurgical/DevilutionX) and add your [Diablo game files](https://devilutionx.com/mpq). Linux Mint users can also use the [separate game installer](https://github.com/L31T1NH0/diablo-mint-installer).

Then download this project and install the launcher:

```bash
git clone https://github.com/L31T1NH0/devilution-launcher.git
cd devilution-launcher
bash install-launcher.sh
```

Open **Devilution Launcher** from your applications menu and click **Play**.

If the game is not found, go to **Settings → Advanced** and choose the game executable and data folder. Use **Save changes** after editing your game settings.

The launcher does not include Diablo or Hellfire. You need your own game files.

## Technical notes

- Linux, Rust 1.88+, and egui. The installer builds the app on your machine.
- The option list targets DevilutionX 1.5.5; extra settings found in your game file are included too.
- Settings are backed up as `diablo.ini.launcher.bak`. Unsaved game settings are discarded when you close the launcher.
- Nine tests pass. The interface has been checked on X11; Linux Mint and a full Hellfire installation still need testing.

<details>
<summary>Build requirements and checks</summary>

Install [Rust](https://rustup.rs). On Linux Mint or Ubuntu, install these packages before running the installer:

```bash
sudo apt-get install git build-essential pkg-config python3 libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libgl1-mesa-dev libwayland-dev libxkbcommon-dev
```

To run from source or check the code:

```bash
cargo run --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

The launcher installs to `~/.local/bin/devilution-launcher`. The game log is saved in `~/.local/state/devilution-launcher/game.log`, or under `$XDG_STATE_HOME` when set.

</details>

[Option reference](config/README.md) · [Image credits](assets/SOURCES.md)

An independent project. Not affiliated with Blizzard Entertainment or the DevilutionX team.
