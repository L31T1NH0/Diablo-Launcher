use crate::game_config::ExtraOption;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    Video,
    Audio,
    Game,
    Interface,
    Keyboard,
    Controller,
    Network,
    Startup,
    Other,
}

impl Category {
    pub const ALL: [Self; 9] = [
        Self::Video,
        Self::Audio,
        Self::Game,
        Self::Interface,
        Self::Keyboard,
        Self::Controller,
        Self::Network,
        Self::Startup,
        Self::Other,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Self::Video => "Vídeo",
            Self::Audio => "Áudio",
            Self::Game => "Jogo",
            Self::Interface => "Interface",
            Self::Keyboard => "Teclado",
            Self::Controller => "Controle",
            Self::Network => "Rede",
            Self::Startup => "Inicialização",
            Self::Other => "Outras opções",
        }
    }
}

pub fn category(option: &ExtraOption) -> Category {
    match option.section.as_str() {
        "Graphics" => Category::Video,
        "Audio" => Category::Audio,
        "Game"
            if matches!(
                option.key.as_str(),
                "Experience Bar"
                    | "Show Item Graphics in Stores"
                    | "Show health values"
                    | "Show mana values"
                    | "Enemy Health Bar"
                    | "Show Monster Type"
                    | "Show Item Labels"
                    | "Enable floating numbers"
            ) =>
        {
            Category::Interface
        }
        "Game" | "Language" => Category::Game,
        "Keymapping" => Category::Keyboard,
        "Controller" | "Padmapping" => Category::Controller,
        "Network" | "NetMsg" => Category::Network,
        "StartUp" => Category::Startup,
        _ => Category::Other,
    }
}

