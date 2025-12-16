use anyhow::Context;
use std::collections::HashMap;
use std::str::FromStr;
use truinlag::{
    self, ChallengeSet, ChallengeStatus, ChallengeType, InputChallenge, RandomPlaceType, TextError,
    Zone,
};

pub async fn get_data(url: &str) -> Vec<HashMap<String, String>> {
    let mut records = Vec::new();

    let csv_bytes = reqwest::get(url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
        .into_bytes();

    let mut rdr = csv::Reader::from_reader(csv_bytes.as_slice());
    for result in rdr.deserialize() {
        let challenge_data: HashMap<String, String> = match result {
            Ok(result) => result,
            Err(error) => {
                eprintln!(
                    "Error occurred in gathering data:\n{}\nIgnoring and continuing",
                    error
                );
                continue;
            }
        };
        records.push(challenge_data);
    }

    records
}

fn parse_bool(record: &HashMap<String, String>, field: &str) -> anyhow::Result<bool> {
    match record
        .get(field)
        .context(format!("field \"{}\" not found", field))?
        .to_lowercase()
        .as_str()
    {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(TextError(format!("can't interpret {} as a boolean", other)))?,
    }
}

fn parse_to<T>(record: &HashMap<String, String>, field: &str) -> anyhow::Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: Sync,
    <T as FromStr>::Err: Send,
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: 'static,
{
    record
        .get(field)
        .context(format!("field \"{}\" not found", field))?
        .parse()
        .context(format!(
            "couldn't parse {} into {}",
            field,
            std::any::type_name::<T>()
        ))
}

fn parse_to_option<T>(record: &HashMap<String, String>, field: &str) -> anyhow::Result<Option<T>>
where
    T: FromStr,
    <T as FromStr>::Err: Sync,
    <T as FromStr>::Err: Send,
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: 'static,
{
    match record
        .get(field)
        .context(format!("field \"{}\" not found", field))?
        .as_str()
    {
        "" => Ok(None),
        text => Ok(Some(text.parse().context(format!(
            "couldn't parse {} into {}",
            field,
            std::any::type_name::<T>()
        ))?)),
    }
}

fn parse_to_or<T>(record: &HashMap<String, String>, field: &str, default: T) -> anyhow::Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: Sync,
    <T as FromStr>::Err: Send,
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: 'static,
{
    match parse_to_option(record, field)? {
        Some(thing) => Ok(thing),
        None => Ok(default),
    }
}
pub fn parse_record(
    record: &HashMap<String, String>,
    challenge_sets: &[ChallengeSet],
    truin_zones: &[Zone],
) -> anyhow::Result<Option<InputChallenge>> {
    let status: ChallengeStatus = record
        .get("status")
        .context("field \"status\" not found")?
        .parse()?;
    use ChallengeStatus::*;
    match status {
        Edited | Rejected | Glorious | ToSort => return Ok(None),
        Approved | Refactor => (),
    }

    let kind: ChallengeType = record
        .get("challenge_type")
        .context("field challenge_type not present")?
        .parse()?;

    let mut sets = Vec::new();
    for set in record
        .get("sets")
        .context("field \"sets\" not found")?
        .split(",")
        .map(|s| s.trim())
    {
        match challenge_sets.iter().find(|e| e.name == set) {
            Some(s) => sets.push(s.id),
            None => Err(TextError(format!("Couldn't find challenge set {}", set)))?,
        }
    }

    let title = match record
        .get("title")
        .context("field \"title\" not found")?
        .trim()
    {
        "" => None,
        text => Some(text),
    };

    let description = match record
        .get("description")
        .context("field \"description\" not found")?
        .trim()
    {
        "" => None,
        text => Some(text),
    };

    let place = match record
        .get("place")
        .context("field \"place\" not found")?
        .trim()
    {
        "" => None,
        text => Some(text),
    };
    match kind {
        ChallengeType::Kaff | ChallengeType::ZKaff => {
            if place.is_none() {
                Err(TextError(
                    "challenge type is Kaff or ZKaff, but there is no place".into(),
                ))?
            }
        }
        _ => {
            if title.is_none() || description.is_none() || place.is_some() {
                Err(TextError("challenge type is not Kaff or ZKaff, but either title or description are missing or place is present".into()))?
            }
        }
    }

    let mut random_place = None;
    if let Some(title) = title {
        if title.contains("%z") {
            random_place = Some(RandomPlaceType::Zone);
        }
        if title.contains("%s") {
            random_place = Some(RandomPlaceType::SBahnZone);
        }
    }
    if let Some(description) = description {
        if description.contains("%z") {
            random_place = Some(RandomPlaceType::Zone);
        }
        if description.contains("%s") {
            random_place = Some(RandomPlaceType::SBahnZone);
        }
    }
    if random_place.is_some() {
        match kind {
            ChallengeType::ZKaff | ChallengeType::Kaff => Err(TextError(
                "challenge type is Kaff or ZKaff but it has random place".into(),
            ))?,
            _ => (),
        }
    }

    let mut zones = Vec::new();
    let zone_text = record
        .get("zone")
        .context("field \"zone\" not found")?
        .trim();
    if !zone_text.is_empty() {
        for zone in zone_text.split(",").map(|s| s.trim()) {
            let zone = zone.parse().context("couldn't parse a zone")?;
            zones.push(
                truin_zones
                    .iter()
                    .find(|z| z.zone == zone)
                    .context(format!("couldn't find zone {} in truin zones", zone))?
                    .id,
            )
        }
    }

    let title_de_ch = parse_to_option(record, "title_de_ch")?;
    let description_de_ch = parse_to_option(record, "description_de_ch")?;
    let title_en_uk = parse_to_option(record, "title_en_uk")?;
    let description_en_uk = parse_to_option(record, "description_en_uk")?;
    let title_fr_ch = parse_to_option(record, "title_fr_ch")?;
    let description_fr_ch = parse_to_option(record, "description_fr_ch")?;

    let mut translated_titles = HashMap::new();
    let mut translated_descriptions = HashMap::new();

    if let Some(title) = title_de_ch {
        translated_titles.insert("de_ch".into(), title);
    }
    if let Some(title) = title_fr_ch {
        translated_titles.insert("de_fr".into(), title);
    }
    if let Some(title) = title_en_uk {
        translated_titles.insert("en_uk".into(), title);
    }

    if let Some(description) = description_de_ch {
        translated_descriptions.insert("de_ch".into(), description);
    }
    if let Some(description) = description_fr_ch {
        translated_descriptions.insert("fr_ch".into(), description);
    }
    if let Some(description) = description_en_uk {
        translated_descriptions.insert("en_uk".into(), description);
    }

    let challenge = InputChallenge {
        kind: parse_to(record, "challenge_type")?,
        sets,
        status: parse_to(record, "status")?,
        title: parse_to_option(record, "title")?,
        description: parse_to_option(record, "description")?,
        place: parse_to_option(record, "place")?,
        kaffskala: parse_to_option(record, "kaffskala")?,
        grade: parse_to_option(record, "grade")?,
        zone: zones,
        bias_sat: parse_to_or(record, "bias_sat", 1.0)?,
        bias_sun: parse_to_or(record, "bias_sun", 1.0)?,
        walking_time: parse_to_or(record, "walking_time", 0)?,
        stationary_time: parse_to_or(record, "stationary_time", 0)?,
        additional_points: parse_to_or(record, "additional_points", 0)?,
        repetitions: parse_to_or(record, "min_reps", 0)?..parse_to_or(record, "max_reps", 0)?,
        points_per_rep: parse_to_or(record, "points_per_rep", 0)?,
        station_distance: parse_to_or(record, "station_distance", 0)?,
        time_to_hb: parse_to_or(record, "time_to_hb", 0)?,
        departures: parse_to_or(record, "departures", 0)?,
        dead_end: parse_bool(record, "dead_end")?,
        no_disembark: parse_bool(record, "no_disembark")?,
        fixed: parse_bool(record, "fixed_points")?,
        in_perimeter_override: Some(parse_bool(record, "in_perim")?),
        action: None,
        comment: parse_to(record, "comment")?,
        id: None,
        random_place,
        translated_titles,
        translated_descriptions,
    };

    challenge.check_validity()?;

    Ok(Some(challenge))
}
