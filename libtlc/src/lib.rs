use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

pub use libtruinlag::{
    Challenge, CompletedChallenge, DetailedLocation, Event, MinimalLocation, Picture, Player, Team,
    TeamRole,
};

pub mod api;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToServer {
    Login(String),
    Location(DetailedLocation),
    AttachPeriodPictures {
        event_id: usize,
        pictures: Vec<Vec<u8>>,
    },
    UploadPlayerPicture(Vec<u8>),
    UploadTeamPicture(Vec<u8>),
    Complete {
        completed_id: usize,
        period_id: usize,
    },
    Catch {
        caught_id: usize,
        period_id: usize,
    },
    RequestEverything,
    Ping(Option<String>),
    RequestPictures(Vec<u64>),
    RequestThumbnails(Vec<u64>),
    RequestPastLocations {
        of_past_seconds: Option<NonZeroU32>,
        team_id: usize,
    },
    SetPhoneNumber(Option<String>),
    SetTeamName(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToServerPackage {
    contents: ToServer,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Everything {
    pub state: State,
    pub teams: Vec<Team>,
    pub events: Vec<Event>,
    pub you: u64,
    pub your_team: usize,
    pub your_session: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToApp {
    Everything(Everything),
    LoginSuccessful(bool),
    Ping(Option<String>),
    BecomeCatcher(Everything), // I'm too lazy for notifications here
    BecomeRunner(Everything),
    ChallengeCompleted(Event, Everything),
    BecomeNoGameRunning(Everything),
    BecomeShutDown,
    Location {
        team: usize,
        location: DetailedLocation,
    },
    AddedPeriod(usize),
    Pictures(Vec<Picture>),
    Error(ClientError),
    SendPastLocations {
        team: usize,
        locations: Vec<MinimalLocation>,
    },
    GameStarted(Everything),
    EventOccurred(Event, Everything),
    YouLeftGracePeriod(Everything),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientError {
    NotFound(String),     // Element you were looking for wasn't found
    TeamExists(String),   // You cannot create a team if one with a similar name already exists
    AlreadyExists,        // Things that already exist cannot be created
    GameInProgress,       // Commands like AddTeam cannot be run if a game is in progress
    GameNotRunning,       // Commands like catch can only be run if a game is in progress
    AmbiguousData,        // If multiple matching objects exist, e.g. players with passphrase lol
    InternalError,        // Some sort of internal database error
    NotImplemented,       // Feature is not yet implemented
    TeamIsRunner(usize),  // A relevant team is runner, but has to be catcher
    TeamIsCatcher(usize), // A relevant team is catcher, but has to be runner
    TeamsTooFar,          // Two relevant teams are too far away from each other
    BadData(String),
    TextError(String), // Some other kind of error with a custom text
    PictureProblem,    // An Image-related error
    TooRapid,          // When requests are sent too rapidly
    TooFewChallenges,  // When there are too few challenges to start a game
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(ctx) => write!(f, "{} was not found", ctx),
            Self::TeamExists(team) => write!(f, "Team {} already exists", team),
            Self::AlreadyExists => write!(f, "Already exists"),
            Self::GameInProgress => write!(f, "There is already a game in progress"),
            Self::GameNotRunning => write!(f, "There is no game in progress"),
            Self::AmbiguousData => write!(f, "Ambiguous data"),
            Self::InternalError => write!(f, "There was a truinlag-internal error"),
            Self::NotImplemented => write!(f, "Not yet implemented"),
            Self::TeamIsRunner(team) => write!(f, "team {} is runner", team),
            Self::TeamIsCatcher(team) => write!(f, "team {} is catcher", team),
            Self::TeamsTooFar => write!(f, "the teams are too far away from each other"),
            Self::BadData(text) => write!(f, "bad data: {}", text),
            Self::TextError(text) => write!(f, "{}", text),
            Self::PictureProblem => write!(f, "there was a problem processing an image"),
            Self::TooRapid => write!(f, "not enough time has passed since the last request"),
            Self::TooFewChallenges => write!(
                f,
                "there are not enough challenges to start a game in the challenge db"
            ),
        }
    }
}

impl TryFrom<libtruinlag::commands::Error> for ClientError {
    type Error = ();
    fn try_from(value: libtruinlag::commands::Error) -> Result<Self, Self::Error> {
        use libtruinlag::commands::Error::*;
        match value {
            NoSessionSupplied => Err(()),
            SessionSupplied => Err(()),
            NotFound(text) => Ok(Self::NotFound(text)),
            TeamExists(team) => Ok(Self::TeamExists(team)),
            AlreadyExists => Ok(Self::AlreadyExists),
            GameInProgress => Ok(Self::GameInProgress),
            GameNotRunning => Ok(Self::GameNotRunning),
            AmbiguousData => Ok(Self::AmbiguousData),
            InternalError => Ok(Self::InternalError),
            NotImplemented => Ok(Self::NotImplemented),
            TeamIsRunner(team) => Ok(Self::TeamIsRunner(team)),
            TeamIsCatcher(team) => Ok(Self::TeamIsCatcher(team)),
            TeamsTooFar => Ok(Self::TeamsTooFar),
            BadData(text) => Ok(Self::BadData(text)),
            TextError(text) => Ok(Self::TextError(text)),
            PictureProblem => Ok(Self::PictureProblem),
            TooRapid => Ok(Self::TooRapid),
            TooFewChallenges => Ok(Self::TooFewChallenges),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToAppPackage {
    contents: ToApp,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum State {
    GameNotRunning,
    Runner,
    Catcher,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PictureWrapper {
    pub kind: PictureKind,
    pub picture: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PictureKind {
    TeamProfile {
        session: u64,
        team: usize,
    },
    PlayerProfile(u64),
    Period {
        session: u64,
        team: usize,
        period_id: usize,
    },
}
