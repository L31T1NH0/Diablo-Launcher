# Devilution Launcher

![Interface do launcher](assets/launcher.png)

O **Devilution Launcher** é uma interface nativa para iniciar uma instalação existente do DevilutionX. Tem seleção de Diablo I/Hellfire, detecção de instalação, caminhos configuráveis, opção de pular os vídeos, configuração persistente e acesso aos dados, `diablo.ini` e registro de execução. A interface fica disponível enquanto o jogo roda e impede abrir uma segunda instância pelo mesmo launcher.

Ele usa uma instalação existente do DevilutionX, incluindo a criada pelo [instalador para Linux Mint](https://github.com/L31T1NH0/diablo-mint-installer). Hellfire exige também `hellfire.mpq`, `hfmonk.mpq`, `hfmusic.mpq` e `hfvoice.mpq`. Os caminhos em **Configuração** devem ser completos, sem `~`. Escolha preferencialmente o executável original `devilutionx`, em vez de um script que já passe seus próprios argumentos.

### Compilar no Linux Mint

Instale [Rust 1.88 ou superior](https://rustup.rs) e as dependências de compilação:

```bash
sudo apt-get install build-essential pkg-config python3 libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libgl1-mesa-dev libwayland-dev libxkbcommon-dev
```

Na pasta deste repositório:

```bash
cargo run --locked
```

Para compilar uma versão otimizada e criar um atalho no menu:

```bash
bash instalar-launcher.sh
```

Não execute o launcher nem esse instalador como root. O instalador do launcher não instala o motor nem baixa os dados do jogo; para isso, use o [instalador separado](https://github.com/L31T1NH0/diablo-mint-installer).

O launcher mantém os saves e o `diablo.ini` na pasta dos dados selecionada. Quando necessário, copia `devilutionx.mpq` e `fonts.mpq` da pasta do motor para essa pasta. Suas preferências de interface são salvas pelo eframe na pasta de dados do usuário. O registro do jogo fica em `~/.local/state/devilution-launcher/game.log` (ou `$XDG_STATE_HOME/devilution-launcher/game.log`). Fechar o launcher mantém o jogo aberto.

### Verificar o launcher

```bash
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Validado: compilação, três testes automatizados, Clippy sem avisos e abertura da interface em Linux com X11. O comando de execução também foi conferido com uma instalação real de Diablo I. A execução em Linux Mint e os dados reais de Hellfire ainda não foram testados.
