use std::{time::Duration, sync::Arc, str::FromStr};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response, Html},
    routing::get, Router, extract::{Query, State, Path},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use common::{Average, Averages, Stats, LeagueRecord, Round};
use tetrio_api::{http, models::{streams::league_stream::LeagueEndContext, users::{user_rank::UserRank, user_role::UserRole, user_info::UserInfoPacketData}}};

const TETRA_HTML_FILE: &str = include_str!("../assets/tetra/index.html");
const TETRA_HTML_MATCH: &str = "<div class=\"multilog_result scroller_block zero\" data-hover=\"tap\" data-hit=\"click\">
                        <div class=\"multilog_result_self {left_success}\"><span>{left_pps}</span> PPS - <span>{left_apm}</span> APM -
                            <span>{left_vs}</span> VS</div>
                        <div class=\"multilog_result_time\">{time}</div>
                        <div class=\"multilog_result_opponent {right_success}\"><span>{right_pps}</span> PPS - <span>{right_apm}</span> APM -
                            <span>{right_vs}</span> VS</div>
                    </div>";


const TETO_HTML_BOT_FILE: &str = include_str!("../assets/teto/bot.html");
const TETO_HTML_BANNED_FILE: &str = include_str!("../assets/teto/banned.html");
const TETO_HTML_FILE: &str = include_str!("../assets/teto/index.html");
const TETO_VERIFIED_BADGE: &str = "<img src=\"https://tetr.io/res/verified-light.png\" title=\"Verified`\">";
const TETO_HTML_FLAG: &str = "<img class=\"flag\" src=\"https://tetr.io/res/flags/{{country_code}}.png\">";
const TETO_HTML_BANNER: &str = "<img class=\"tetra_modal_banner ns\" src=\"{{banner_url}}\">";
const TETO_HTML_MOD_BADGE: &str = "<img class=\"mod_badge\" src=\"{{mod_icon}}\" title=\"This person has unlimited permissions on TETR.IO.\" alt=\"Sysop\">";
const TETO_HTML_BANNER_SEP: &str = "<div class=\"tetra_modal_banner_sep ns\"></div>";
const TETO_HTML_GAME_TIME: &str = "<div class=\"tetra_tag_gametime\" title=\"Total time played\">{{time}}<span>{{unit}}</span></div>";
const TETO_HTML_SUPPORTER: &str = "<img class=\"supporter_badge\" src=\"https://tetr.io/res/supporter{{supporter_tier}}.png\" title=\"This person is supporting TETR.IO ♥\" alt=\"Supporter\">";
const TETO_HTML_RECORDS: &str = "<div class=\"tetra_modal_records flex-row\">
{{tetra_league}}
{{sprint}}
{{blitz}}";

const TETO_HTML_RECORDS_TETRA_LEAGUE: &str = "<div class=\"tetra_modal_record flex-item tetra_modal_record_league tetra_modal_record_league_active\">
<h6>TETRA LEAGUE</h6>
<h5 title=\"22943.836819608496\"><img src=\"{{rank}}\">{{tr}}<span class=\"ms\">TR</span>
    {{standing_set}}
</h5>
<h3><span>{{apm}}</span> apm <span>{{pps}}</span> pps <span>{{vs}}</span> vs</h3>
</div>";

const TETO_HTML_RECORDS_TETRA_LEAGUE_RATING: &str = "<div class=\"tetra_modal_record flex-item tetra_modal_record_league\"><h6>TETRA LEAGUE</h6><h5>{{games_played}}<span class=\"ms\">/10 rating games</span></h5><h3><span>{{games_won}}</span> game won</h3></div>";

const TETO_HTML_RECORDS_TETRA_LEAGUE_STANDING_SET: &str = "<div class=\"standingset\"><div class=\"{{global_ranking_class}}\">
<h1>GLOBAL</h1>
<p><span>#</span>{{global_ranking}}</p>
</div>
<div class=\"{{country_ranking_class}}\">
<h1>COUNTRY</h1><p><span>#</span>{{country_ranking}}
</p></div>";

const TETO_HTML_RECORDS_STANDING_SET: &str = "<div class=\"standingset\"><div class=\"{{global_ranking_class}}\">
<h1>GLOBAL</h1>
<p><span>#</span>{{global_ranking}}</p>
</div>
</div>";

const TETO_HTML_RECORDS_SPRINT: &str = "<div class=\"tetra_modal_record flex-item\">
<h6>40 LINES</h6>
<h5>{{sprint_time}}<span class=\"ms\">.{{sprint_time_ms}}</span>
    {{standing_set}}
</h5>
<h3><span>{{date}}</span> ago</h3>
</div>";

const TETO_HTML_RECORDS_BLITZ: &str = "<div class=\"tetra_modal_record flex-item\">
<h6>BLITZ</h6>
<h5>{{blitz_score}}{{standing_set}}
</h5>
<h3><span>{{date}}</span> ago</h3>
</div>
</div>";