pub enum Kind {
    Toggle,
    Integer(i32, i32),
    Decimal(f64, f64),
    Choices(&'static [(&'static str, &'static str)]),
    Text,
}

pub fn kind(option: &ExtraOption) -> Kind {
    match (option.section.as_str(), option.key.as_str()) {
        ("Graphics", "Scaling Quality") => Kind::Choices(&[
            ("0", "Pixel original"),
            ("1", "Bilinear"),
            ("2", "Anisotrópico"),
        ]),
        ("Graphics", "Hardware Cursor Maximum Size") => Kind::Choices(&[
            ("0", "Sem limite"),
            ("64", "64 px"),
            ("128", "128 px"),
            ("256", "256 px"),
            ("512", "512 px"),
        ]),
        ("Graphics", "Gamma Correction") => Kind::Integer(30, 100),
        ("Audio", "Sample Rate") => Kind::Choices(&[
            ("22050", "22050 Hz"),
            ("44100", "44100 Hz"),
            ("48000", "48000 Hz"),
        ]),
        ("Audio", "Channels") => Kind::Choices(&[("1", "Mono"), ("2", "Estéreo")]),
        ("Audio", "Buffer Size") => {
            Kind::Choices(&[("1024", "1024"), ("2048", "2048"), ("5120", "5120")])
        }
        ("Audio", "Resampler") => Kind::Choices(&[("Speex", "Speex"), ("SDL", "SDL")]),
        ("Audio", "Resampling Quality") => Kind::Integer(0, 10),
        ("Audio", "Device") => Kind::Text,
        ("Game", "Speed") => Kind::Integer(1, 1000),
        ("Game", "Enable floating numbers") => Kind::Choices(&[
            ("0", "Desativado"),
            ("1", "Ângulos aleatórios"),
            ("2", "Somente vertical"),
        ]),
        ("Game", key) if key.ends_with("Potion Pickup") => Kind::Choices(&[
            ("0", "Não coletar"),
            ("1", "1"),
            ("2", "2"),
            ("4", "4"),
            ("8", "8"),
            ("16", "16"),
        ]),
        ("StartUp", "Diablo Intro" | "Hellfire Intro") => {
            Kind::Choices(&[("0", "Nunca"), ("1", "Uma vez"), ("2", "Sempre")])
        }
        ("StartUp", "Splash") => Kind::Choices(&[
            ("0", "Nenhuma"),
            ("1", "Tela de título"),
            ("2", "Logo e título"),
        ]),
        ("Network", "Port") => Kind::Integer(1, 65535),
        ("Controller", "deadzone") => Kind::Decimal(0.0, 1.0),
        ("Graphics" | "Game" | "GameMode", _)
            if matches!(
                crate::game_config::default_value(&option.section, &option.key),
                Some("0" | "1")
            ) =>
        {
            Kind::Toggle
        }
        ("Audio", "Walking Sound" | "Auto Equip Sound" | "Item Pickup Sound") => Kind::Toggle,
        _ => Kind::Text,
    }
}

pub fn validate(option: &ExtraOption) -> Result<(), String> {
    let val = option.value.as_str();
    let valid = match kind(option) {
        Kind::Toggle => matches!(val, "0" | "1"),
        Kind::Integer(min, max) => val.parse::<i32>().is_ok_and(|n| (min..=max).contains(&n)),
        Kind::Decimal(min, max) => val
            .parse::<f64>()
            .is_ok_and(|n| n.is_finite() && (min..=max).contains(&n)),
        Kind::Choices(choices) => choices.iter().any(|(v, _)| *v == val),
        Kind::Text => true,
    };
    if valid {
        Ok(())
    } else {
        Err(format!("Valor inválido para {}.", label(option)))
    }
}

pub fn label(option: &ExtraOption) -> String {
    let key = option.key.as_str();
    let translated = match key {
        "Fit to Screen" => "Ajustar à proporção da tela",
        "Scaling Quality" => "Filtro de imagem",
        "Integer Scaling" => "Escala inteira de pixels",
        "Gamma Correction" => "Brilho (gamma)",
        "Zoom" => "Zoom",
        "Color Cycling" => "Animação das cores",
        "Alternate nest art" => "Paleta alternativa do ninho",
        "Hardware Cursor" => "Cursor por hardware",
        "Hardware Cursor For Items" => "Cursor por hardware para itens",
        "Hardware Cursor Maximum Size" => "Tamanho máximo do cursor",
        "Walking Sound" => "Som dos passos",
        "Auto Equip Sound" => "Som ao equipar automaticamente",
        "Item Pickup Sound" => "Som ao coletar itens",
        "Sample Rate" => "Frequência de áudio",
        "Channels" => "Canais de áudio",
        "Buffer Size" => "Buffer de áudio",
        "Resampler" => "Reamostrador",
        "Resampling Quality" => "Qualidade da reamostragem",
        "Device" => "Dispositivo de áudio (vazio: padrão)",
        "Speed" => "Velocidade do jogo (ticks/s)",
        "Friendly Fire" => "Dano entre jogadores",
        "MultiplayerFullQuests" => "Missões completas no multiplayer",
        "Randomize Quests" => "Sortear missões em novos jogos",
        "Theo Quest" => "Missão da menina",
        "Cow Quest" => "Missão de Jersey",
        "Test Bard" => "Habilitar Barda",
        "Test Barbarian" => "Habilitar Bárbaro",
        "Experience Bar" => "Barra de experiência",
        "Show Item Graphics in Stores" => "Imagens dos itens nas lojas",
        "Show health values" => "Valores de vida",
        "Show mana values" => "Valores de mana",
        "Enemy Health Bar" => "Barra de vida dos inimigos",
        "Show Monster Type" => "Tipo dos monstros",
        "Show Item Labels" => "Nomes dos itens no chão",
        "Enable floating numbers" => "Números flutuantes",
        "Auto Refill Belt" => "Reabastecer o cinto automaticamente",
        "Auto Equip Weapons" => "Equipar armas automaticamente",
        "Auto Equip Armor" => "Equipar armaduras automaticamente",
        "Auto Equip Helms" => "Equipar elmos automaticamente",
        "Auto Equip Shields" => "Equipar escudos automaticamente",
        "Auto Equip Jewelry" => "Equipar joias automaticamente",
        "Auto Elixir Pickup" => "Coletar elixires automaticamente",
        "Auto Oil Pickup" => "Coletar óleos automaticamente",
        "Heal Potion Pickup" => "Limite de poções de cura",
        "Full Heal Potion Pickup" => "Limite de poções de cura completa",
        "Mana Potion Pickup" => "Limite de poções de mana",
        "Full Mana Potion Pickup" => "Limite de poções de mana completa",
        "Rejuvenation Potion Pickup" => "Limite de poções de rejuvenescimento",
        "Full Rejuvenation Potion Pickup" => "Limite de poções de rejuvenescimento completo",
        "Auto Pickup in Town" => "Coletar itens automaticamente na cidade",
        "Disable Crippling Shrines" => "Desativar santuários prejudiciais",
        "Adria Refills Mana" => "Adria restaura a mana",
        "Grab Input" => "Prender o mouse à janela",
        "Diablo Intro" => "Vídeo de abertura",
        "Hellfire Intro" => "Vídeo de abertura da expansão",
        "Splash" => "Telas de abertura",
        "Shareware" => "Restringir ao modo de demonstração",
        "Port" => "Porta de rede",
        "Bind Address" => "Endereço de escuta",
        "Previous Host" => "Último servidor",
        "Previous Game ID" => "Último ID de partida",
        "Mapping" => "Mapeamento SDL do controle",
        "deadzone" => "Zona morta do controle",
        "Inventory" => "Inventário",
        "Character" => "Personagem",
        "QuestLog" => "Missões",
        "SpellBook" => "Livro de magias",
        "DisplaySpells" => "Lista de magias",
        "QuickSave" => "Salvar rapidamente",
        "QuickLoad" => "Carregar rapidamente",
        "UseHealthPotion" => "Usar poção de cura",
        "UseManaPotion" => "Usar poção de mana",
        "QuitGame" => "Sair do jogo",
        "StopHero" => "Parar personagem",
        "Item Highlighting" => "Destacar itens",
        "Toggle Item Highlighting" => "Alternar destaque dos itens",
        "Toggle Automap" => "Mapa",
        "Hide Info Screens" => "Fechar telas de informação",
        "Pause Game" => "Pausar",
        "Pause Game (Alternate)" => "Pausa alternativa",
        "DecreaseGamma" => "Diminuir brilho",
        "IncreaseGamma" => "Aumentar brilho",
        "Help" => "Ajuda",
        "Screenshot" => "Captura de tela",
        "GameInfo" => "Informações da partida",
        "ChatLog" => "Histórico do chat",
        "PrimaryAction" => "Ação principal",
        "SecondaryAction" => "Ação secundária",
        "SpellAction" => "Lançar magia",
        "CancelAction" => "Cancelar",
        "MoveUp" => "Mover para cima",
        "MoveDown" => "Mover para baixo",
        "MoveLeft" => "Mover para a esquerda",
        "MoveRight" => "Mover para a direita",
        "StandGround" => "Manter posição",
        "ToggleStandGround" => "Alternar posição fixa",
        "MouseUp" => "Mouse para cima",
        "MouseDown" => "Mouse para baixo",
        "MouseLeft" => "Mouse para a esquerda",
        "MouseRight" => "Mouse para a direita",
        "LeftMouseClick1" => "Clique esquerdo",
        "LeftMouseClick2" => "Clique esquerdo alternativo",
        "RightMouseClick1" => "Clique direito",
        "RightMouseClick2" => "Clique direito alternativo",
        "PadHotspellMenu" => "Menu rápido de magias",
        "PadMenuNavigator" => "Navegação dos menus",
        "ToggleGameMenu1" => "Menu do jogo",
        "ToggleGameMenu2" => "Menu do jogo alternativo",
        _ => "",
    };
    if !translated.is_empty() {
        return translated.into();
    }
    for (prefix, title) in [
        ("BeltItem", "Usar item do cinto"),
        ("QuickSpell", "Magia rápida"),
        ("QuickMessage", "Mensagem rápida"),
    ] {
        if let Some(index) = key.strip_prefix(prefix) {
            return format!("{title} {index}");
        }
    }
    key.into()
}
