use crate::parsley::{get_data, parse_record};
use std::collections::HashMap;
use truinlag::{self, Challenge, ChallengeSet, Mode};
use truinlag::{commands::*, Player};

const CHALLENGE_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRcImsj8yCZNaKSx4wYk6GZnBkZ_Eody246mqM4UjsvYIW3wqd37kIhhIlrWJ3tiwSLbN9RWzMVs-V1/pub?gid=1012921349&single=true&output=csv";
const ZONENKAFF_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vSEz-OcFSz13kGB2Z9iRzLmBkor8R2o7C-tzOSm91cQKt4foAG6iGynlT8PhO3I5Pt5iB_Mj7Bu0BeO/pub?gid=1336941165&single=true&output=csv";
const DISTANCES_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vSEz-OcFSz13kGB2Z9iRzLmBkor8R2o7C-tzOSm91cQKt4foAG6iGynlT8PhO3I5Pt5iB_Mj7Bu0BeO/pub?gid=381450010&single=true&output=csv";

pub fn get_input(query: &str) -> String {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    print!("{}", query);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

pub async fn interactive() {
    let (sender, _receiver) = truinlag::api::connect(None).await.unwrap();
    match get_input(
        "What do you want to do?\n\
        1: add player\n\
        2: set player session\n\
        3: add team\n\
        4: add session\n\
        5: assign\n\
        6: get state\n\
        7: get global state\n\
        8: make team catcher\n\
        9: make team runner\n\
        10: add challenge to team\n\
        11: rename team\n\
        12: import sheet challenges to truinlag\n\
        13: delete all challenges\n\
        14: get challenges from truinlag\n\
        15: start game\n\
        16: stop game\n\
        17: get zones\n\
        18: get locations\n\
        i'm all ears: ",
    )
    .as_str()
    {
        "1" => add_player(sender).await,
        "2" => set_session(sender).await,
        "3" => add_team(sender).await,
        "4" => add_session(sender).await,
        "5" => assign(sender).await,
        "6" => get_session_state(sender).await,
        "7" => get_global_state(sender).await,
        "8" => make_team_catcher(sender).await,
        "9" => make_team_runner(sender).await,
        "10" => add_challenge_to_team(sender).await,
        "11" => rename_team(sender).await,
        "12" => import_challenges(sender).await,
        "13" => delete_all_challenges(sender).await,
        "14" => get_challenges(sender).await,
        "15" => start_game(sender).await,
        "16" => stop_game(sender).await,
        "17" => get_zones(sender).await,
        "18" => get_locations(sender).await,
        _ => println!("Sorry, I can't count that high :("),
    };
}

async fn get_locations(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::GetLocations,
        })
        .await
        .unwrap();
    println!("{:?}", response);
    if let ResponseAction::SendLocations(locations) = response {
        let mut count = 0;
        for loc in locations {
            count += loc.1.len();
        }
        println!("Total: {} locations", count)
    }
}

