use crate::parsley::{get_data, parse_record};
use libtruinlag::ChallengeSet;
use libtruinlag::commands::*;
use std::collections::HashMap;

const CHALLENGE_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRcImsj8yCZNaKSx4wYk6GZnBkZ_Eody246mqM4UjsvYIW3wqd37kIhhIlrWJ3tiwSLbN9RWzMVs-V1/pub?gid=1012921349&single=true&output=csv";
const ZONENKAFF_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vSEz-OcFSz13kGB2Z9iRzLmBkor8R2o7C-tzOSm91cQKt4foAG6iGynlT8PhO3I5Pt5iB_Mj7Bu0BeO/pub?gid=1336941165&single=true&output=csv";
const DISTANCES_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vSEz-OcFSz13kGB2Z9iRzLmBkor8R2o7C-tzOSm91cQKt4foAG6iGynlT8PhO3I5Pt5iB_Mj7Bu0BeO/pub?gid=381450010&single=true&output=csv";

pub fn get_input(query: &str) -> String {
    use std::io::{Write, stdin, stdout};
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

fn printnnl(text: &str) {
    use std::io::Write;
    print!("{}", text);
    std::io::stdout().flush().unwrap();
}

pub async fn import_challenges(mut sender: libtruinlag::api::SendConnection) {
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
