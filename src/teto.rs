// can you make this file cleaner?  



use std::{str::FromStr, sync::Arc, time::Duration};

use axum::{extract::{Path, State}, response::{Html, IntoResponse}};

use chrono::Utc;
use tetrio_api::models::users::{summaries::{blitz::BlitzSummary, sprint::SprintSummary, tetra_league::LeagueSummary}, user_badge::UserBadge, user_distinguishment::UserDistinguishment, user_info::UserInfo, user_role::UserRole};

use crate::AppState;

const TETO_HTML_BOT_FILE: &str = include_str!("../assets/teto/bot.html");
const TETO_HTML_BANNED_FILE: &str = include_str!("../assets/teto/banned.html");
const TETO_HTML_FILE: &str = include_str!("../assets/teto/index.html");
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

const TETO_TWC_DISTINGUISHMENT: &str = r#"<div class="tetra_distinguishment ns tetra_distinguishment_twc" data-detail="{{detail}}"><h1>TETR.IO WORLD CHAMPION</h1><p>{{detail}} TETR.IO WORLD CHAMPIONSHIP</p></div>"#;

const TETO_HTML_RECORDS_TETRA_LEAGUE: &str = r#"<div class="tetra_modal_record flex-item tetra_modal_record_league tetra_modal_record_league_active">
							<div class="tetra_modal_record_header">
								<h6>TETRA LEAGUE</h6>
								<div class="standingset">
									
										{{country_ranking}}
									
									
										<div class="standingset_global " data-digits="4">#<span>{{global_ranking}}</span></div>
									
								</div>
							</div>
							<h5 title="14173.95200992754"><img src="{{rank}}">{{tr}}<span class="ms">TR</span></h5>
							<h3><span>{{apm}}</span> apm <span>{{pps}}</span> pps <span>{{vs}}</span> vs</h3></div>"#;

const TETO_HTML_RECORDS_TETRA_LEAGUE_RATING: &str = r#"<div class="tetra_modal_record flex-item tetra_modal_record_league">
							<div class="tetra_modal_record_header"><h6>TETRA LEAGUE</h6></div>
							<h5>{{games_played}}<span class="ms">/10 rating games</span></h5>
							<h3><span>{{games_won}}</span> games won</h3>
						</div>
"#;

const TETO_HTML_RECORDS_COUNTRY_RANKING: &str = r#"<div class="standingset_local">{{country_flag}} #<span>{{country_ranking}}</span></div>"#;

const TETO_HTML_RECORDS_SPRINT: &str = r#"<div class="tetra_modal_record flex-item">
						<div class="tetra_modal_record_header">
							<h6>40 LINES</h6>
							<div class="standingset">
								
									{{country_ranking}}
								
								
									<div class="standingset_global " data-digits="6">#<span>{{global_ranking}}</span></div>
								
							</div>
						</div>
						<h5>{{sprint_time}}<span class="ms">{{sprint_time_ms}}</span></h5>
						<h3><span>{{date}}</span> ago</h3></div>"#;

const TETO_HTML_RECORDS_BLITZ: &str = r#"<div class="tetra_modal_record flex-item">
						<div class="tetra_modal_record_header">
							<h6>BLITZ</h6>
							<div class="standingset">
								
									{{country_ranking}}
								
								
									<div class="standingset_global " data-digits="5">#<span>{{global_ranking}}</span></div>
								
							</div>
						</div>
						<h5>{{blitz_score}}</h5>
						<h3><span>{{date}}</span> ago</h3></div>"#;

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




fn level_from_xp(x: f64) -> f64 {
    // simplify the formula
    (x / 500.0).powf(0.6) + x / (5000.0 + (f64::max(0.0, x - 4.0 * 10.0f64.powi(6)) / 5000.0)) + 1.0
}

async fn handle_banned(data: &UserInfo) -> impl IntoResponse {
    let avatar_rev = data.avatar_revision.unwrap_or(0);
    let avatar = if avatar_rev != 0 {
        format!("https://tetr.io/user-content/avatars/{}.jpg?rv={}", data.id, avatar_rev)
    }
    else {
        String::from("/teto/unkown_avatar.webp")
    };

    Html(
        TETO_HTML_BANNED_FILE.replacen("{{avatar}}",&avatar, 1)
        .replacen("{{username}}", &data.username.to_uppercase(), 1)
    )
}

