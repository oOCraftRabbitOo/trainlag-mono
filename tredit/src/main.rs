use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, shells::Zsh};
use colored::Colorize;
use libtruinlag::{
    Challenge, PartialGameConfig,
    api::{SendConnection, connect},
    commands::EngineAction,
};

mod interactive;
mod parsley;

async fn run_command(command: EngineAction, mut sender: SendConnection) {
    match sender.send(command).await {
        Ok(response) => {
            eprintln!("{}", "Command executed successfully:".green().bold());
            println!("{:#?}", response);
        }
        Err(err) => {
            eprintln!(
                "{}",
                "There was an issue executing the command:".red().bold()
            );
            println!("{}", err);
        }
    }
}

async fn get_player_by_name(name: &str, sender: &mut SendConnection) -> u64 {
    match name.parse() {
        Ok(id) => id,
        Err(_) => {
            sender
                .get_global_state()
                .await
                .unwrap()
                .1
                .iter()
                .find(|p| p.name.to_lowercase() == name.to_lowercase())
                .expect("couldn't find player by name")
                .id
        }
    }
}

async fn get_session_by_name(name: &str, sender: &mut SendConnection) -> u64 {
    match name.parse() {
        Ok(id) => id,
        Err(_) => {
            sender
                .get_global_state()
                .await
                .unwrap()
                .0
                .iter()
                .find(|s| s.name.to_lowercase() == name.to_lowercase())
                .expect("couldn't find session by name")
                .id
        }
    }
}

async fn get_team_by_name(session: u64, name: &str, sender: &mut SendConnection) -> usize {
    match name.parse() {
        Ok(id) => id,
        Err(_) => {
            sender
                .get_session_state(session)
                .await
                .unwrap()
                .0
                .iter()
                .find(|t| {
                    t.name.to_lowercase() == name.to_lowercase()
                        || t.players
                            .iter()
                            .any(|p| p.name.to_lowercase() == name.to_lowercase())
                })
                .expect("couldn't find team by name")
                .id
        }
    }
}

async fn get_zone_by_number(number: u64, sender: &mut SendConnection) -> u64 {
    sender
        .get_zones()
        .await
        .unwrap()
        .iter()
        .find(|z| z.zone == number || z.id == number)
        .expect("couldn't find zone")
        .id
}

