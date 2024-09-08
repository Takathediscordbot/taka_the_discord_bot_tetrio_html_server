use std::sync::Arc;

use axum::{response::{IntoResponse, Html, Response}, extract::{State, Query}};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;
use common::{LeagueRecord, Averages, Average, Round, Stats, LeagueRecordRequest};
use itertools::Itertools;
use reqwest::StatusCode;

use crate::AppState;

#[derive(Deserialize)]
pub struct TetraParam {
    user_id: String,
    game_num: usize
}

#[derive(Deserialize)]
pub struct TetraTestParam {
    left_score: Option<u32>,
    right_score: Option<u32>,
}

#[derive(Deserialize)]
pub struct ReplayParam {
    replay_id: String,
    user_id: String
}

const TETRA_HTML_FILE: &str = include_str!("../assets/tetra/index.html");
const TETRA_HTML_MATCH: &str = "<div class=\"multilog_result scroller_block zero\" data-hover=\"tap\" data-hit=\"click\">
                        <div class=\"multilog_result_self {left_success}\"><span>{left_pps}</span> PPS - <span>{left_apm}</span> APM -
                            <span>{left_vs}</span> VS</div>
                        <div class=\"multilog_result_time\">{time}</div>
                        <div class=\"multilog_result_opponent {right_success}\"><span>{right_pps}</span> PPS - <span>{right_apm}</span> APM -
                            <span>{right_vs}</span> VS</div>
                        </div>";
const TETRA_EXTRA_HTML: &str = r#"<span>{pps}</span> PPS - <span>{apm}</span> APM -
<span>{vs}</span> VS"#;

#[derive(Default)]
struct TetraHtmlMatch {
    pub left_success: bool,
    pub left_pps: f64,
    pub left_apm: f64,
    pub left_vs: f64,
    pub time: String,
    pub right_success: bool,
    pub right_pps: f64,
    pub right_apm: f64,
    pub right_vs: f64,
}

impl TetraHtmlMatch {


    pub fn into_html_page(self) -> String {
        // use replacen instead of replace to specify the number of replacements
        let TetraHtmlMatch {
            left_success,
            left_pps,
            left_apm,
            left_vs,
            time,
            right_success,
            right_pps,
            right_apm,
            right_vs,
        } = self;

        TETRA_HTML_MATCH
            .replacen("{left_success}", if left_success { "success" } else { "" }, 1)
            .replacen("{left_pps}", &format!("{:.2}", left_pps), 1)
            .replacen("{left_apm}", &format!("{:.2}", left_apm), 1)
            .replacen("{left_vs}", &format!("{:.2}", left_vs), 1)
            .replacen("{time}", &time, 1)
            .replacen("{right_success}", if right_success { "success" } else { "" } , 1)
            .replacen("{right_pps}", &format!("{:.2}", right_pps), 1)
            .replacen("{right_apm}", &format!("{:.2}", right_apm), 1)
            .replacen("{right_vs}", &format!("{:.2}", right_vs), 1)
    }
}

impl From<Round> for TetraHtmlMatch {
    fn from(round: Round) -> Self {
        let Round {
            left,
            right,
            time,
        } = round;

        let Stats {
            pps: left_pps,
            apm: left_apm,
            vs: left_vs,
            success: left_success,
        } = left;

        let Stats {
            pps: right_pps,
            apm: right_apm,
            vs: right_vs,
            success: right_success,
        } = right;

        Self {
            left_success,
            left_pps,
            left_apm,
            left_vs,
            time,
            right_success,
            right_pps,
            right_apm,
            right_vs,
        }
    }
}


#[derive(Default)]
struct TetraHtmlPage {
    pub matches: Vec<TetraHtmlMatch>,
    pub left_username: String,
    pub right_username: String,
    pub left_score: u32,
    pub right_score: u32,
    pub left_pps: f64,
    pub right_pps: f64,
    pub left_apm: f64,
    pub right_apm: f64,
    pub left_vs: f64,
    pub right_vs: f64,
    pub played_date: String,
    pub played_time: String,
}


