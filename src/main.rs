use std::fs;
use std::io;
use std::io::Read;

use csv::Reader;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(Debug, Deserialize)]
struct GameStub {
    id: String,
    bgg_link: String,
}

#[derive(Debug, Serialize)]
struct Game {
    id: String,
    bgg_link: String,
    name: String,
}

impl Game {
    fn from_stub(gs: GameStub) -> Game {
        Game {
            id: gs.id.clone(),
            bgg_link: gs.bgg_link,
            name: gs.id,
        }
    }
}

#[derive(Debug, Serialize)]
struct Player {
    name: String,
    total_score: u64,
}

#[derive(Debug, Serialize)]
struct Match<'a> {
    game: &'a Game,
    scores: Vec<(String, u64)>,
}

fn load_games(file: &str) -> Vec<Game> {
    let mut games = vec![];
    let mut rdr = Reader::from_path(file).expect("Failed to read file");
    for res in rdr.deserialize::<GameStub>() {
        if let Ok(gs) = res {
            let resp = blocking::get(&gs.bgg_link).expect("Failed to fetch bgg game data");
            let body = resp.text().expect("Failed to parse bgg body data");
            dbg!(body);
            games.push(Game::from_stub(gs));
        } else {
            println!("DBG: skipping invalid game line - {}", res.unwrap_err());
        }
    }
    games
}

fn load_matches<'a>(data_dir: &str, _games: &'a [Game]) -> (Vec<Player>, Vec<Match<'a>>) {
    let players = vec![];
    let matches = vec![];
    (players, matches)
}

fn main() -> io::Result<()> {
    let games = load_games("example_data/games.csv");
    let (players, matches) = load_matches("example_data/matches", &games);

    let tera = Tera::new("templates/**/*").unwrap();
    let mut ctx = tera::Context::new();

    ctx.insert("games", &games);
    ctx.insert("players", &players);
    ctx.insert("matches", &matches);

    let rendered = tera.render("index.html", &ctx).unwrap();
    dbg!(&rendered);

    fs::create_dir_all("public")?;
    fs::remove_file("public/index.html")?;
    fs::write("public/index.html", rendered)?;

    Ok(())
}