async fn handle_bot(data: &UserInfo) -> impl IntoResponse {
    let bot_owner = match &data.botmaster {
        Some(data) => data.to_uppercase(),
        None => String::new()
    };
    let avatar_rev = data.avatar_revision.unwrap_or(0);
    let avatar = if avatar_rev != 0 {
        format!("https://tetr.io/user-content/avatars/{}.jpg?rv={}", data.id, avatar_rev)
    }
    else {
        String::from("/teto/unkown_avatar.webp")
    };

    Html(
        TETO_HTML_BOT_FILE.replacen("{{avatar}}",&avatar, 1)
        .replacen("{{username}}", &data.username.to_uppercase(), 1)
        .replacen("{{owner}}", &bot_owner, 1)   
    )
}

fn parse_banner(has_banner: bool, banner_rev: i64, user_id: &str) -> String {
    if has_banner {

        TETO_HTML_BANNER.replacen("{{banner_url}}", &format!(
            "https://tetr.io/user-content/banners/{user_id}.jpg?rv={banner_rev}"
        ), 1) + TETO_HTML_BANNER_SEP
    } else {
        String::new()
    }
}

fn parse_avatar(avatar_rev: i64, user_id: &str) -> String {
    if avatar_rev != 0 {
        format!("https://tetr.io/user-content/avatars/{user_id}.jpg?rv={avatar_rev}")
    }
    else {
        String::from("/teto/unkown_avatar.webp")
    }
}

fn parse_bad_standing(bad_standing: Option<bool>) -> &'static str {
    if let Some(bad_standing) = bad_standing {
        if bad_standing {
            TETO_HTML_BAD_STANDING
        }
        else {
            ""
        }
    } else {
        ""
    }
}

fn has_banner(banner_rev: i64, supporter_tier: i64) -> bool {
    return banner_rev != 0 && supporter_tier != 0;
}

fn parse_mod_badge(role: &UserRole) -> String {
    match role {
        UserRole::Anon => String::new(),
        UserRole::User => String::new(),
        UserRole::Bot => String::new(),
        UserRole::Banned => String::new(),
        UserRole::Mod => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-mod.png"),
        UserRole::Admin => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-admin.png"),
        UserRole::SysOp => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-sysop.png"),
        UserRole::HalfMod => TETO_HTML_MOD_BADGE.replace("{{mod_icon}}", "https://tetr.io/res/verified-halfmod.png"),
        UserRole::Hidden => String::new(),
        UserRole::Unknown(_) => String::new(),
    }
}