fn cli() -> Command {
    Command::new("tredit")
        .about("A command line utility to control truinlag")
        .subcommand_required(false)
        .arg_required_else_help(false)
        .allow_external_subcommands(false)
        .arg(
            Arg::new("generate_zsh_completions")
                .long("generate_zsh_completions")
                .num_args(0)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("address")
                .long("address")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("release")
                .short('r')
                .num_args(0)
                .action(ArgAction::Append),
        )
        .subcommand(
            Command::new("add_player")
                .about("Add a player into the DB")
                .arg(Arg::new("Name").required(true))
                .arg(Arg::new("Passphrase").required(true))
                .arg(
                    Arg::new("Session")
                        .required(false)
                        .help("Will be set to None if not provided"),
                ),
        )
        .subcommand(
            Command::new("set_player_session")
                .about("Set a player's session")
                .arg(Arg::new("Player").required(true))
                .arg(
                    Arg::new("Session")
                        .required(false)
                        .help("Will be set to None if not provided"),
                ),
        )
        .subcommand(
            Command::new("set_player_passphrase")
                .about("Set a player's passphrase")
                .arg(Arg::new("Player").required(true))
                .arg(Arg::new("Passphrase").required(true)),
        )
        .subcommand(
            Command::new("add_session")
                .about("Add a new session to the db")
                .arg(Arg::new("Name").required(true)),
        )
        .subcommand(
            Command::new("add_team")
                .about("Add a new team to a certain session in the DB")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Name").required(true)),
        )
        .subcommand(
            Command::new("assign")
                .about("Assign a player to a team")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Player").required(true))
                .arg(
                    Arg::new("Team")
                        .required(false)
                        .help("Player will be unassigned if not provided"),
                ),
        )
        .subcommand(
            Command::new("get_state")
                .about("Get the state of the database, or, optionally, a session")
                .arg(
                    Arg::new("Session")
                        .required(false)
                        .help("Global state fetched if not provided"),
                ),
        )
        .subcommand(
            Command::new("make_catcher")
                .about("Make a team catcher.")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Team").required(true)),
        )
        .subcommand(
            Command::new("make_runner")
                .about("Make a team runner.")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Team").required(true)),
        )
        .subcommand(
            Command::new("add_challenge_to_team")
                .about("Add a challenge to a team")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Team").required(true))
                .arg(Arg::new("Title").required(true))
                .arg(Arg::new("Description").required(true))
                .arg(Arg::new("Points").required(true)),
        )
        .subcommand(
            Command::new("rename_team")
                .about("Rename a team")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Team").required(true))
                .arg(Arg::new("Name").required(true)),
        )
        .subcommand(
            Command::new("import")
                .about("Download, parse and import all challenges from the google sheet"),
        )
        .subcommand(
            Command::new("delete_challenges")
                .about("Delete all challenges from the truinlag DB")
                .arg(
                    Arg::new("yes")
                        .short('y')
                        .required(false)
                        .num_args(0)
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(Command::new("get_challenges").about("Get all challenges from the truinlag DB"))
        .subcommand(
            Command::new("start")
                .about("Start the game")
                .arg(Arg::new("Session").required(true)),
        )
        .subcommand(
            Command::new("stop")
                .about("Finish the game")
                .arg(Arg::new("Session").required(true)),
        )
        .subcommand(Command::new("get_zones").about("Get all zones from the truinlag DB"))
        .subcommand(
            Command::new("get_locations")
                .about("Get all teams and locations from a session")
                .arg(Arg::new("Session").required(true)),
        )
        .subcommand(Command::new("get_challenge_sets").about("Get all challenge sets"))
        .subcommand(
            Command::new("get_game_config")
                .about("Gets the game config from a session")
                .arg(Arg::new("Session").required(true)),
        )
        .subcommand(
            Command::new("set_start_time")
                .about("Set the start time of future games")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Hours").required(true))
                .arg(Arg::new("Minutes").required(true)),
        )
        .subcommand(
            Command::new("set_end_time")
                .about("Set the end time of future games")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Hours").required(true))
                .arg(Arg::new("Minutes").required(true)),
        )
        .subcommand(
            Command::new("set_start_zone")
                .about("Set the start zone of future games")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Zone").required(true)),
        )
        .subcommand(
            Command::new("set_num_catchers")
                .about("Set the number of hunters in future games")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Number of hunters").required(true)),
        )
        .subcommand(
            Command::new("set_challenge_sets")
                .about("Set the challenge sets that are used in future games")
                .arg(Arg::new("Session").required(true))
                .arg(
                    Arg::new("Challenge Set ID")
                        .action(ArgAction::Append)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("print_address")
                .about("print the socket address used to contact truinlag"),
        )
        .subcommand(
            Command::new("remove_team")
                .about("Remove a team")
                .arg(Arg::new("Session").required(true))
                .arg(Arg::new("Team").required(true)),
        )
        .subcommand(
            Command::new("rename_player")
                .about("Rename a player")
                .arg(Arg::new("Player").required(true))
                .arg(Arg::new("Name").required(true)),
        )
        .subcommand(Command::new("get_sectors").about("Get all sectors"))
}

#[tokio::main]
async fn main() {
    let mut args = cli().get_matches();
    let generate_arg = args.contains_id("generate_zsh_completions");
    let address = args.get_one("address").cloned();
    let release = args.contains_id("release");
    let address = address.unwrap_or(format!(
        "/tmp/truinsocket_{}{}",
        if release { "" } else { "dev_" },
        env!("CARGO_PKG_VERSION")
    ));
    let args = args.remove_subcommand();
    let (name, mut sub_args) = match args {
        None => {
            if generate_arg {
                generate(Zsh, &mut cli(), "tredit", &mut std::io::stdout());
            } else {
                println!("Hiii, I'm tredit :3")
            }
            return;
        }
        Some(args) => args,
    };
    let (mut sender, _recvr) = connect(Some(&address)).await.unwrap();

    match name.as_str() {
        "get_sectors" => run_command(EngineAction::GetSectors, sender).await,

        "rename_player" => {
            let player_id = get_session_by_name(
                sub_args.get_one::<String>("Player").expect("required"),
                &mut sender,
            )
            .await;
            let name = sub_args.remove_one("Name").expect("required");
            run_command(
                EngineAction::RenamePlayer {
                    player_id,
                    new_name: name,
                },
                sender,
            )
            .await
        }
        "remove_team" => {
            let session_id = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let team_id = get_team_by_name(
                session_id,
                sub_args.get_one::<String>("Team").expect("required"),
                &mut sender,
            )
            .await;
            run_command(
                EngineAction::RemoveTeam {
                    session_id,
                    team_id,
                },
                sender,
            )
            .await
        }
        "print_address" => {
            println!("{}", address);
        }
        "set_challenge_sets" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let sets = sub_args
                .remove_many::<u64>("Challenge Set ID")
                .unwrap()
                .collect();
            let config = PartialGameConfig {
                challenge_sets: Some(sets),
                ..Default::default()
            };
            run_command(
                EngineAction::SetGameConfig {
                    session_id: session,
                    config,
                },
                sender,
            )
            .await
        }

        "set_num_catchers" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let num_catchers = sub_args.remove_one::<u64>("Number of hunters").unwrap();
            let config = PartialGameConfig {
                num_catchers: Some(num_catchers),
                ..Default::default()
            };
            run_command(
                EngineAction::SetGameConfig {
                    session_id: session,
                    config,
                },
                sender,
            )
            .await
        }

        "set_start_zone" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let zone = sub_args.remove_one::<u64>("Zone").unwrap();
            let config = PartialGameConfig {
                start_zone: Some(zone),
                ..Default::default()
            };
            run_command(
                EngineAction::SetGameConfig {
                    session_id: session,
                    config,
                },
                sender,
            )
            .await
        }

        "set_end_time" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let hours = sub_args.remove_one::<u32>("Hours").unwrap();
            let minutes = sub_args.remove_one::<u32>("Minutes").unwrap();
            let time = chrono::NaiveTime::from_hms_opt(hours, minutes, 0).unwrap();
            let config = PartialGameConfig {
                end_time: Some(time),
                ..Default::default()
            };
            run_command(
                EngineAction::SetGameConfig {
                    session_id: session,
                    config,
                },
                sender,
            )
            .await
        }

        "set_start_time" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let hours = sub_args.remove_one::<u32>("Hours").unwrap();
            let minutes = sub_args.remove_one::<u32>("Minutes").unwrap();
            let time = chrono::NaiveTime::from_hms_opt(hours, minutes, 0).unwrap();
            let config = PartialGameConfig {
                start_time: Some(time),
                ..Default::default()
            };
            run_command(
                EngineAction::SetGameConfig {
                    session_id: session,
                    config,
                },
                sender,
            )
            .await
        }

        "get_game_config" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            run_command(EngineAction::GetGameConfig(session), sender).await
        }

        "get_challenge_sets" => run_command(EngineAction::GetChallengeSets, sender).await,

        "get_locations" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            run_command(EngineAction::GetLocations(session), sender).await
        }

        "get_zones" => run_command(EngineAction::GetAllZones, sender).await,

        "stop" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            run_command(EngineAction::Stop(session), sender).await
        }

        "start" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            run_command(EngineAction::Start(session), sender).await
        }

        "get_challenges" => run_command(EngineAction::GetRawChallenges, sender).await,

        "delete_challenges" => {
            if sub_args.contains_id("yes")
                || interactive::get_input("Are you sure (yes/no) ").as_str() == "yes"
            {
                run_command(EngineAction::DeleteAllChallenges, sender).await
            }
        }

        "import" => interactive::import_challenges(sender).await,

        "rename_team" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let team = get_team_by_name(
                session,
                sub_args.get_one::<String>("Team").expect("required"),
                &mut sender,
            )
            .await;
            let name = sub_args.remove_one::<String>("Name").expect("required");
            run_command(
                EngineAction::RenameTeam {
                    session_id: session,
                    team,
                    new_name: name,
                },
                sender,
            )
            .await
        }

        "add_challenge_to_team" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let team = get_team_by_name(
                session,
                sub_args.get_one::<String>("Team").expect("required"),
                &mut sender,
            )
            .await;
            let title = sub_args.remove_one::<String>("Title").expect("required");
            let description = sub_args
                .remove_one::<String>("Description")
                .expect("required");
            let points = sub_args.remove_one::<u64>("Points").expect("required");
            run_command(
                EngineAction::AddChallengeToTeam {
                    session_id: session,
                    team,
                    challenge: Challenge {
                        title,
                        description,
                        points,
                        id: 0,
                    },
                },
                sender,
            )
            .await
        }

        "make_runner" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let team = get_team_by_name(
                session,
                sub_args.get_one::<String>("Team").expect("required"),
                &mut sender,
            )
            .await;
            run_command(
                EngineAction::MakeTeamRunner {
                    session_id: session,
                    team_id: team,
                },
                sender,
            )
            .await
        }

        "make_catcher" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let team = get_team_by_name(
                session,
                sub_args.get_one::<String>("Team").expect("required"),
                &mut sender,
            )
            .await;
            run_command(
                EngineAction::MakeTeamCatcher {
                    session_id: session,
                    team_id: team,
                },
                sender,
            )
            .await
        }

        "get_state" => {
            let session = sub_args.get_one::<String>("Session");
            let session = match session {
                None => None,
                Some(s) => Some(get_session_by_name(s, &mut sender).await),
            };
            run_command(EngineAction::GetState(session), sender).await
        }

        "assign" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let player = get_player_by_name(
                sub_args.get_one::<String>("Player").expect("required"),
                &mut sender,
            )
            .await;
            let team = sub_args.get_one::<String>("Team");
            let team = match team {
                None => None,
                Some(t) => Some(get_team_by_name(session, t, &mut sender).await),
            };
            run_command(
                EngineAction::AssignPlayerToTeam {
                    session_id: session,
                    player,
                    team,
                },
                sender,
            )
            .await
        }

        "add_team" => {
            let session = get_session_by_name(
                sub_args.get_one::<String>("Session").expect("required"),
                &mut sender,
            )
            .await;
            let name = sub_args
                .get_one::<String>("Name")
                .expect("required")
                .clone();
            run_command(
                EngineAction::AddTeam {
                    session_id: session,
                    name,
                    discord_channel: None,
                    colour: None,
                },
                sender,
            )
            .await
        }

        "add_session" => {
            let name = sub_args
                .get_one::<String>("Name")
                .expect("required")
                .clone();
            run_command(
                EngineAction::AddSession {
                    name,
                    mode: libtruinlag::Mode::Traditional,
                },
                sender,
            )
            .await
        }

        "set_player_passphrase" => {
            let id = get_player_by_name(
                sub_args.get_one::<String>("Player").expect("required"),
                &mut sender,
            )
            .await;
            let passphrase = sub_args
                .get_one::<String>("Passphrase")
                .expect("required")
                .clone();
            run_command(
                EngineAction::SetPlayerPassphrase {
                    player: id,
                    passphrase,
                },
                sender,
            )
            .await
        }

        "set_player_session" => {
            let player = get_player_by_name(
                sub_args.get_one::<String>("Player").expect("required"),
                &mut sender,
            )
            .await;
            let session = sub_args.get_one::<u64>("Session").cloned();
            run_command(EngineAction::SetPlayerSession { player, session }, sender).await
        }

        "add_player" => {
            let name = sub_args
                .get_one::<String>("Name")
                .expect("required")
                .clone();
            let passphrase = sub_args
                .get_one::<String>("Passphrase")
                .expect("required")
                .clone();
            let session = sub_args.get_one::<String>("Session").cloned();
            let session_id = match session {
                None => None,
                Some(session) => Some(get_session_by_name(&session, &mut sender).await),
            };
            run_command(
                EngineAction::AddPlayer {
                    name,
                    discord_id: None,
                    passphrase,
                    session: session_id,
                },
                sender,
            )
            .await
        }

        _ => unreachable!(),
    }
}
