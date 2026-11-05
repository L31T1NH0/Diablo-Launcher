#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
if ! command -v cargo >/dev/null; then
    echo 'Instale Rust 1.88 ou superior: https://rustup.rs' >&2
    exit 1
fi
cargo build --release --locked
mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
install -m 755 target/release/devilution-launcher "$HOME/.local/bin/devilution-launcher"
python3 - <<'PY'
from pathlib import Path
home = Path.home()
exe = str(home / '.local/bin/devilution-launcher')
exe = exe.replace('\\', '\\\\').replace('"', '\\"').replace('`', '\\`').replace('$', '\\$').replace('%', '%%')
(home / '.local/share/applications/devilution-launcher.desktop').write_text(
    '[Desktop Entry]\nType=Application\nName=Devilution Launcher\n'
    'Comment=Jogue Diablo I e Hellfire com DevilutionX\n'
    f'Exec="{exe}"\nIcon=applications-games\nCategories=Game;RolePlaying;\nTerminal=false\n'
)
PY
echo 'Pronto! Abra Devilution Launcher pelo menu de aplicativos.'
