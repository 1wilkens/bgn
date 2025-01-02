use std::collections::HashMap;
use std::fs;
use std::io;

use arnak::BoardGameGeekApi;
use csv::Reader;
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use tera::Tera;
use tokio::runtime;

#[derive(Debug, Deserialize)]
struct GameRow {
    id: String,
    bgg_id: u64,
}

#[derive(Debug, Serialize)]
struct Game {
    id: String,
    bgg_link: String,
    name: String,
    matches: Vec<Match>,
}

impl Game {
    fn from_row(gs: GameRow) -> Game {
        Game {
            id: gs.id.clone(),
            bgg_link: gs.bgg_id.to_string(),
            name: gs.id,
            matches: vec![],
        }
    }
}

#[derive(Debug, Serialize)]
struct Player {
    name: String,
    total_rank: u64,
}

#[derive(Debug, Deserialize)]
struct MatchRow {
    game: String,
    matchid: String,
    player: String,
    score: u64,
    rank: u8,
}

#[derive(Debug, Serialize)]
struct Score {
    player: String,
    score: u64,
}

impl Score {
    fn from_row(row: &MatchRow) -> Self {
        Self {
            player: row.player.clone(),
            score: row.score,
        }
    }
}

#[derive(Debug, Serialize)]
struct Match {
    game: String,
    //date: String,
    scores: Vec<Score>,
}

impl Match {
    fn from_row(row: &MatchRow) -> Self {
        Match {
            game: row.game.clone(),
            scores: vec![Score::from_row(row)],
        }
    }
}

fn load_games(file: &str) -> HashMap<String, Game> {
    let mut games = HashMap::new();
    let mut rdr = Reader::from_path(file).expect("Failed to read games file");
    let rt = runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let api = BoardGameGeekApi::new();
        let api_game = api.game();
        for res in rdr.deserialize::<GameRow>() {
            if let Ok(gs) = res {
                //let resp = blocking::get(&gs.bgg_link).expect("Failed to fetch bgg game data");
                //let body = resp.text().expect("Failed to parse bgg body data");
                //dbg!(body);
                dbg!(gs.bgg_id);
                let details = api_game.get_by_id(gs.bgg_id, Default::default()).await;
                dbg!(details.unwrap());
                let game = Game::from_row(gs);
                games.insert(game.id.clone(), game);
            } else {
                println!("DBG: skipping invalid game line - {}", res.unwrap_err());
            }
        }
    });
    games
}

fn load_match<'a>(file: &str) -> Vec<Match> {
    let mut matches: HashMap<String, Match> = HashMap::new();
    let mut rdr = Reader::from_path(file).expect("Failed to read match file");
    for res in rdr.deserialize::<MatchRow>() {
        if let Ok(row) = res {
            dbg!(&row);
            let score = Score::from_row(&row);
            matches
                .entry(row.matchid.clone())
                .and_modify(|m| m.scores.push(score))
                .or_insert(Match::from_row(&row));
        }
    }
    matches.into_values().collect()
}

fn load_matches<'a>(data_dir: &str, games: &mut HashMap<String, Game>) -> Vec<Player> {
    let players = Vec::with_capacity(4);

    for et in fs::read_dir(&data_dir).expect("Failed to open data directory") {
        let path = et.expect("Failed to access dir entry").path();
        if path.is_file() {
            let matches = load_match(path.to_str().unwrap());
            for m in matches {
                games
                    .entry(m.game.clone())
                    .and_modify(|g| g.matches.push(m));
            }
        }
    }
    players
}

fn main() -> io::Result<()> {
    let mut games = load_games("example_data/games.csv");
    let players = load_matches("example_data/matches", &mut games);
    dbg!(&games);
    dbg!(&players);
    //dbg!(&matches);

    let games: Vec<_> = games.values().collect();

    let tera = Tera::new("templates/**/*").unwrap();
    let mut ctx = tera::Context::new();

    ctx.insert("games", &games);
    ctx.insert("players", &players);
    //ctx.insert("matches", &matches);

    let rendered = tera.render("index.html", &ctx).unwrap();
    //dbg!(&rendered);

    fs::create_dir_all("public")?;
    fs::remove_file("public/index.html")?;
    fs::write("public/index.html", rendered)?;

    Ok(())
}