impl TetraHtmlPage {
    pub fn into_html(self) -> String {
        let TetraHtmlPage {
            matches,
            left_username,
            right_username,
            left_score,
            right_score,
            left_pps,
            right_pps,
            left_apm,
            right_apm,
            left_vs,
            right_vs,
            played_date,
            played_time,
        } = self;

        let mut html = TETRA_HTML_FILE.to_string();
        let left_extra = TETRA_EXTRA_HTML.replacen("{pps}", &format!("{:.2}", left_pps), 1)
            .replacen("{apm}", &format!("{:.2}", left_apm), 1)
            .replacen("{vs}", &format!("{:.2}", left_vs), 1);
        let right_extra = TETRA_EXTRA_HTML.replacen("{pps}", &format!("{:.2}", right_pps), 1);
        let right_extra = right_extra.replacen("{apm}", &format!("{:.2}", right_apm), 1);
        let right_extra = right_extra.replacen("{vs}", &format!("{:.2}", right_vs), 1);


        // Replace placeholders with actual values
        html = html.replace("{left_username}", &left_username);
        html = html.replace("{right_username}", &right_username);
        html = html.replacen("{left_score}", &left_score.to_string(), 1);
        html = html.replacen("{right_score}", &right_score.to_string(), 1);
        html = html.replacen("{left_extra}", &left_extra, 1);
        html = html.replacen("{right_extra}", &right_extra, 1);
        html = html.replacen("{played_date}", &played_date, 1);
        html = html.replacen("{played_time}", &played_time, 1);
        html = html.replacen("{matches}", &matches.into_iter().map(|m| m.into_html_page()).join("\n"), 1);

        html
    }

    pub fn from_league_record(league_record: LeagueRecord, timestamp: DateTime<Utc>) -> Self {
        let LeagueRecord {
            averages,
            rounds
        } = league_record;

        let Averages {
            left,
            right,
        } = averages;

        let Average {
            username: left_username,
            pps: left_pps,
            apm: left_apm,
            vs: left_vs,
            score: left_score,
        } = left;

        let Average {
            username: right_username,
            pps: right_pps,
            apm: right_apm,
            vs: right_vs,
            score: right_score,
            
        } = right;

        let played_date = timestamp.format("%d/%m/%Y").to_string();
        let played_time = timestamp.format("%H:%M:%S").to_string();

        let rounds = rounds.into_iter().map(|r| r.into()).collect::<Vec<TetraHtmlMatch>>();
    
        Self {
            matches: rounds,
            left_username,
            right_username,
            left_score,
            right_score,
            left_pps,
            right_pps,
            left_apm,
            right_apm,
            left_vs,
            right_vs,
            played_date,
            played_time,
        }
    }
}




pub fn generate_league_recent(league_record: LeagueRecord, timestamp: DateTime<Utc>) -> String {
    TetraHtmlPage::from_league_record(league_record, timestamp).into_html()
}

pub async fn league_recent_test(Query(replay_data): Query<TetraTestParam>) -> impl IntoResponse {
    Html(generate_league_recent(LeagueRecord 
        { 
            averages: Averages { 
                left: Average {
                    username: "\u{200B}".to_string(),
                    pps: 10.0, 
                    apm: 100.0, 
                    vs: 1.0, 
                    score: replay_data.left_score.unwrap_or(5)
                }, right: Average { 
                    username: "\u{200B}".to_string(),
                    pps: 10.0, 
                    apm: 50.0, 
                    vs: 100.0, 
                    score: replay_data.right_score.unwrap_or(5)
                } 
            }, 
            rounds: vec![]
        }, 
        chrono::offset::Utc::now()
    )).into_response()
}

#[derive(Deserialize)]
pub struct LeagueReplayQuery {
    data: String
}

