use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
    sync::LazyLock,
};

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

static KNOWN_TOOLS: LazyLock<BTreeMap<String, ToolId>> = LazyLock::new(|| {
    let mut set = BTreeSet::new();
    let mut map = BTreeMap::new();

    for (author, tools) in KNOWN_TOOL_AUTHORS_AND_IDS {
        for tool_cased in tools {
            let tool_uncased = tool_cased.to_ascii_lowercase();
            assert!(
                !set.contains(&tool_uncased),
                "Duplicate known tool: {tool_uncased}"
            );
            let id = ToolId::from_str(&format!("{author}/{tool_cased}"))
                .expect("Known tool id should be valid");
            map.insert(tool_uncased.clone(), id);
            set.insert(tool_uncased);
        }
    }

    map
});

pub fn get_known_tool(tool: impl AsRef<str>) -> Option<ToolId> {
    let tool = tool.as_ref().to_ascii_lowercase();
    KNOWN_TOOLS.get(tool.as_str()).cloned()
}
