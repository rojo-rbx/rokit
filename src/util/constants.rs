use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use once_cell::sync::Lazy;
use rokit::tool::ToolId;

const KNOWN_TOOL_AUTHORS_AND_IDS: [(&str, &[&str]); 8] = [
    ("evaera", &["moonwave"]),
    ("Iron-Stag-Games", &["lync"]),
    (
        "JohnnyMorganz",
        &["luau-lsp", "StyLua", "wally-package-types"],
    ),
    ("Kampfkarren", &["selene"]),
    ("luau-lang", &["luau"]),
    ("lune-org", &["lune"]),
    ("rojo-rbx", &["remodel", "rojo", "tarmac"]),
    ("UpliftGames", &["wally"]),
];

pub static KNOWN_TOOLS: Lazy<BTreeMap<&'static str, ToolId>> = Lazy::new(|| {
    let mut set = BTreeSet::new();
    let mut map = BTreeMap::new();

    for (author, tools) in KNOWN_TOOL_AUTHORS_AND_IDS {
        for tool in tools {
            assert!(!set.contains(tool), "Duplicate known tool id: {tool}");
            let id = ToolId::from_str(&format!("{author}/{tool}"))
                .expect("Known tool id should be valid");
            map.insert(*tool, id);
            set.insert(*tool);
        }
    }

    map
});