async fn get_zones(mut sender: truinlag::api::SendConnection) {
    let response = sender
        .send(EngineCommand {
            session: None,
            action: EngineAction::GetAllZones,
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn start_game(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::Start,
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn stop_game(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::Stop,
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn rename_team(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let team: usize = get_input("team: ").parse().unwrap();
    let new_name = get_input("new name for team: ");

    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::RenameTeam { team, new_name },
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn add_challenge_to_team(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let team: usize = get_input("team: ").parse().unwrap();
    let title = get_input("challenge title: ");
    let description = get_input("challenge description: ");
    let points: u64 = get_input("challenge points: ").parse().unwrap();

    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::AddChallengeToTeam {
                team,
                challenge: Challenge {
                    title,
                    description,
                    points,
                },
            },
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn make_team_catcher(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let team: usize = get_input("team: ").parse().unwrap();
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::MakeTeamCatcher(team),
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn make_team_runner(mut sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    let team: usize = get_input("team: ").parse().unwrap();
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: EngineAction::MakeTeamRunner(team),
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn get_session_state(sender: truinlag::api::SendConnection) {
    let session: u64 = get_input("session: ").parse().unwrap();
    get_state(sender, Some(session)).await;
}

async fn get_global_state(sender: truinlag::api::SendConnection) {
    get_state(sender, None).await;
}

async fn get_state(mut sender: truinlag::api::SendConnection, session: Option<u64>) {
    let response = sender
        .send(EngineCommand {
            session,
            action: EngineAction::GetState,
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn get_player_by_passphrase(
    mut sender: truinlag::api::SendConnection,
    passphrase: String,
) -> Player {
    match sender
        .send(EngineCommand {
            session: None,
            action: EngineAction::GetPlayerByPassphrase(passphrase.clone()),
        })
        .await
        .unwrap()
    {
        ResponseAction::Player(player) => player,
        ResponseAction::Error(err) => {
            eprintln!(
                "Couldn't find player with passphrase {}: {:?}",
                passphrase, err
            );
            panic!("oh oh");
        }
        _ => {
            eprintln!("didn't work");
            panic!("the database didn't return a Player??");
        }
    }
}

async fn assign(mut sender: truinlag::api::SendConnection) {
    use EngineAction::*;
    let session: u64 = get_input("in which session? (u64): ").parse().unwrap();
    let player =
        get_player_by_passphrase(sender.clone(), get_input("The player's passphrase: ")).await;
    let team: Option<usize> =
        match get_input("which team should they be assigned to? (leave empty for None): ").as_str()
        {
            "" => None,
            input => Some(input.parse().unwrap()),
        };
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: AssignPlayerToTeam {
                player: player.id,
                team,
            },
        })
        .await
        .unwrap();
    println!("{:?}", response)
}

async fn add_session(mut sender: truinlag::api::SendConnection) {
    use EngineAction::*;
    let name = get_input("Session name: ");
    let response = sender
        .send(EngineCommand {
            session: None,
            action: AddSession {
                name,
                mode: Mode::Traditional,
            },
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn add_team(mut sender: truinlag::api::SendConnection) {
    use EngineAction::*;
    let name = get_input("What should the team's name be?: ");
    let session: u64 = get_input("In which session?: ").parse().unwrap();
    let response = sender
        .send(EngineCommand {
            session: Some(session),
            action: AddTeam {
                name,
                discord_channel: None,
                colour: None,
            },
        })
        .await
        .unwrap();
    println!("{:?}", response);
}

async fn set_session(mut sender: truinlag::api::SendConnection) {
    let passphrase = get_input("Passphrase of player: ");
    let session = get_input("Session (leave empty for None): ");
    let session = match session.as_str() {
        "" => None,
        text => Some(text.parse::<u64>().unwrap()),
    };
    println!("parsed session, getting player id");
    let player = get_player_by_passphrase(sender.clone(), passphrase).await;
    println!("Got player id, sending SetPlayerSession command");
    let response = sender
        .send(EngineCommand {
            session: None,
            action: EngineAction::SetPlayerSession {
                player: player.id,
                session,
            },
        })
        .await
        .unwrap();
    println!("Engine Response: {:?}", response)
}

async fn add_player(mut sender: truinlag::api::SendConnection) {
    let name = get_input("Name: ");
    let passphrase = get_input("Passphrase: ");
    let session = get_input("Session (leave empty for None): ");
    let session = match session.as_str() {
        "" => None,
        text => Some(text.parse::<u64>().unwrap()),
    };
    let response = sender
        .send(EngineCommand {
            session: None,
            action: EngineAction::AddPlayer {
                name,
                discord_id: None,
                passphrase,
                session,
            },
        })
        .await
        .unwrap();
    println!("Engine response: {:?}", response)
}

#[allow(non_camel_case_types)]
#[derive(Debug, serde::Deserialize)]
enum SheetChallengeType {
    kaff,
    z_kaff,
    ortsspezifisch,
    regionsspezifisch,
    unspezifisch,
    zoneable,
}

#[allow(non_camel_case_types)]
#[derive(Debug, serde::Deserialize)]
enum SheetStatus {
    approved,
    edited,
    rejected,
    glorious,
    to_sort,
    refactor,
}

fn printnnl(text: &str) {
    use std::io::Write;
    print!("{}", text);
    std::io::stdout().flush().unwrap();
}

pub async fn import_challenges(mut sender: truinlag::api::SendConnection) {
    let sheet_sets = [
        "og",
        "geso",
        "family",
        "transit_specialist",
        "off_with_the_hinges",
        "physical",
        "base",
    ];

    printnnl("fetching challenge sets...");
    let challenge_sets = sender.get_challenge_sets().await.unwrap();
    println!("  done!");
    let challenge_set_names: Vec<String> = challenge_sets.iter().map(|s| s.name.clone()).collect();

    printnnl("adding missing challenge sets");
    for sset in sheet_sets {
        printnnl(".");
        if !challenge_set_names.contains(&sset.into()) {
            sender.add_challenge_set(sset.into()).await.unwrap();
        }
    }
    println!("  done!");

    printnnl("re-fetching challenge sets... ");
    let challenge_sets: Vec<ChallengeSet> = sender
        .get_challenge_sets()
        .await
        .unwrap()
        .iter()
        .filter(|s| sheet_sets.contains(&s.name.as_str()))
        .cloned()
        .collect();
    println!("  done!");

    printnnl("fetching existing zones...");
    let truin_zones = sender.get_zones().await.unwrap();
    println!("  done!");
    printnnl("fetching sheet zonic kaff data...");
    let sheet_zones: Vec<HashMap<String, String>> = get_data(ZONENKAFF_SHEET)
        .await
        .iter()
        .filter(|z| {
            let zz = z.get("Zone").unwrap().parse().unwrap();
            !truin_zones.iter().any(|tz| tz.zone == zz)
        })
        .cloned()
        .collect();
    println!("  done!");
    let s_bahn_zones = [
        110, 112, 117, 120, 121, 132, 133, 134, 141, 142, 151, 154, 155, 156, 180, 181,
    ];
    printnnl("adding missing zones");
    for sheet_zone in sheet_zones {
        printnnl(".");
        sender
            .send(EngineCommand {
                session: None,
                action: EngineAction::AddZone {
                    zone: sheet_zone.get("Zone").unwrap().parse().unwrap(),
                    num_conn_zones: sheet_zone.get("num conn zones").unwrap().parse().unwrap(),
                    num_connections: sheet_zone.get("num connections").unwrap().parse().unwrap(),
                    train_through: sheet_zone
                        .get("train through")
                        .unwrap()
                        .to_lowercase()
                        .parse()
                        .unwrap(),
                    mongus: sheet_zone
                        .get("Mongus")
                        .unwrap()
                        .to_lowercase()
                        .parse()
                        .unwrap(),
                    s_bahn_zone: s_bahn_zones
                        .contains(&sheet_zone.get("Zone").unwrap().parse::<i64>().unwrap()),
                },
            })
            .await
            .unwrap();
    }
    println!("  done!");

    printnnl("re-fetching zones...");
    let truin_zones = sender.get_zones().await.unwrap();
    println!("  done!");
    printnnl("fetching sheet zone distance data...");
    let connections = get_data(DISTANCES_SHEET).await;
    println!("  done!");
    struct Connection {
        pub zone_from: u64,
        pub zone_to: u64,
        pub minutes: u64,
    }
    let mut cookednections = Vec::new();
    for conn in connections {
        cookednections.push(Connection {
            zone_from: truin_zones
                .iter()
                .find(|z| z.zone == conn.get("Zone A").unwrap().parse().unwrap())
                .unwrap()
                .id,
            zone_to: truin_zones
                .iter()
                .find(|z| z.zone == conn.get("Zone B").unwrap().parse().unwrap())
                .unwrap()
                .id,
            minutes: conn.get("Travel Time").unwrap().parse().unwrap(),
        });
    }
    printnnl("adding connections to db");
    for conn in cookednections {
        printnnl(".");
        sender
            .send(EngineCommand {
                session: None,
                action: EngineAction::AddMinutesTo {
                    from_zone: conn.zone_from,
                    to_zone: conn.zone_to,
                    minutes: conn.minutes,
                },
            })
            .await
            .unwrap();
    }
    println!("  done!");

    printnnl("fetching UC4 challenge data...");
    let records = get_data(CHALLENGE_SHEET).await;
    println!("  done!");

    let mut challenges = Vec::new();
    println!("parsing challenges...");
    for (i, record) in records.iter().enumerate() {
        let challenge = parse_record(record, &challenge_sets, &truin_zones);
        match challenge {
            Ok(c) => {
                if let Some(c) = c {
                    challenges.push(c);
                }
            }
            Err(err) => eprintln!("Error while parsing line {}:\n{}\n", i, err),
        }
    }
    println!("done!");

    printnnl("adding challenges to db");
    for challenge in challenges {
        printnnl(".");
        sender.add_raw_challenge(challenge).await.unwrap();
    }
    println!("  done!");
}

async fn delete_all_challenges(mut sender: truinlag::api::SendConnection) {
    match get_input("Are you sure? (yes/no): ").as_str() {
        "yes" => {
            let response = sender
                .send(EngineCommand {
                    session: None,
                    action: EngineAction::DeleteAllChallenges,
                })
                .await;
            println!("{:?}", response);
        }
        "no" => println!("Alright, I won't delete all challenges"),
        _ => println!(
            "I'm not sure whether you're sure here, so I will not delete all challenges :)"
        ),
    }
}

async fn get_challenges(mut sender: truinlag::api::SendConnection) {
    println!("{:?}", sender.get_raw_challenges().await);
}
