use crate::parsley::{get_data, parse_record};
use libtruinlag::ChallengeSet;
use libtruinlag::commands::*;
use std::collections::HashMap;

const CHALLENGE_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRcImsj8yCZNaKSx4wYk6GZnBkZ_Eody246mqM4UjsvYIW3wqd37kIhhIlrWJ3tiwSLbN9RWzMVs-V1/pub?gid=1012921349&single=true&output=csv";
const ZONENKAFF_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRcImsj8yCZNaKSx4wYk6GZnBkZ_Eody246mqM4UjsvYIW3wqd37kIhhIlrWJ3tiwSLbN9RWzMVs-V1/pub?gid=1336941165&single=true&output=csv";
const DISTANCES_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRcImsj8yCZNaKSx4wYk6GZnBkZ_Eody246mqM4UjsvYIW3wqd37kIhhIlrWJ3tiwSLbN9RWzMVs-V1/pub?gid=381450010&single=true&output=csv";
const SECTOR_SHEET: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vRcImsj8yCZNaKSx4wYk6GZnBkZ_Eody246mqM4UjsvYIW3wqd37kIhhIlrWJ3tiwSLbN9RWzMVs-V1/pub?gid=2097621621&single=true&output=csv";

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
    let sheet_zones: Vec<HashMap<String, String>> = get_data(ZONENKAFF_SHEET).await.to_vec();
    println!("  done!");
    let s_bahn_zones = [
        110, 112, 117, 120, 121, 132, 133, 134, 141, 142, 151, 154, 155, 156, 180, 181,
    ];
    printnnl("adding missing zones");
    for sheet_zone in &sheet_zones {
        if truin_zones
            .iter()
            .any(|tz| tz.zone == sheet_zone.get("Zone").unwrap().parse().unwrap())
        {
            continue;
        }
        printnnl(".");
        sender
            .send(EngineAction::AddZone {
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
            .send(EngineAction::AddMinutesTo {
                from_zone: conn.zone_from,
                to_zone: conn.zone_to,
                minutes: conn.minutes,
            })
            .await
            .unwrap();
    }
    println!("  done!");

    printnnl("fetching existing sectors...");
    let truin_sectors = sender.get_sectors().await.unwrap();
    println!("  done!");
    printnnl("fetching sheet sector data...");
    let sheet_sectors: Vec<(char, Vec<char>)> = get_data(SECTOR_SHEET)
        .await
        .iter()
        .map(|s| {
            (
                s.get("sector").unwrap().chars().next().unwrap(),
                s.get("neighbours").unwrap().chars().collect(),
            )
        })
        .collect();
    println!("  done!");
    printnnl("adding missing sectors");
    for (sheet_sector, _) in &sheet_sectors {
        printnnl(".");
        if !truin_sectors.iter().any(|s| &s.name == sheet_sector) {
            sender
                .send(EngineAction::AddSector(*sheet_sector))
                .await
                .unwrap();
        }
    }
    println!("  done!");

    printnnl("re-fetching sectors...");
    let truin_sectors = sender.get_sectors().await.unwrap();
    println!("  done!");

    printnnl("adding missing sector neighbourhoods...");
    for truin_sector in &truin_sectors {
        if let Some((_sheet_sector, sheet_neighbours)) =
            sheet_sectors.iter().find(|s| s.0 == truin_sector.name)
        {
            for sheet_neighbour in sheet_neighbours {
                let sheet_neighbour_id = truin_sectors
                    .iter()
                    .find(|s| &s.name == sheet_neighbour)
                    .unwrap();
                if !(sheet_neighbour_id.id > truin_sector.id
                    || truin_sector.neighbours.contains(&sheet_neighbour_id.id))
                {
                    printnnl(".");
                    sender
                        .send(EngineAction::AddNeighbourhood(
                            truin_sector.id,
                            sheet_neighbour_id.id,
                        ))
                        .await
                        .unwrap();
                }
            }
        }
    }
    println!("  done!");

    printnnl("resetting close sectors of zones...");
    for truin_zone in &truin_zones {
        let sheet_zone = match sheet_zones
            .iter()
            .find(|z| z.get("Zone").unwrap().parse::<u64>().unwrap() == truin_zone.zone)
        {
            Some(z) => z,
            None => {
                printnnl("!");
                continue;
            }
        };
        let mut zone_sector_ids: Vec<u64> = sheet_zone
            .get("sectors")
            .unwrap()
            .trim()
            .chars()
            .map(|c| truin_sectors.iter().find(|s| s.name == c).unwrap().id)
            .collect();
        zone_sector_ids.sort();
        let mut truin_close_sectors: Vec<u64> =
            truin_zone.close_sectors.iter().map(|s| s.id).collect();
        truin_close_sectors.sort();
        if zone_sector_ids != truin_close_sectors {
            printnnl(".");
            sender
                .send(EngineAction::SetCloseSectors {
                    zone_id: truin_zone.id,
                    sector_ids: zone_sector_ids,
                })
                .await
                .unwrap();
        }
    }
    println!("  done!");

    printnnl("fetching UC4 challenge data...");
    let records = get_data(CHALLENGE_SHEET).await;
    println!("  done!");

    let mut challenges = Vec::new();
    println!("parsing challenges...");
    for (i, record) in records.iter().enumerate() {
        let challenge = parse_record(record, &challenge_sets, &truin_zones, &truin_sectors);
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