fn parse_distinguishment(distinguishment: &Option<UserDistinguishment>) -> Option<String> {
    if let Some(distinguishment) = distinguishment {
        if let Some(detail) = &distinguishment.detail {

            if distinguishment.distinguishment_type.as_str() == "twc" {
                return Some(TETO_TWC_DISTINGUISHMENT.replacen("{{detail}}", detail.as_ref(), 2))
            }

            let header = if let Some(header) = &distinguishment.header {
                header.replace("%tetrio%", TETO_HTML_STAFF_DISTINGUISHMENT_TETRIO_LOGO)
                .replace("%osk%", TETO_HTML_STAFF_DISTINGUISHMENT_OSK)
            }
            else {
                String::new()
            };
            
            let footer = if let Some(footer) = &distinguishment.footer {
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
    }
}


fn parse_country_flag(country: Option<&str>) -> String {
    if let Some(country) = &country {
        TETO_HTML_FLAG.replacen("{{country_code}}", &country.to_lowercase(), 1)
    }
    else {
        String::new()
    }
}

fn parse_level_tag(level: u64) -> String {
    if level != 5000u64 {
        let shape_color = (level / 10) % 10;
        let shape = (level / 100) % 5;
        let badge_color = (level / 500) % 10;
        format!("lt_shape_{shape} lt_badge_color_{badge_color} lt_shape_color_{shape_color}") 
    }
    else {
        String::from("lt_golden")
    }
}

fn parse_total_games(gamesplayed: i64, gameswon: i64) -> String {
    {
        if gamesplayed != -1 || gameswon != -1 {
            let games_won = if gameswon != -1 {
                TETO_HTML_GAMES_WON.replacen("{{online_games_won}}", &gameswon.to_string(), 1)
            } else {
                String::new()
            };

            let games_played = if gamesplayed != -1 {
                format!(" / {}", gamesplayed)
            } else {
                String::new()
            };

            TETO_HTML_TOTAL_GAMES.replacen("{{online_games_won}}", games_won.as_ref(), 1).replacen("{{online_games_played}}", games_played.as_ref(), 1)
        }
        else {
            String::new()
        }

    }
}

fn parse_gametime(gametime: f64) -> String {
    if gametime != -1.0 {
        let playtime = Duration::from_secs_f64(gametime);
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
    }
}

fn parse_supporter_badge(supporter_tier: i64) -> String {
    if supporter_tier != 0 {
        TETO_HTML_SUPPORTER.replacen("{{supporter_tier}}", &supporter_tier.to_string(), 1)
    }
    else {
        String::new()
    }
}

fn parse_user_badges(badges: &[UserBadge]) -> String {
    if !badges.is_empty() {
        let badges = badges.iter().map(|e| {
            <&str>::clone(&TETO_HTML_BADGE).replacen("{{badge}}", &format!("https://tetr.io/res/badges/{}.png", e.id), 1)
        }).fold(String::new(), |g, next| {g + "\n" + &next});

        TETO_HTML_BADGES.replacen("{{badges}}", &badges, 1)
    } else {
        String::new()
    }
}

fn parse_duration_since(duration: &chrono::Duration) -> String {
    if duration.num_days() > 365 {
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
    }
}

struct ParsedResultWithDistinguishment {
    distinguishment: Option<&'static str>,
    result: String
}

fn parse_blitz_score_number(blitz: u64) -> String {
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


    return format!("{}{}{}", million, thousands, units);
}

fn parse_blitz_score(summary: &BlitzSummary) -> Result<ParsedResultWithDistinguishment, &'static str> {
    let mut result = ParsedResultWithDistinguishment {
        distinguishment: None,
        result: String::new()
    };
    
    result.result = if let Some(record) = &summary.record {

            let time = match chrono::DateTime::<Utc>::from_str(record.ts.as_ref()){ 
                Ok(e) => e,
                Err(_) => return Err("<h1> Couldn't parse Blitz record </h1>")
            };

            let now = Utc::now();
            let duration = now.signed_duration_since(time);

            let ago = parse_duration_since(&duration);
        
            let blitz = record.results.stats.score as u64;
        
            let blitz_score = parse_blitz_score_number(blitz);


            result.distinguishment = if summary.rank == 1 {
                Some(TETO_HTML_BLITZ_CHAMPION_CHAMPION_DISTINGUISHMENT)
            }
            else {
                None
            };

            let country_ranking = if summary.rank_local != -1 {
                TETO_HTML_RECORDS_COUNTRY_RANKING.replacen("{{country_ranking}}", &&parse_blitz_score_number(summary.rank_local as u64).to_string(), 1)
            }  
            else {
                String::new()
            };    


            TETO_HTML_RECORDS_BLITZ
                .replacen("{{blitz_score}}", &blitz_score, 1)
                .replacen("{{date}}", &ago, 1)
                .replacen("{{global_ranking}}", &parse_blitz_score_number(summary.rank as u64).to_string(), 1)
                .replacen("{{country_ranking}}", &country_ranking, 1)

        } else {
            String::new()
        };

    Ok(result)
}


fn parse_tetra_league(league: &LeagueSummary) -> ParsedResultWithDistinguishment {
    let mut result = ParsedResultWithDistinguishment {
        distinguishment: None,
        result: String::new()
    };

    result.result = if let LeagueSummary { tr: Some(tr), gamesplayed: Some(gamesplayed), gameswon: Some(gameswon), .. } = &league {
        if let LeagueSummary {rank: Some(rank), standing: Some(standing), standing_local: Some(standing_local), ..} = &league {
            result.distinguishment = if *standing == 1 {
                Some(TETO_HTML_TETRA_LEAGUE_CHAMPION_DISTINGUISHMENT)
            } else {
                None
            };
            
            if gamesplayed == &0 {
                String::new()
            }
            else if gamesplayed < &10 {
                TETO_HTML_RECORDS_TETRA_LEAGUE_RATING
                .replacen("{{games_played}}", &gamesplayed.to_string(), 1)
                .replacen("{{games_won}}", &gameswon.to_string(), 1)
            }
            
            else {

                let country_ranking = if standing_local != &-1 {
                    TETO_HTML_RECORDS_COUNTRY_RANKING.replacen("{{country_ranking}}", &parse_blitz_score_number(*standing_local as u64).to_string(), 1)
                }  
                else {
                    String::new()
                };          
                TETO_HTML_RECORDS_TETRA_LEAGUE
                .replacen("{{rank}}", &format!("https://tetr.io/res/league-ranks/{}.png", rank).to_lowercase(), 1)
                .replacen("{{tr}}", &(tr.round()).to_string(), 1)
                .replacen("{{vs}}", &format!("{:.2}", league.vs.unwrap_or(0.0)), 1)
                .replacen("{{apm}}", &format!("{:.2}", league.apm.unwrap_or(0.0)), 1)
                .replacen("{{pps}}", &format!("{:.2}", league.pps.unwrap_or(0.0)), 1)
                .replacen("{{country_ranking}}", &country_ranking, 1)
                .replacen("{{global_ranking}}", &parse_blitz_score_number(*standing as u64).to_string(), 1)
            }
        }
        else {
            String::new()
        }
    }
    else {
        String::new()
    };


    


    return result;
}


fn parse_sprint(sprint_record: &SprintSummary) -> Result<ParsedResultWithDistinguishment, &'static str> {
    let mut result = ParsedResultWithDistinguishment {
        distinguishment: None,
        result: String::new()
    };

    result.result = if let Some(record) = &sprint_record.record {

        let time = match chrono::DateTime::<Utc>::from_str(record.ts.as_ref()){
            Ok(v) => v,
            Err(_) => return Err("<h1> parsing error </h1>")
        };

        let now = Utc::now();
        let duration = now.signed_duration_since(time);
        let ago = parse_duration_since(&duration);
        let sprint = record.results.stats.finaltime as u64;

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



        result.distinguishment = if sprint_record.rank == 1 {
            Some(TETO_HTML_SPRINT_CHAMPION_DISTINGUISHMENT)
        } else {
            None
        };

        let country_ranking = if sprint_record.rank_local != -1 {
            TETO_HTML_RECORDS_COUNTRY_RANKING.replacen("{{country_ranking}}", &parse_blitz_score_number(sprint_record.rank_local as u64).to_string(), 1)
        }  
        else {
            String::new()
        };          

        TETO_HTML_RECORDS_SPRINT
            .replacen("{{sprint_time}}", &final_sprint_time, 1)
            .replacen("{{sprint_time_ms}}", &ms.to_string(), 1)
            .replacen("{{date}}", &ago, 1)
            .replacen("{{country_ranking}}", &country_ranking, 1)
            .replacen("{{global_ranking}}", &parse_blitz_score_number(sprint_record.rank as u64).to_string(), 1)
    }
    else {
        String::new()
    };
    
    return Ok(result);
}


