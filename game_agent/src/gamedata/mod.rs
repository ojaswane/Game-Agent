#[derive(Debug, clone)]
pub Struct GameData{
    pub id: &'static str,
    pub name: &'static str,
    pub enabled: bool,
}

pub fn find_game(game_id: &str) -> Option<GameData> {
    match game_id {
        "rdr2" => Some(GameDefinition {
            id: "rdr2",
            name: "Red Dead Redemption 2",
            enabled: true,
        }),
        _ => None,
    }
}