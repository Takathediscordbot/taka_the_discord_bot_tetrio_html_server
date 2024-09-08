mod teto;
pub mod tetra;

use common::Error;

use std::{time::Duration, sync::Arc, fs::DirEntry};

use axum::{
    response::IntoResponse,
    routing::get, Router, extract::State,
};


use serde::{Deserialize, Serialize};
use tetrio_api::{http::clients::reqwest_client::{RedisReqwestClient, ReqwestClient}, models::common::{APIfloat, APIint, APIstring}};
use itertools::Itertools;

use moka::future::Cache;

use crate::tetra::{league_recent_test, league_recent, league_replay, league_replay_from_data};



struct TetrioCachedClient {
   tetrio_replays_cache: Cache<Box<str>, Arc<GameReplayPacket>>,
}

impl Default for TetrioCachedClient {
    fn default() -> Self {
        Self { tetrio_replays_cache: Cache::builder().time_to_live(Duration::from_secs(15 * 60)).build()}
    }
}

impl TetrioCachedClient {
    pub async fn me(&self,  tetrio_token: &str) -> anyhow::Result<String> {
        let result =reqwest::Client::new()
        .get(&format!("https://tetr.io/api/users/me"))
        .header("Authorization", tetrio_token)
        .header("Accept", "application/json")
        .send()
        .await?
        .text()
        .await?;

        return Ok(result)
    }

    pub async fn fetch_tetrio_replay(&self, replay_id: &str, tetrio_token: &str) -> anyhow::Result<Arc<GameReplayPacket>> {
        let replay_id = replay_id.to_string().into_boxed_str();
        if let Some(data) = self.tetrio_replays_cache.get(&replay_id).await {
            return Ok(Arc::clone(&data));
        }

        let result =reqwest::Client::new()
            .get(&format!("https://tetr.io/api/games/{}", replay_id))
            .header("Authorization", tetrio_token)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<GameReplayPacket>()
            .await
            .map(Arc::new)
            ?;

        self.tetrio_replays_cache.insert(replay_id, Arc::clone(&result)).await;
        Ok(result)
    }
}