struct TetoHTMLParams {
    bad_standing: &'static str,
    has_banner: bool,
    banner: String,
    avatar: String,
    username: String,
    flag: String,
    joined_at: String,
    friends: i64,
    mod_badge: String,
    distinguishment: String,
    leveltag: String,
    level: u64,
    game_time: String,
    total_games: String,
    supporter_badge: String,
    badges: String,
    records: String,
}

impl TetoHTMLParams {
    fn into_html_page(self) -> String {
        let TetoHTMLParams { bad_standing, has_banner, banner, avatar, username, flag, joined_at, friends, mod_badge, distinguishment, leveltag, level, game_time, total_games, supporter_badge, badges, records } = self;
        TETO_HTML_FILE
            .replacen("{{bad_standing}}", bad_standing.as_ref(), 1)
            .replacen("{{has_banner}}", if has_banner {"has_banner"} else { "" }, 1)
            .replacen("{{banner}}", banner.as_ref(), 1)
            .replacen("{{avatar}}", &avatar, 1)
            .replacen("{{username}}", &username, 1)
            .replacen("{{flag}}", &flag.as_ref(), 1)
            .replacen("{{joined_at}}", &joined_at, 1)
            .replacen("{{friends}}", &friends.to_string(), 1)
            .replacen("{{mod_badge}}", mod_badge.as_ref(), 1)
            .replacen("{{distinguishment}}", &distinguishment, 1)
            .replacen("{{leveltag}}", &leveltag, 1)
            .replacen("{{level}}", &level.to_string(), 1)
            .replacen("{{game_time}}", game_time.as_ref(), 1)
            .replacen("{{total_games}}", total_games.as_ref(), 1)
            .replacen("{{supporter_badge}}", supporter_badge.as_ref(), 1)
            .replacen("{{badges}}", badges.as_ref(), 1)
            .replacen("{{records}}", &records, 1)
            .replace("{{country_flag}}", &flag)
    }
}

