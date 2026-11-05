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
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Game => "Game",
            Self::Interface => "Interface",
            Self::Keyboard => "Keyboard",
            Self::Controller => "Controller",
            Self::Network => "Network",
            Self::Startup => "Startup",
            Self::Other => "Other options",
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
            ("0", "Nearest pixel"),
            ("1", "Bilinear"),
            ("2", "Anisotropic"),
        ]),
        ("Graphics", "Hardware Cursor Maximum Size") => Kind::Choices(&[
            ("0", "Unlimited"),
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
        ("Audio", "Channels") => Kind::Choices(&[("1", "Mono"), ("2", "Stereo")]),
        ("Audio", "Buffer Size") => {
            Kind::Choices(&[("1024", "1024"), ("2048", "2048"), ("5120", "5120")])
        }
        ("Audio", "Resampler") => Kind::Choices(&[("Speex", "Speex"), ("SDL", "SDL")]),
        ("Audio", "Resampling Quality") => Kind::Integer(0, 10),
        ("Audio", "Device") => Kind::Text,
        ("Game", "Speed") => Kind::Integer(1, 1000),
        ("Game", "Enable floating numbers") => {
            Kind::Choices(&[("0", "Off"), ("1", "Random angles"), ("2", "Vertical only")])
        }
        ("Game", key) if key.ends_with("Potion Pickup") => Kind::Choices(&[
            ("0", "Do not collect"),
            ("1", "1"),
            ("2", "2"),
            ("4", "4"),
            ("8", "8"),
            ("16", "16"),
        ]),
        ("StartUp", "Diablo Intro" | "Hellfire Intro") => {
            Kind::Choices(&[("0", "Never"), ("1", "Once"), ("2", "Always")])
        }
        ("StartUp", "Splash") => Kind::Choices(&[
            ("0", "None"),
            ("1", "Title screen"),
            ("2", "Logo and title"),
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
        Err(format!("Invalid value for {}.", label(option)))
    }
}

pub fn label(option: &ExtraOption) -> String {
    let key = option.key.as_str();
    match key {
        "MultiplayerFullQuests" => return "Full quests in multiplayer".into(),
        "Speed" => return "Game speed (ticks per second)".into(),
        "Device" => return "Output device (blank for default)".into(),
        "deadzone" => return "Controller deadzone".into(),
        "Mapping" => return "SDL controller mapping".into(),
        "Test Bard" => return "Enable Bard".into(),
        "Test Barbarian" => return "Enable Barbarian".into(),
        _ => {}
    }
    for (prefix, title) in [
        ("BeltItem", "Use belt item"),
        ("QuickSpell", "Quick spell"),
        ("QuickMessage", "Quick message"),
    ] {
        if let Some(index) = key.strip_prefix(prefix) {
            return format!("{title} {index}");
        }
    }
    if key.contains(' ') {
        return key.into();
    }
    let mut label = String::new();
    for (index, ch) in key.chars().enumerate() {
        if index > 0 && ch.is_uppercase() {
            label.push(' ');
            label.extend(ch.to_lowercase());
        } else {
            label.push(ch);
        }
    }
    label
}