pub struct AppState<'a> {
    tetrio_token: String,
    tetrio_http_client: Arc<TetrioCachedClient>,
    api_http_client: Arc<RedisReqwestClient<'a>>,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    dotenvy::dotenv().expect("Couldn't read .env file");
    tracing_subscriber::fmt::init();

    let ip_bind = std::env::var("BIND_URL").unwrap_or("0.0.0.0:80".to_string());
    println!("{ip_bind}");
    let tetrio_token = std::env::var("TETRIO_API_TOKEN").expect("Couldn't get tetrio token");
    let redis_url = std::env::var("REDIS_URL").expect("Couldn't get tetrio token");
    let client = redis::Client::open(redis_url)?;
    let state = AppState {tetrio_token, tetrio_http_client: Default::default(), api_http_client: Arc::new(
        RedisReqwestClient::new(
            ReqwestClient::default(),
            tetrio_api::http::caches::redis_cache::RedisCache { client: std::borrow::Cow::Owned(client) }
        ))
    };

    tokio::spawn(async {
        let ip_bind = std::env::var("HEALTH_URL").unwrap_or("0.0.0.0:8080".to_string());
        println!("{ip_bind}");
    
        // let origins = [
        //     "https://health.takathedinosaur.tech/".parse().unwrap()
        // ];
        // build our application with a route
        let app = Router::new()
        .route("/health", axum::routing::get(health_status))
        .route("/assets", axum::routing::get(assets_folder));
        
    
        // run our app with hyper
        let listener = tokio::net::TcpListener::bind(&ip_bind).await.map_err(|e| {
            Error(format!("Couldn't bind to address {ip_bind}: {e}"))
        });

        match listener {
            // run our app with hyper
            Ok(listener) => {
                let _ = axum::serve(listener, app)
                    .await;
            },
            Err(e) => {
                eprintln!("{e:?}");
            }
        }

    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route_service("/tetra/hun2.ttf", tower_http::services::ServeFile::new("./assets/tetra/hun2.ttf"))
        .route_service("/teto/hun2.ttf", tower_http::services::ServeFile::new("./assets/teto/hun2.ttf"))

        .route_service("/teto/unkown_avatar.webp", tower_http::services::ServeFile::new("./assets/teto/unkown_avatar.webp"))
        .route("/login", get(try_login))
        .route("/league_recent_test", get(league_recent_test))
        .route("/league_recent", get(league_recent))
        .route("/league_replay", get(league_replay))
        .route("/league_replay_from_data", get(league_replay_from_data))

        .route("/teto_test/:user_id", get(teto::teto_test))
        
        .with_state(Arc::new(state))
        ;
    // run our app with hyper
    let listener = tokio::net::TcpListener::bind(&ip_bind).await.map_err(|e| {
        anyhow::anyhow!(Error(format!("Couldn't bind to address {ip_bind}: {e}")))
    })?;

    // run our app with hyper
    let _ = axum::serve(listener, app)
        .await;

    Ok(())
}


async fn health_status() -> impl IntoResponse {
    "OK"
}

async fn try_login(State(state): State<Arc<AppState<'_>>>) -> impl IntoResponse {
    format!("{:?}", state.tetrio_http_client.me(&state.tetrio_token).await)
}

fn read_dir_entry(dir_entry: &DirEntry) -> String {
    match dir_entry.file_type() {
        Err(_) => String::from("(couldn't read file type)"),
        Ok(f) => match (f.is_dir(), f.is_file()) {
            (true, false) => {
                match std::fs::read_dir(dir_entry.path()) {
                    Ok(directory) => {
                        directory.map(|f| {
                            match f {
                                Ok(f) => read_dir_entry(&f), 
                                Err(_) => String::from("(couldn't read file)")
                            }
                        }).join("\n")
            
                    },
                    Err(_) => return String::from("Couldn't read directory!"),
                }
            },
            (false, true) => {
                dir_entry.path().to_string_lossy().to_string()
            },
            _ => String::from("Unhandled file type")
        }
    }
}

async fn assets_folder() -> impl IntoResponse {
    match std::fs::read_dir("./assets") {
        Ok(directory) => {
            directory.map(|f| {
                match f {
                    Ok(f) => read_dir_entry(&f), 
                    Err(_) => String::from("(couldn't read file)")
                }
            }).join("\n")

        },
        Err(_) => return String::from("Couldn't read directory!"),
    }
}




  






#[derive(Deserialize, Serialize, Clone)]
pub struct GameReplayGameBoardUser {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String
}

#[derive(Deserialize, Clone)]
pub struct GameReplayGameBoard {
    pub user: Option<GameReplayGameBoardUser>,
    pub id: Option<String>,
    pub username: Option<String>,
    #[allow(unused)]
    pub active: bool,
    pub success: bool
}

impl GameReplayGameBoard {
    pub fn get_id(&self) -> Option<String> {
        return self.id.clone().or(self.user.clone().map(|user| user.id))
    }

    pub fn get_username(&self) -> Option<String> {
        return self.username.clone().or(self.user.clone().map(|user| user.username))
    }
}



#[derive(Deserialize, Clone, Debug)]
struct GameReplayData {
    ts: chrono::DateTime<chrono::Utc>,
    results: LeagueEndContext
}

#[derive(Deserialize, Debug)]
struct GameReplayPacket {
    #[allow(unused)]
    success: bool,
    game: Option<GameReplayData>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeagueEndContextUser {
    pub id: APIstring,
    pub username: APIstring
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeagueEndContext {
   pub leaderboard: Vec<LeagueEndContextLeaderboard>,
   pub rounds: Vec<Vec<LeagueEndContextRound>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeagueEndContextLeaderboard {
    pub user: Option<LeagueEndContextUser>,

    pub id: Option<APIstring>,
    pub username: Option<APIstring>,
    pub active: bool,
    pub inputs: Option<APIint>,
    pub natural_order: Option<APIfloat>,
    pub wins: APIint,
    pub stats: LeagueEndContextStats
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeagueEndContextStats {
    pub apm: APIfloat,
    pub pps: APIfloat,
    pub vsscore: APIfloat,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeagueEndContextRound {
    pub id: Option<APIstring>,
    pub username: Option<APIstring>,
    pub stats: LeagueEndContextStats,
    pub alive: bool,
    pub lifetime: APIint,
}








