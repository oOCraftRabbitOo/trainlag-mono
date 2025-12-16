use clap::{value_parser, Arg, ArgAction, Command};
use clap_complete::{generate, shells::Zsh};
use colored::Colorize;
use truinlag::{
    api::connect,
    commands::{EngineAction, EngineCommand},
    Challenge, PartialGameConfig,
};

mod interactive;
mod parsley;

async fn run_command(command: EngineCommand, address: Option<String>) {
    let (mut sender, _recvr) = connect(address.as_deref()).await.unwrap();
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
                    Arg::new("Session ID")
                        .required(false)
                        .help("Will be set to None if not provided")
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("set_player_session")
                .about("Set a player's session")
                .arg(
                    Arg::new("Player ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Session ID")
                        .required(false)
                        .help("Will be set to None if not provided")
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("set_player_passphrase")
                .about("Set a player's passphrase")
                .arg(
                    Arg::new("Player ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
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
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(Arg::new("Name").required(true)),
        )
        .subcommand(
            Command::new("assign")
                .about("Assign a player to a team")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Player ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Team ID")
                        .required(false)
                        .help("Player will be unassigned if not provided")
                        .value_parser(value_parser!(usize)),
                ),
        )
        .subcommand(
            Command::new("get_state")
                .about("Get the state of the database, or, optionally, a session")
                .arg(
                    Arg::new("Session ID")
                        .required(false)
                        .help("Global state fetched if not provided")
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("make_catcher")
                .about("Make a team catcher.")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Team ID")
                        .required(true)
                        .value_parser(value_parser!(usize)),
                ),
        )
        .subcommand(
            Command::new("make_runner")
                .about("Make a team runner.")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Team ID")
                        .required(true)
                        .value_parser(value_parser!(usize)),
                ),
        )
        .subcommand(
            Command::new("add_challenge_to_team")
                .about("Add a challenge to a team")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Team ID")
                        .required(true)
                        .value_parser(value_parser!(usize)),
                )
                .arg(Arg::new("Title").required(true))
                .arg(Arg::new("Description").required(true))
                .arg(
                    Arg::new("Points")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("rename_team")
                .about("Rename a team")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Team ID")
                        .required(true)
                        .value_parser(value_parser!(usize)),
                )
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
            Command::new("start").about("Start the game").arg(
                Arg::new("Session ID")
                    .value_parser(value_parser!(u64))
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("stop").about("Finish the game").arg(
                Arg::new("Session ID")
                    .required(true)
                    .value_parser(value_parser!(u64)),
            ),
        )
        .subcommand(Command::new("get_zones").about("Get all zones from the truinlag DB"))
        .subcommand(
            Command::new("get_locations")
                .about("Get all teams and locations from a session")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(Command::new("get_challenge_sets").about("Get all challenge sets"))
        .subcommand(
            Command::new("get_game_config")
                .about("Gets the game config from a session")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("set_start_time")
                .about("Set the start time of the next game")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Hours")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    Arg::new("Minutes")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("set_end_time")
                .about("Set the end time of the next game")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Hours")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    Arg::new("Minutes")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("set_start_zone")
                .about("Set the end time of the next game")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Zone")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("set_num_catchers")
                .about("Set the end time of the next game")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Number of hunters")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("set_challenge_sets")
                .about("Set the end time of the next game")
                .arg(
                    Arg::new("Session ID")
                        .required(true)
                        .value_parser(value_parser!(u64)),
                )
                .arg(
                    Arg::new("Challenge Set ID")
                        .action(ArgAction::Append)
                        .required(true)
                        .value_parser(value_parser!(u64)),
                ),
        )
}

#[tokio::main]
async fn main() {
    let mut args = cli().get_matches();
    let generate_arg = args.contains_id("generate_zsh_completions");
    let mut address = args.get_one("address").cloned();
    let release = args.contains_id("release");
    address = address.or(Some(format!(
        "truinsocket_{}{}",
        if release { "" } else { "dev_" },
        env!("CARGO_PKG_VERSION")
    )));
    let args = args.remove_subcommand();
    let (name, mut sub_args) = match args {
        None => {
            if generate_arg {
                generate(Zsh, &mut cli(), "tredit", &mut std::io::stdout());
            } else {
                interactive::interactive().await;
            }
            return;
        }
        Some(args) => args,
    };

    match name.as_str() {
        "set_challenge_sets" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let sets = sub_args
                .remove_many::<u64>("Challenge Set ID")
                .unwrap()
                .collect();
            let config = PartialGameConfig {
                challenge_sets: Some(sets),
                ..Default::default()
            };
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::SetGameConfig(config),
                },
                address,
            )
            .await
        }

        "set_num_catchers" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let num_catchers = sub_args.remove_one::<u64>("Zone").unwrap();
            let config = PartialGameConfig {
                num_catchers: Some(num_catchers),
                ..Default::default()
            };
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::SetGameConfig(config),
                },
                address,
            )
            .await
        }

        "set_start_zone" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let zone = sub_args.remove_one::<u64>("Zone").unwrap();
            let config = PartialGameConfig {
                start_zone: Some(zone),
                ..Default::default()
            };
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::SetGameConfig(config),
                },
                address,
            )
            .await
        }

        "set_end_time" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let hours = sub_args.remove_one::<u32>("Hours").unwrap();
            let minutes = sub_args.remove_one::<u32>("Minutes").unwrap();
            let time = chrono::NaiveTime::from_hms_opt(hours, minutes, 0).unwrap();
            let config = PartialGameConfig {
                end_time: Some(time),
                ..Default::default()
            };
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::SetGameConfig(config),
                },
                address,
            )
            .await
        }

        "set_start_time" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let hours = sub_args.remove_one::<u32>("Hours").unwrap();
            let minutes = sub_args.remove_one::<u32>("Minutes").unwrap();
            let time = chrono::NaiveTime::from_hms_opt(hours, minutes, 0).unwrap();
            let config = PartialGameConfig {
                start_time: Some(time),
                ..Default::default()
            };
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::SetGameConfig(config),
                },
                address,
            )
            .await
        }

        "get_game_config" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::GetGameConfig,
                },
                address,
            )
            .await
        }

        "get_challenge_sets" => {
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::GetChallengeSets,
                },
                address,
            )
            .await
        }

        "get_locations" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::GetLocations,
                },
                address,
            )
            .await
        }

        "get_zones" => {
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::GetAllZones,
                },
                address,
            )
            .await
        }

        "stop" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::Stop,
                },
                address,
            )
            .await
        }

        "start" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::Start,
                },
                address,
            )
            .await
        }

        "get_challenges" => {
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::GetRawChallenges,
                },
                address,
            )
            .await
        }

        "delete_challenges" => {
            if sub_args.contains_id("yes")
                || interactive::get_input("Are you sure (yes/no) ").as_str() == "yes"
            {
                run_command(
                    EngineCommand {
                        session: None,
                        action: EngineAction::DeleteAllChallenges,
                    },
                    address,
                )
                .await
            }
        }

        "import" => {
            let (sender, _recvr) = connect(None).await.unwrap();
            interactive::import_challenges(sender).await
        }

        "rename_team" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let team = sub_args.remove_one::<usize>("Team ID").expect("required");
            let name = sub_args.remove_one::<String>("Name").expect("required");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::RenameTeam {
                        team,
                        new_name: name,
                    },
                },
                address,
            )
            .await
        }

        "add_challenge_to_team" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let team = sub_args.remove_one::<usize>("Team ID").expect("required");
            let title = sub_args.remove_one::<String>("Title").expect("required");
            let description = sub_args
                .remove_one::<String>("Description")
                .expect("required");
            let points = sub_args.remove_one::<u64>("Points").expect("required");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::AddChallengeToTeam {
                        team,
                        challenge: Challenge {
                            title,
                            description,
                            points,
                        },
                    },
                },
                address,
            )
            .await
        }

        "make_runner" => {
            let session = sub_args.remove_one::<u64>("Session ID").expect("required");
            let team = sub_args.remove_one::<usize>("Team ID").expect("required");
            run_command(
                EngineCommand {
                    session: Some(session),
                    action: EngineAction::MakeTeamRunner(team),
                },
                address,
            )
            .await
        }

        "make_catcher" => {
            let session = sub_args.remove_one::<u64>("Session ID").expect("required");
            let team = sub_args.remove_one::<usize>("Team ID").expect("required");
            run_command(
                EngineCommand {
                    session: Some(session),
                    action: EngineAction::MakeTeamCatcher(team),
                },
                address,
            )
            .await
        }

        "get_state" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::GetState,
                },
                address,
            )
            .await
        }

        "assign" => {
            let session = sub_args.remove_one::<u64>("Session ID");
            let player = sub_args.remove_one::<u64>("Player ID").expect("required");
            let team = sub_args.remove_one::<usize>("Team ID");
            run_command(
                EngineCommand {
                    session,
                    action: EngineAction::AssignPlayerToTeam { player, team },
                },
                address,
            )
            .await
        }

        "add_team" => {
            let session = sub_args.get_one::<u64>("Session ID").expect("required");
            let name = sub_args
                .get_one::<String>("Name")
                .expect("required")
                .clone();
            run_command(
                EngineCommand {
                    session: Some(*session),
                    action: EngineAction::AddTeam {
                        name,
                        discord_channel: None,
                        colour: None,
                    },
                },
                address,
            )
            .await
        }

        "add_session" => {
            let name = sub_args
                .get_one::<String>("Name")
                .expect("required")
                .clone();
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::AddSession {
                        name,
                        mode: truinlag::Mode::Traditional,
                    },
                },
                address,
            )
            .await
        }

        "set_player_passphrase" => {
            let id = sub_args.get_one("Player ID").expect("required");
            let passphrase = sub_args
                .get_one::<String>("Passphrase")
                .expect("required")
                .clone();
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::SetPlayerPassphrase {
                        player: *id,
                        passphrase,
                    },
                },
                address,
            )
            .await
        }

        "set_player_session" => {
            let id = sub_args.get_one::<u64>("Player ID").expect("required");
            let session = sub_args.get_one::<u64>("Session ID").cloned();
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::SetPlayerSession {
                        player: *id,
                        session,
                    },
                },
                address,
            )
            .await
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
            let session = sub_args.get_one::<u64>("Session ID").cloned();
            run_command(
                EngineCommand {
                    session: None,
                    action: EngineAction::AddPlayer {
                        name,
                        discord_id: None,
                        passphrase,
                        session,
                    },
                },
                address,
            )
            .await
        }

        _ => unreachable!(),
    }
}
