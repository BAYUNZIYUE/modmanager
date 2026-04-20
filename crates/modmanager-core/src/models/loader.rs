use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Forge,
    NeoForge,
    Cauldron,
    Fabric,
    Quilt,
    LiteLoader,
    Babric,
    BTA,
    LegacyFabric,
    Ornithe,
    Rift,
}

impl ModLoader {
    pub fn curseforge_id(self) -> i32 {
        match self {
            Self::Forge => 1,
            Self::Cauldron => 2,
            Self::LiteLoader => 3,
            Self::Fabric => 4,
            Self::Quilt => 5,
            Self::NeoForge => 6,
            _ => 0,
        }
    }

    pub fn modrinth_name(self) -> &'static str {
        match self {
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
            Self::Cauldron => "cauldron",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::LiteLoader => "liteloader",
            Self::Babric => "babric",
            Self::BTA => "bta",
            Self::LegacyFabric => "legacyfabric",
            Self::Ornithe => "ornithe",
            Self::Rift => "rift",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Forge => "Forge",
            Self::NeoForge => "NeoForge",
            Self::Cauldron => "Cauldron",
            Self::Fabric => "Fabric",
            Self::Quilt => "Quilt",
            Self::LiteLoader => "LiteLoader",
            Self::Babric => "Babric",
            Self::BTA => "BTA",
            Self::LegacyFabric => "Legacy Fabric",
            Self::Ornithe => "Ornithe",
            Self::Rift => "Rift",
        }
    }

    pub fn all_major() -> Vec<ModLoader> {
        vec![Self::Forge, Self::NeoForge, Self::Fabric, Self::Quilt]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModLoaderSet {
    Single(ModLoader),
    Multiple,
    Any,
}