pub(crate) async fn teto_test(State(state): State<Arc<AppState<'_>>>, Path(user_id): Path<String>) -> impl IntoResponse {
    let client = state.api_http_client.as_ref();
    let user = match client.fetch_user_info(&user_id).await {
        Ok(e) => e,
        Err(e) => return Html(format!("<h1> Invalid user (1) {e:?} </h1>")).into_response()
    };
    let data = match user.data {
        Some(e) => e,
        None => return Html("<h1> Invalid user (2) </h1>").into_response()
    };

    if let UserRole::Banned = data.role {
        return handle_banned(&data).await.into_response()
    }

    if let UserRole::Bot = data.role {
        return handle_bot(&data).await.into_response();
    }

    
    let distinguishment = None;
    let banner_rev = data.banner_revision.unwrap_or(0);
    let supporter_tier = data.supporter_tier;
    let has_banner = has_banner(banner_rev, supporter_tier);
    let banner = parse_banner(has_banner, banner_rev, &data.id);
    let bad_standing = parse_bad_standing(data.badstanding);
    let avatar_rev = data.avatar_revision.unwrap_or(0);
    let avatar = parse_avatar(avatar_rev, &data.id);
    let mod_badge = parse_mod_badge(&data.role);
    let staff_distinguishment = parse_distinguishment(&data.distinguishment);
    let flag = parse_country_flag(data.country.as_deref());
    let level = level_from_xp(data.xp) as u64;
    let leveltag = parse_level_tag(level);
    let total_games = parse_total_games(data.gamesplayed, data.gameswon);
    let game_time = parse_gametime(data.gametime);
    let supporter_badge = parse_supporter_badge(data.supporter_tier);
    let badges = parse_user_badges(&data.badges);

    let username = data.username.to_uppercase();
    let summaries = match client.fetch_user_summaries(&data.id).await {
        Ok(v) => v,
        Err(_) => return Html("<h1> Couldn't get summaries! </h1>").into_response()
    };

    let friends = data.friend_count.unwrap_or(0);
    let joined_at = if let Some(ts) = data.ts {
        let time = chrono::DateTime::<Utc>::from_str(&ts).unwrap_or_default();

        let now = Utc::now();
        let duration = now.signed_duration_since(time);

        let ago = parse_duration_since(&duration);

        String::from("JOINED ") + &ago + " AGO - "        
    }
    else {
        String::from("HERE SINCE THE BEGINNING - ")
    };

    let data = match summaries.data {
        Some(v) => v,
        None => return Html("<h1> No user records </h1>").into_response(),
    };

    let ParsedResultWithDistinguishment {distinguishment: league_distinguishment, result: league} = parse_tetra_league(&data.league);
    let distinguishment = distinguishment.or(league_distinguishment);



    let ParsedResultWithDistinguishment {distinguishment: sprint_distinguishment, result: sprint } = match parse_sprint(&data.sprint) {
        Ok(result) => result,
        Err(err) => return Html(err).into_response()
    };

    let distinguishment = distinguishment.or(sprint_distinguishment);
    
    let ParsedResultWithDistinguishment { distinguishment: blitz_distinguishment, result: blitz} = match parse_blitz_score(&data.blitz) {
        Ok(result) => result,
        Err(err) => return Html(err).into_response()
    };

    let distinguishment = distinguishment.or(blitz_distinguishment);

    let records = TETO_HTML_RECORDS
        .replacen("{{tetra_league}}", &league, 1)
        .replacen("{{sprint}}", &sprint, 1)
        .replacen("{{blitz}}", blitz.as_ref(), 1);

    let distinguishment = if let Some(staff_distinguishment) = staff_distinguishment{
        staff_distinguishment
    } else if let Some(distinguishment) = distinguishment {
        distinguishment.to_string()
    }
    else {
        String::new()
    };



    let page = TetoHTMLParams {
        bad_standing,
        has_banner,
        banner,
        avatar,
        username,
        flag,
        joined_at,
        friends,
        mod_badge,
        distinguishment,
        leveltag,
        level,
        game_time,
        total_games,
        supporter_badge,
        badges,
        records,
    };

    return Html(page.into_html_page()).into_response()
}