pub async fn league_replay_from_data(Query(data): Query<LeagueReplayQuery>) -> Response {
    let data = data.data;
    let Ok(data) = urlencoding::decode(&data) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't decode data").into_response()
    };

    let Ok(data) = serde_json::from_str::<LeagueRecordRequest>(&data) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't parse data").into_response()
    };

    let LeagueRecordRequest {
        league_record,
        ts,
    } = data;
    
    Html(dbg!(generate_league_recent(league_record, DateTime::parse_from_rfc3339(&ts).unwrap_or_else(|_| chrono::offset::Utc::now().into()).with_timezone(&chrono::offset::Utc)))).into_response()
}

// basic handler that responds with a static string
pub async fn league_recent(State(state): State<Arc<AppState<'_>>>, Query(user_id): Query<TetraParam>) -> Response {
    let Ok(packet) = state.api_http_client.fetch_user_personal_league_records (&user_id.user_id, tetrio_api::http::parameters::personal_user_records::PersonalLeaderboard::Recent, tetrio_api::http::parameters::personal_user_records::PersonalRecordsQuery::NotBound { limit: Some(10) }).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't fetch or parse data").into_response()
    };
    
    let Some(data) = packet.data else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Tetrio server error").into_response()
    };

    let Some(record) = data.entries.get(user_id.game_num - 1) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "No recent records").into_response()
    };

    generate_league_replay(state, &record.replayid, &user_id.user_id).await
}

pub async fn league_replay(State(state): State<Arc<AppState<'_>>>, Query(replay_data): Query<ReplayParam>) -> Response {
    generate_league_replay(state, &replay_data.replay_id, &replay_data.user_id).await
}

async fn generate_league_replay(state: Arc<AppState<'_>>, replay_id: &str, user_id: &str) -> Response {
    
    let replay_data = state.tetrio_http_client.fetch_tetrio_replay(replay_id, &state.tetrio_token).await;
    dbg!(&replay_data);
    let replay_data = match replay_data {
        Ok(replay_data) => replay_data,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Couldn't fetch replay data: {e}")).into_response(),
    };

    let Some(data) = &replay_data.game else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't fetch replay data (game)").into_response()
    };

    let (Some(left), Some(right)) = (        
        data.results.leaderboard.iter().find(|f| f.id.clone().unwrap_or("".into()) == user_id), data.results.leaderboard.iter().find(|f| f.id.clone().unwrap_or("".into()) != user_id)) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse data (couldn't find end contexts)").into_response()
    };
    

    let league_record = LeagueRecord {
        averages: Averages {
            left: Average {
                username: left.username.clone().unwrap_or(String::new()),
                pps: left.stats.pps,
                apm: left.stats.apm,
                vs: left.stats.vsscore,
                score: left.wins as u32,
            },
            right: Average {
                username: right.username.clone().unwrap_or(String::new()),
                pps: right.stats.pps,
                apm: right.stats.apm,
                vs: right.stats.vsscore,
                score: right.wins as u32,
            }
        },
        rounds: data.results.rounds.iter().filter_map(|data| {
            let frame = data.iter().map(|f| f.lifetime).max().unwrap_or(0);
            let frames = frame as u64;

            let duration = Duration::from_millis(frames as u64);
            
            let minutes = duration.as_secs() / 60;
            let seconds = duration.as_secs() % 60;

            let left = data.iter().find(|f| f.id.clone().unwrap_or(String::new()) == user_id);
            let right = data.iter().find(|f| f.id.clone().unwrap_or(String::new()) != user_id);

            if let (Some(left), Some(right)) = (left, right) {

            Some(Round { 
                    left: Stats {
                        pps: left.stats.pps,
                        apm: left.stats.apm,
                        vs: left.stats.vsscore,
                        success: left.alive
                    }, 
                    right: Stats {
                        pps: right.stats.pps,
                        apm: right.stats.apm,
                        vs: right.stats.vsscore,
                        success: right.alive
                    }, 
                    time: format!("{minutes}:{seconds:02}")
                
                })
            } else {
                None
            }
        }).collect(),
    };

    Html(dbg!(generate_league_recent(league_record, data.ts))).into_response()
}
