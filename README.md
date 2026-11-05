# Devilution Launcher

![Interface do launcher](assets/launcher.png)

O **Devilution Launcher** é uma interface nativa para iniciar uma instalação existente do DevilutionX. A tela principal mantém apenas a arte de Diablo, a navegação e o botão Jogar. A aba **Configurações** permite editar vídeo, áudio, idioma e jogabilidade no `diablo.ini`. A opção de Hellfire fica recolhida em **Configurações → Avançado**, junto dos caminhos da instalação. A interface fica disponível enquanto o jogo roda e impede abrir uma segunda instância pelo mesmo launcher.

Ele usa uma instalação existente do DevilutionX, incluindo a criada pelo [instalador para Linux Mint](https://github.com/L31T1NH0/diablo-mint-installer). Hellfire exige também `hellfire.mpq`, `hfmonk.mpq`, `hfmusic.mpq` e `hfvoice.mpq`. Os caminhos em **Configurações → Avançado** devem ser completos, sem `~`. Escolha preferencialmente o executável original `devilutionx`, em vez de um script que já passe seus próprios argumentos.

### Configurações do jogo

Resolução, tela cheia, escala da imagem, controle de FPS, volume de música e efeitos, idioma, corrida na cidade, coleta automática de ouro e atalhos de magia estão disponíveis na aba de configurações. Clique em **Salvar alterações** para aplicar. Enquanto o jogo estiver rodando pelo launcher, a edição fica bloqueada.

O launcher escreve apenas as opções alteradas e preserva as outras chaves do arquivo. Antes de gravar, cria `diablo.ini.launcher.bak`; a substituição do INI é feita por renomeação de um arquivo temporário. Se o arquivo tiver sido alterado externamente desde a leitura, a gravação é recusada: use **Recarregar** para ler a versão atual. Configurações do jogo não salvas são descartadas ao fechar; os caminhos e preferências do launcher continuam sendo salvos pelo eframe.

As imagens são incorporadas ao executável e funcionam sem conexão. As fontes e créditos estão em [assets/SOURCES.md](assets/SOURCES.md).

![Configurações](assets/settings.png)

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

Validado: compilação, seis testes automatizados, Clippy sem avisos e abertura da interface em Linux com X11. O comando de execução também foi conferido com uma instalação real de Diablo I. A execução em Linux Mint e os dados reais de Hellfire ainda não foram testados.