const TETO_HTML_BADGES: &str = "<div class=\"tetra_badge_holder ns\">{{badges}}</div>";

const TETO_HTML_BADGE: &str = "<img
class=\"tetra_badge\" src=\"{{badge}}\" title=\"Huge Supporter\" style=\"--i: 0;\">";

const TETO_HTML_TETRA_LEAGUE_CHAMPION_DISTINGUISHMENT: &str = 
"<div class=\"tetra_distinguishment ns tetra_distinguishment_champion\" data-detail=\"league\"><h1>TETRA LEAGUE CHAMPION</h1></div>";
const TETO_HTML_SPRINT_CHAMPION_DISTINGUISHMENT: &str = 
"<div class=\"tetra_distinguishment ns tetra_distinguishment_champion\" data-detail=\"40l\"><h1>40 LINES CHAMPION</h1></div>";
const TETO_HTML_BLITZ_CHAMPION_CHAMPION_DISTINGUISHMENT: &str = "<div class=\"tetra_distinguishment ns tetra_distinguishment_champion\" data-detail=\"blitz\"><h1>BLITZ CHAMPION</h1></div>";
const TETO_HTML_STAFF_DISTINGUISHMENT: &str = "<div class=\"tetra_distinguishment ns tetra_distinguishment_staff\" data-detail=\"{{staff_type}}\">
<h1>{{title}}</h1>
{{teto_staff_subtitle}}
</div>";

const TETO_HTML_TOTAL_GAMES: &str = "<div class=\"tetra_tag_record\" title=\"Online games won / online games played\">
{{online_games_won}}{{online_games_played}}
</div>";

const TETO_HTML_GAMES_WON: &str = "<span>{{online_games_won}}</span>";

const TETO_HTML_BAD_STANDING: &str = "<div class=\"tetra_badstanding ns\"><h1>BAD STANDING</h1><p>one or more recent bans on record</p></div>";
const TETO_HTML_STAFF_DISTINGUISHMENT_SUBTITLE: &str = "<p>{{subtitle_text}}</p>";
const TETO_HTML_STAFF_DISTINGUISHMENT_TETRIO_LOGO: &str = "<img src=\"https://tetr.io/res/tetrio-logo.svg\" style=\"filter: invert(1);\">";
const TETO_HTML_STAFF_DISTINGUISHMENT_OSK: &str = "<img src=\"https://tetr.io/res/osk.svg\">";
use moka::future::Cache;

struct TetrioCachedClient {
   tetrio_replays_cache: Cache<Box<str>, Arc<GameReplayPacket>>
}

impl Default for TetrioCachedClient {
    fn default() -> Self {
        Self { tetrio_replays_cache: Cache::builder().time_to_live(Duration::from_secs(15 * 60)).build() }
    }
}

impl TetrioCachedClient {
    pub async fn fetch_tetrio_replay(&self, replay_id: &str, tetrio_token: &str) -> anyhow::Result<Arc<GameReplayPacket>> {
        let replay_id = replay_id.to_string().into_boxed_str();
        if let Some(data) = self.tetrio_replays_cache.get(&replay_id) {
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

#[derive(Clone)]
struct AppState {
    tetrio_token: String,
    tetrio_http_client: Arc<TetrioCachedClient>
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    dotenvy::dotenv().expect("Couldn't read .env file");
    tracing_subscriber::fmt::init();

    let ip_bind = std::env::var("BIND_URL").unwrap_or("0.0.0.0:8003".to_string());
    println!("{ip_bind}");
    let tetrio_token = std::env::var("TETRIO_API_TOKEN").expect("Couldn't get tetrio token");

    let state = AppState {tetrio_token, tetrio_http_client: Default::default()};

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route_service("/tetra/hun2.ttf", tower_http::services::ServeFile::new("./assets/tetra/hun2.ttf"))
        .route_service("/tetra/tetrio.css", tower_http::services::ServeFile::new("./assets/tetra/tetrio.css"))
        .route_service("/teto/hun2.ttf", tower_http::services::ServeFile::new("./assets/teto/hun2.ttf"))
        .route_service("/teto/tetrio.css", tower_http::services::ServeFile::new("./assets/teto/tetrio.css"))
        .route_service("/teto/unkown_avatar.webp", tower_http::services::ServeFile::new("./assets/teto/unkown_avatar.webp"))

        .route("/league_recent_test", get(league_recent_test))
        .route("/league_recent", get(league_recent))
        .route("/league_replay", get(league_replay))
        .route("/teto_test/:user_id", get(teto_test))
        .with_state(state)
        ;
    

    // run our app with hyper
    let _ = axum::Server::bind(&ip_bind.parse()?)
        .serve(app.into_make_service())
        .await;

    Ok(())
}


async fn root() -> impl IntoResponse {
    "Hello, World!"
}


fn level_from_xp(x: f64) -> f64 {
    (x / 500.0).powf(0.6) + x / (5000.0 + (f64::max(0.0, x - 4.0 * 10.0f64.powi(6)) / 5000.0)) + 1.0
}

async fn handle_banned(data: &UserInfoPacketData) -> impl IntoResponse {
    let avatar_rev = data.user.avatar_revision.unwrap_or(0);
    let avatar = if avatar_rev != 0 {
        format!("https://tetr.io/user-content/avatars/{}.jpg?rv={}", data.user.id, avatar_rev)
    }
    else {
        String::from("/teto/unkown_avatar.webp")
    };

    Html(
        TETO_HTML_BANNED_FILE.replacen("{{avatar}}",&avatar, 1)
        .replacen("{{username}}", &data.user.username.to_uppercase(), 1)
    )
}

async fn handle_bot(data: &UserInfoPacketData) -> impl IntoResponse {
    let bot_owner = match &data.user.botmaster {
        Some(data) => data.to_uppercase(),
        None => String::new()
    };
    let avatar_rev = data.user.avatar_revision.unwrap_or(0);
    let avatar = if avatar_rev != 0 {
        format!("https://tetr.io/user-content/avatars/{}.jpg?rv={}", data.user.id, avatar_rev)
    }
    else {
        String::from("/teto/unkown_avatar.webp")
    };

    Html(
        TETO_HTML_BOT_FILE.replacen("{{avatar}}",&avatar, 1)
        .replacen("{{username}}", &data.user.username.to_uppercase(), 1)
        .replacen("{{owner}}", &bot_owner, 1)   
    )
}
  

async fn teto_test(Path(user_id): Path<String>) -> impl IntoResponse {
    let user = match tetrio_api::http::client::fetch_user_info(&user_id).await {
        Ok(e) => e,
        Err(_) => return Html("<h1> Invalid user </h1>").into_response()
    };
    let data = match user.data {
        Some(e) => e,
        None => return Html("<h1> Invalid user </h1>").into_response()
    };

    if let UserRole::Banned = data.user.role {
        return handle_banned(&data).await.into_response()
    }

    if let UserRole::Bot = data.user.role {
        return handle_bot(&data).await.into_response();
    }

    let banner_rev = data.user.banner_revision.unwrap_or(0);
    let has_banner = banner_rev != 0 && data.user.supporter_tier != 0;

    let banner = if has_banner {
        TETO_HTML_BANNER.replacen("{{banner_url}}", &format!(
            "https://tetr.io/user-content/banners/{}.jpg?rv={}", data.user.id, banner_rev
        ), 1) + TETO_HTML_BANNER_SEP
    } else {
        String::new()
    };
    let bad_standing = if let Some(bad_standing) = data.user.badstanding {
        if bad_standing {
            TETO_HTML_BAD_STANDING
        }
        else {
            ""
        }
    } else {
        ""
    };
    
    let avatar_rev = data.user.avatar_revision.unwrap_or(0);
    let avatar = if avatar_rev != 0 {
        format!("https://tetr.io/user-content/avatars/{}.jpg?rv={}", data.user.id, avatar_rev)
    }
    else {
        String::from("/teto/unkown_avatar.webp")
    };

    let mod_badge = match data.user.role {
        tetrio_api::models::users::user_role::UserRole::Anon => String::new(),
        tetrio_api::models::users::user_role::UserRole::User => String::new(),
        tetrio_api::models::users::user_role::UserRole::Bot => String::new(),
        tetrio_api::models::users::user_role::UserRole::Banned => String::new(),
        tetrio_api::models::users::user_role::UserRole::Mod => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-mod.png"),
        tetrio_api::models::users::user_role::UserRole::Admin => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-admin.png"),
        tetrio_api::models::users::user_role::UserRole::SysOp => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-sysop.png"),
        tetrio_api::models::users::user_role::UserRole::HalfMod => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-halfmod.png"),
    };

    let staff_distinguishment = if let Some(distinguishment) = data.user.distinguishment {
        if let Some(detail) = distinguishment.detail {
            let header = distinguishment.header.unwrap_or(Arc::from(""))
                .replace("%tetrio%", TETO_HTML_STAFF_DISTINGUISHMENT_TETRIO_LOGO)
                .replace("%osk%", TETO_HTML_STAFF_DISTINGUISHMENT_OSK);

            let footer = if let Some(footer) = distinguishment.footer {
                TETO_HTML_STAFF_DISTINGUISHMENT_SUBTITLE.replacen("{{subtitle_text}}",
                &footer
                .replace("%tetrio%", TETO_HTML_STAFF_DISTINGUISHMENT_TETRIO_LOGO)
                .replace("%osk%", TETO_HTML_STAFF_DISTINGUISHMENT_OSK)
                , 1)
            }
            else {
                String::new()
            };


            Some(TETO_HTML_STAFF_DISTINGUISHMENT
                .replacen("{{staff_type}}", &detail, 1)
                .replacen("{{title}}", &header, 1)
                .replacen("{{teto_staff_subtitle}}", &footer, 1)
            )
        }
        else {
            None
        }
    } else {
        None
    };


    let verified = if data.user.verified {
        TETO_VERIFIED_BADGE
    }
    else {
        ""
    };
    
    let country_flag = if let Some(country) = &data.user.country {
        TETO_HTML_FLAG.replacen("{{country_code}}", &country.to_lowercase(), 1)
    }
    else {
        String::new()
    };

    let level = level_from_xp(data.user.xp) as usize;

    let leveltag = if level != 5000 {

        let shape_color = (level / 10) % 10;
        let shape = (level / 100) % 5;
        let badge_color = (level / 500) % 10;
        format!("lt_shape_{shape} lt_badge_color_{badge_color} lt_shape_color_{shape_color}") 
    }
    else {
        String::from("lt_golden")
    };

    let total_games = {
        if data.user.gamesplayed != -1 || data.user.gameswon != -1 {
            let games_won = if data.user.gameswon != -1 {
                TETO_HTML_GAMES_WON.replacen("{{online_games_won}}", &data.user.gameswon.to_string(), 1)
            } else {
                String::new()
            };

            let games_played = if data.user.gamesplayed != -1 {
                format!(" / {}", data.user.gamesplayed)
            } else {
                String::new()
            };

            TETO_HTML_TOTAL_GAMES.replacen("{{online_games_won}}", &games_won, 1).replacen("{{online_games_played}}", &games_played, 1)
        }
        else {
            String::new()
        }

    };

    let game_time = if data.user.gametime != -1.0 {
        let playtime = Duration::from_secs_f64(data.user.gametime);
        let seconds = playtime.as_secs();
        let (time, unit) = if seconds > 3600 {
            (seconds / 3600, "H")
        } else if seconds > 60 {
            (seconds / 60, "M")
        } else {
            (seconds, "S")
        };

        TETO_HTML_GAME_TIME.replacen("{{time}}", &time.to_string(), 1).replacen("{{unit}}", unit, 1)
    } else {
        String::new()
    };

    let supporter_badge = if data.user.supporter_tier != 0 {
        TETO_HTML_SUPPORTER.replacen("{{supporter_tier}}", &data.user.supporter_tier.to_string(), 1)
    }
    else {
        String::new()
    };

    let badges = if !data.user.badges.is_empty() {
        let badges = data.user.badges.iter().map(|e| {
            <&str>::clone(&TETO_HTML_BADGE).replacen("{{badge}}", &format!("https://tetr.io/res/badges/{}.png", e.id), 1)
        }).fold(String::new(), |g, next| {g + "\n" + &next});

        TETO_HTML_BADGES.replacen("{{badges}}", &badges, 1)
    } else {
        String::new()
    };

    let mut distinguishment = None;

    let tetra_league = if data.user.league.rating > 0.0 {


        let standing_set = if data.user.league.rank == UserRank::Z {
            String::new()
        }
        else if data.user.league.standing_local != -1 {
             
            let global_ranking = data.user.league.standing.to_string();
            let global_ranking_class = match data.user.league.standing {
                1 => {
                    distinguishment = Some(TETO_HTML_TETRA_LEAGUE_CHAMPION_DISTINGUISHMENT);
                    "t1"
                },
                2..=10 => "t10",
                11..=100 => "t100",
                _ => ""
            };

            let country_ranking = data.user.league.standing_local.to_string();
            let country_ranking_class = match data.user.league.standing_local {
                1 => "t1",
                2..=10 => "t10",
                11..=100 => "t100",
                _ => ""
            };


            TETO_HTML_RECORDS_TETRA_LEAGUE_STANDING_SET
                .replacen("{{global_ranking}}", &global_ranking, 1)
                .replacen("{{global_ranking_class}}", global_ranking_class, 1)
                .replacen("{{country_ranking}}", &country_ranking, 1)
                .replacen("{{country_ranking_class}}", country_ranking_class, 1)
        }
        else {
            let global_ranking = data.user.league.standing.to_string();
            let global_ranking_class = match data.user.league.standing {
                1 => {
                    distinguishment = Some(TETO_HTML_TETRA_LEAGUE_CHAMPION_DISTINGUISHMENT);
                    "t1"
                },
                2..=10 => "t10",
                11..=100 => "t100",
                _ => ""
            };

            TETO_HTML_RECORDS_STANDING_SET
                .replacen("{{global_ranking}}", &global_ranking, 1)
                .replacen("{{global_ranking_class}}", global_ranking_class, 1)
        };


        TETO_HTML_RECORDS_TETRA_LEAGUE
            .replacen("{{rank}}", &format!("https://tetr.io/res/league-ranks/{}.png", data.user.league.rank).to_lowercase(), 1)
            .replacen("{{tr}}", &(data.user.league.rating.round()).to_string(), 1)
            .replacen("{{vs}}", &format!("{:.2}", data.user.league.vs.unwrap_or(0.0)), 1)
            .replacen("{{apm}}", &format!("{:.2}", data.user.league.apm.unwrap_or(0.0)), 1)
            .replacen("{{pps}}", &format!("{:.2}", data.user.league.pps.unwrap_or(0.0)), 1)
            .replacen("{{standing_set}}", &standing_set, 1)
    }
    else if data.user.league.gamesplayed > 0 {
        TETO_HTML_RECORDS_TETRA_LEAGUE_RATING
            .replacen("{{games_played}}", &data.user.league.gamesplayed.to_string(), 1)
            .replacen("{{games_won}}", &data.user.league.gameswon.to_string(), 1)
    }
    else {
        String::new()
    };



    let username = data.user.username.to_uppercase();
    let records = match tetrio_api::http::client::fetch_user_records(&data.user.id).await {
        Ok(v) => v,
        Err(_) => return Html("<h1> Invalid user </h1>").into_response()
    };

    let friends = data.user.friend_count.unwrap_or(0);
    let joined_at = if let Some(ts) = data.user.ts {
        let time = chrono::DateTime::<Utc>::from_str(&ts).unwrap_or_default();

        let now = Utc::now();
        let duration = now.signed_duration_since(time);

        let ago = if duration.num_days() > 365 {
            format!("{} YEAR{}", (duration.num_days() / 365), if duration.num_days() - 365 >= 365 {
                "S"
            } else { ""})
        }
        else if duration.num_days() > 30 {
            format!("{} MONTH{}", (duration.num_days() / 30), if duration.num_days() - 30 >= 30 {
                "S"
            } else { ""})
        }
        else if duration.num_weeks() != 0 {
            format!("{} WEEK{}", (duration.num_weeks()), if duration.num_weeks() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_days() != 0 {
            format!("{} DAY{}", (duration.num_days()), if duration.num_days() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_hours() != 0 {
            format!("{} HOUR{}", (duration.num_hours()), if duration.num_hours() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_minutes() != 0 {
            format!("{} MINUTE{}", (duration.num_minutes()), if duration.num_minutes() > 1 {
                "S"
            } else { ""})
        }
        else {
            format!("{} SECOND{}", (duration.num_seconds()), if duration.num_seconds() > 1 {
                "S"
            } else { ""})
        };

        String::from("JOINED ") + &ago + " AGO - "        
    }
    else {
        String::from("HERE SINCE THE BEGINNING - ")
    };


    let data = match records.data {
        Some(v) => v,
        None => return Html("<h1> No user records </h1>").into_response(),
    };

    let sprint = if let Some(record) = data.records.sprint.record {
        if !record.is_null() {
        let obj = match record.as_object() {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        };

        let time = match obj.get("ts") {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        };
        let time = match time.as_str() {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        };

        let time = match chrono::DateTime::<Utc>::from_str(time){
            Ok(v) => v,
            Err(_) => return Html("<h1> parsing error </h1>").into_response()
        };

        let now = Utc::now();
        let duration = now.signed_duration_since(time);

        let ago = if duration.num_days() > 365 {
            format!("{} YEAR{}", (duration.num_days() / 365), if duration.num_days() - 365 >= 365 {
                "S"
            } else { ""})
        }
        else if duration.num_days() > 30 {
            format!("{} MONTH{}", (duration.num_days() / 30), if duration.num_days() - 30 >= 30 {
                "S"
            } else { ""})
        }
        else if duration.num_weeks() != 0 {
            format!("{} WEEK{}", (duration.num_weeks()), if duration.num_weeks() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_days() != 0 {
            format!("{} DAY{}", (duration.num_days()), if duration.num_days() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_hours() != 0 {
            format!("{} HOUR{}", (duration.num_hours()), if duration.num_hours() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_minutes() != 0 {
            format!("{} MINUTE{}", (duration.num_minutes()), if duration.num_minutes() > 1 {
                "S"
            } else { ""})
        }
        else {
            format!("{} SECOND{}", (duration.num_seconds()), if duration.num_seconds() > 1 {
                "S"
            } else { ""})
        };

        let sprint = match obj.get("endcontext") {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        };
        let sprint = match sprint.as_object() {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        };
        let sprint = match sprint.get("finalTime") {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        };
        let sprint = match sprint.as_f64() {
            Some(v) => v,
            None => return Html("<h1> parsing error </h1>").into_response()
        } as u64;

        let duration = Duration::from_millis(sprint);
        let ms = duration.subsec_millis();
        let secs = duration.as_secs() % 60;
        let minutes = duration.as_secs() / 60;
        let hours = minutes / 60;
        let hours_formated = if hours != 0 {
            format!("{}:", hours)
        }
        else {
            String::new()
        };

        let minutes = if hours != 0 {
            format!("{minutes:0width$}", width = 2)
        }
        else {
            minutes.to_string()
        };

        let final_sprint_time = format!("{hours_formated}{}:{:0width$}", minutes, secs, width = 2);

        let standing_set = if let Some(x) = data.records.sprint.rank  {
            if x < 1000 {
                let global_ranking = x.to_string();
                let global_ranking_class = match x {
                    1 => {
                        distinguishment = Some(TETO_HTML_SPRINT_CHAMPION_DISTINGUISHMENT);
                        "t1"
                    },
                    2..=10 => "t10",
                    11..=100 => "t100",
                    _ => ""
                };

                TETO_HTML_RECORDS_STANDING_SET
                    .replacen("{{global_ranking}}", &global_ranking, 1)
                    .replacen("{{global_ranking_class}}", global_ranking_class, 1)
            }
            else {
                String::new()
            }
        }
        else {
            String::new()
        };

        TETO_HTML_RECORDS_SPRINT
            .replacen("{{sprint_time}}", &final_sprint_time, 1)
            .replacen("{{sprint_time_ms}}", &ms.to_string(), 1)
            .replacen("{{date}}", &ago, 1)
            .replacen("{{standing_set}}", &standing_set, 1)
    } else {
        String::new()
    }
    }

    else {
        String::new()
    };

    let blitz = if let Some(record) = data.records.blitz.record {
        if !record.is_null() {

            let obj = match record.as_object() {
                Some(e) => e,
                None => return Html("<h1> Invalid user </h1>").into_response()
            }; 
            
            let time = match obj.get("ts") {
                Some(e) => e,
                None => return Html("<h1> Invalid user </h1>").into_response()
            };
            let time = match time.as_str() {
                Some(e) => e,
                None => return Html("<h1> Invalid user </h1>").into_response()
            };

        let time = match chrono::DateTime::<Utc>::from_str(time){ 
            Ok(e) => e,
            Err(_) => return Html("<h1> Invalid user </h1>").into_response()
        };

        let now = Utc::now();
        let duration = now.signed_duration_since(time);

        let ago = if duration.num_days() > 365 {
            format!("{} YEAR{}", (duration.num_days() / 365), if duration.num_days() - 365 >= 365 {
                "S"
            } else { ""})
        }
        else if duration.num_days() > 30 {
            format!("{} MONTH{}", (duration.num_days() / 30), if duration.num_days() - 30 >= 30 {
                "S"
            } else { ""})
        }
        else if duration.num_weeks() != 0 {
            format!("{} WEEK{}", (duration.num_weeks()), if duration.num_weeks() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_days() != 0 {
            format!("{} DAY{}", (duration.num_days()), if duration.num_days() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_hours() != 0 {
            format!("{} HOUR{}", (duration.num_hours()), if duration.num_hours() > 1 {
                "S"
            } else { ""})
        }
        else if duration.num_minutes() != 0 {
            format!("{} MINUTE{}", (duration.num_minutes()), if duration.num_minutes() > 1 {
                "S"
            } else { ""})
        }
        else {
            format!("{} SECOND{}", (duration.num_seconds()), if duration.num_seconds() > 1 {
                "S"
            } else { ""})
        };
        
        let blitz = match obj.get("endcontext") {
            Some(e) => e,
            None => return Html("<h1> Invalid user </h1>").into_response()
        };
        let blitz = match blitz.as_object() {
            Some(e) => e,
            None => return Html("<h1> Invalid user </h1>").into_response()
        };
        let blitz = match blitz.get("score") {
            Some(e) => e,
            None => return Html("<h1> Invalid user </h1>").into_response()
        };
        let blitz = match blitz.as_f64() {
            Some(e) => e,
            None => return Html("<h1> Invalid user </h1>").into_response()
        } as u64;
        
        let (million, blitz) = (blitz / 1_000_000, blitz % 1_000_000);

        let (thousands, blitz) = (blitz / 1_000, blitz % 1_000);
        
        let units = blitz;
        
        let units = if thousands != 0 {
            format!("{units:0width$}", width=3)
        }
        else {
            units.to_string()
        };

        let thousands = if million != 0 {
            format!("{thousands:0width$},", width=3)
        }
        else if thousands != 0 {
            format!("{thousands},")
        }
        else {
            String::new()
        };

        let million = 
            if million != 0 {
                format!("{million},")
            }
            else {
                String::new()
            };



        let standing_set = if let Some(x) = data.records.blitz.rank  {
            if x < 1000 {
                let global_ranking = x.to_string();
                let global_ranking_class = match x {
                    1 => {
                        distinguishment = Some(TETO_HTML_BLITZ_CHAMPION_CHAMPION_DISTINGUISHMENT);

                        "t1"
                    },
                    2..=10 => "t10",
                    11..=100 => "t100",
                    _ => ""
                };

                TETO_HTML_RECORDS_STANDING_SET
                    .replacen("{{global_ranking}}", &global_ranking, 1)
                    .replacen("{{global_ranking_class}}", global_ranking_class, 1)
            }
            else {
                String::new()
            }
        } else {
            String::new()
        };



        let blitz_score = format!("{million}{thousands}{units}");

        TETO_HTML_RECORDS_BLITZ
            .replacen("{{blitz_score}}", &blitz_score, 1)
            .replacen("{{date}}", &ago, 1)
            .replacen("{{standing_set}}", &standing_set, 1)
    }
    else {
        String::new()
    }
}
    else {
        String::new()
    };

    let user_records = TETO_HTML_RECORDS
        .replacen("{{tetra_league}}", &tetra_league, 1)
        .replacen("{{sprint}}", &sprint, 1)
        .replacen("{{blitz}}", &blitz, 1);

    let distinguishment = if let Some(staff_distinguishment) = staff_distinguishment{
        staff_distinguishment
    } else if let Some(distinguishment) = distinguishment {
        distinguishment.to_string()
    }
    else {
        String::new()
    };

    

    Html(TETO_HTML_FILE
        .replacen("{{bad_standing}}", bad_standing, 1)
        .replacen("{{has_banner}}", if has_banner {"has_banner"} else { "" }, 1)
        .replacen("{{banner}}", &banner, 1)
        .replacen("{{avatar}}", &avatar, 1)
        .replacen("{{username}}", &username, 1)
        .replacen("{{verified}}", verified, 1)
        .replacen("{{flag}}", &country_flag, 1)
        .replacen("{{joined_at}}", &joined_at, 1)
        .replacen("{{friends}}", &friends.to_string(), 1)
        .replacen("{{mod_badge}}", &mod_badge, 1)
        .replacen("{{distinguishment}}", &distinguishment, 1)
        .replacen("{{leveltag}}", &leveltag, 1)
        .replacen("{{level}}", &level.to_string(), 1)
        .replacen("{{game_time}}", &game_time, 1)
        .replacen("{{total_games}}", &total_games, 1)
        .replacen("{{supporter_badge}}", &supporter_badge, 1)
        .replacen("{{supporter_badge}}", &supporter_badge, 1)
        .replacen("{{badges}}", &badges, 1)
        .replacen("{{records}}", &user_records, 1)).into_response()

}


#[derive(Deserialize)]
struct TetraParam {
    user_id: String,
    game_num: usize
}


#[derive(Deserialize)]
struct ReplayParam {
    replay_id: String,
    user_id: String
}

#[derive(Deserialize, Serialize, Clone)]
struct GameReplayGameBoardUser {
    #[serde(rename = "_id")]
    id: String,
    username: String
}

#[derive(Deserialize, Clone)]
struct GameReplayGameBoard {
    user: GameReplayGameBoardUser,
    #[allow(unused)]
    active: bool,
    success: bool
}

#[allow(unused)]
#[derive(Deserialize, Clone)]
struct GameReplayFrame {
    #[allow(unused)]
    frame: u64,
    #[serde(rename = "type")]
    #[allow(unused)]
    frame_type: String,
    #[allow(unused)]
    data: serde_json::Value 
}

#[derive(Deserialize, Clone)]
struct GameReplayObject {
    frames: u64,
    // #[allow(unused)]
    // events: Vec<GameReplayFrame>
}

#[derive(Deserialize, Clone)]
struct GameReplayGameData {
    board: Vec<GameReplayGameBoard>,
    replays: Vec<GameReplayObject>
}

#[derive(Deserialize, Clone)]
struct GameReplayData {
    data: Vec<GameReplayGameData>,
    ts: chrono::DateTime<chrono::Utc>,
    endcontext: Vec<LeagueEndContext>
}

#[derive(Deserialize)]
struct GameReplayPacket {
    #[allow(unused)]
    success: bool,
    game: Option<GameReplayData>,
}


// basic handler that responds with a static string
async fn league_recent(State(state): State<AppState>, Query(user_id): Query<TetraParam>) -> Response {
    let Ok(packet) = http::client::fetch_tetra_league_recent(&user_id.user_id).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't fetch or parse data").into_response()
    };
    
    let Some(data) = packet.data else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Tetrio server error").into_response()
    };

    let Some(record) = data.records.get(user_id.game_num - 1) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "No recent records").into_response()
    };

    generate_league_replay(state, &record.replay_id, &user_id.user_id).await
}

async fn league_replay(State(state): State<AppState>, Query(replay_data): Query<ReplayParam>) -> Response {
    generate_league_replay(state, &replay_data.replay_id, &replay_data.user_id).await
}

async fn generate_league_replay(state: AppState, replay_id: &str, user_id: &str) -> Response {
    
    let replay_data = state.tetrio_http_client.fetch_tetrio_replay(replay_id, &state.tetrio_token).await;

    let replay_data = match replay_data {
        Ok(replay_data) => replay_data,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Couldn't fetch replay data: {e}")).into_response(),
    };

    let Some(data) = &replay_data.game else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't fetch replay data (game)").into_response()
    };



    let (Some(left), Some(right)) = (        
        data.endcontext.iter().find(|f| f.user.id == user_id.into()), data.endcontext.iter().find(|f| f.user.id != user_id.into())) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse data (couldn't find end contexts)").into_response()
    };

    let rounds = left.points.secondary_avg_tracking.len();

    let league_record = LeagueRecord {
        averages: Averages {
            left: Average {
                username: left.user.username.to_string(),
                pps: left.points.tertiary,
                apm: left.points.secondary,
                vs: left.points.extra.vs,
                score: left.points.primary as u32,
            },
            right: Average {
                username: right.user.username.to_string(),
                pps: right.points.tertiary,
                apm: right.points.secondary,
                vs: right.points.extra.vs,
                score: right.points.primary as u32,
            }
        },
        rounds: (0..rounds).filter_map(|index| {
            let game_data = &data.data[index];
            let frame = game_data.replays[0].frames;
            let frames = frame as f64 / 60.0;

            let duration = Duration::from_secs_f64(frames);
            
            let minutes = duration.as_secs() / 60;
            let seconds = duration.as_secs() % 60;


            Some(Round { 
                left: Stats {
                    pps: left.points.tertiary_avg_tracking[index],
                    apm: left.points.secondary_avg_tracking[index],
                    vs: left.points.extra_avg_tracking.aggregate_stats_vs_score[index],
                    success: match game_data.board.iter().find(|a| a.user.id == user_id) {
                        Some(e) => e.success,
                        None => return None
                    },
                }, 
                right: Stats {
                    pps: right.points.tertiary_avg_tracking[index],
                    apm: right.points.secondary_avg_tracking[index],
                    vs: right.points.extra_avg_tracking.aggregate_stats_vs_score[index],
                    success: match game_data.board.iter().find(|a| a.user.id != user_id) {
                        Some(e) => e.success,
                        None => return None
                    },
                }, 
                time: format!("{minutes}:{seconds:02}")
            })
        }).collect(),
    };

    generate_league_recent(league_record, data.ts)
}

fn generate_league_recent(league_record: LeagueRecord, timestamp: DateTime<Utc>) -> Response 
{
    let matches = league_record.rounds.iter().map(|round| {
        <&str>::clone(&TETRA_HTML_MATCH)
            .replacen("{left_success}", if round.left.success {"success"} else {""}, 1)
            .replacen("{right_success}", if round.right.success {"success"} else {""}, 1) 
            .replacen("{left_pps}", &format!("{:.2}", round.left.pps), 1)
            .replacen("{left_apm}", &format!("{:.2}", round.left.apm), 1)
            .replacen("{left_vs}", &format!("{:.2}", round.left.vs), 1)
            .replacen("{time}", &round.time, 1)
            .replacen("{right_pps}", &format!("{:.2}", round.right.pps), 1)
            .replacen("{right_apm}", &format!("{:.2}", round.right.apm), 1)
            .replacen("{right_vs}", &format!("{:.2}", round.right.vs), 1)
    }).fold(String::new(), |g, next| {g + "\n" + &next});
    let generated_html = <&str>::clone(&TETRA_HTML_FILE)
        .replacen("{left_score}", &league_record.averages.left.score.to_string(), 1)
        .replacen("{right_score}", &league_record.averages.right.score.to_string(), 1)
        .replacen("{left_username}", &league_record.averages.left.username.to_uppercase(), 2)
        .replacen("{left_pps}", &format!("{:.2}", league_record.averages.left.pps), 1)
        .replacen("{left_apm}", &format!("{:.2}", league_record.averages.left.apm), 1)
        .replacen("{left_vs}",  &format!("{:.2}", league_record.averages.left.vs), 1)
        .replacen("{right_username}", &league_record.averages.right.username.to_uppercase(), 2)
        .replacen("{right_pps}", &format!("{:.2}", league_record.averages.right.pps), 1)
        .replacen("{right_apm}", &format!("{:.2}", league_record.averages.right.apm), 1)
        .replacen("{right_vs}",  &format!("{:.2}", league_record.averages.right.vs), 1)
        .replacen("{matches}", &matches, 1)
        .replacen("{played_date}", &format!("{}", timestamp.format("%d/%m/%Y")), 1)
        .replacen("{played_time}", &format!("{}", timestamp.format("%H:%M:%S")), 1)
        ;

    Html(generated_html).into_response()
}

async fn league_recent_test() -> impl IntoResponse {
    generate_league_recent(LeagueRecord 
        { 
            averages: Averages { 
                left: Average {
                    username: "TAKATHEDINOSAUR".to_string(),
                    pps: 10.0, 
                    apm: 100.0, 
                    vs: 1.0, 
                    score: 7
                }, right: Average { 
                    username: "RUDOT".to_string(),
                    pps: 10.0, 
                    apm: 50.0, 
                    vs: 100.0, 
                    score: 6 
                } 
            }, 
            rounds: vec![
                Round { 
                    left: Stats{ pps: 1.90, apm: 100.0, vs: 80.0, success: true }, 
                    right: Stats { pps: 1.80, apm: 90.0, vs: 90.0, success: false }, 
                    time: "2:55".to_string() 
                },
                Round { 
                    left: Stats{ pps: 1.90, apm: 100.0, vs: 80.0, success: false }, 
                    right: Stats { pps: 1.80, apm: 90.0, vs: 90.0, success: true }, 
                    time: "2:55".to_string() 
                }
            ] 
        },  chrono::offset::Utc::now()
    )
}

