#![warn(clippy::panic_in_result_fn)]

use super::*;
use bincode;
use futures::{SinkExt, StreamExt};
use std::io::{Error, ErrorKind};
use tokio::{
    io::{AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};
use tokio_rustls::rustls::{
    self,
    pki_types::{CertificateDer, pem::PemObject},
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

type EncryptedStream = tokio_rustls::client::TlsStream<tokio::net::TcpStream>;

pub struct TrainlappcommsSender {
    sender: FramedWrite<WriteHalf<EncryptedStream>, LengthDelimitedCodec>,
}

impl TrainlappcommsSender {
    pub async fn send(&mut self, message: &ToServer) -> Result<(), Error> {
        match self
            .sender
            .send(
                bincode::serialize(message)
                    .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?
                    .into(),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::other(e)),
        }
    }
}

pub struct TrainlappcommsReceiver {
    receiver: FramedRead<ReadHalf<EncryptedStream>, LengthDelimitedCodec>,
}

impl TrainlappcommsReceiver {
    pub async fn recv(&mut self) -> Result<ToApp, Error> {
        bincode::deserialize(&self.receiver.next().await.ok_or(Error::new(
            ErrorKind::ConnectionAborted,
            "connection with trainlappcomms ended",
        ))??)
        .map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("message from trainappcomms couldn't be decoded: {}", e),
            )
        })
    }
}

pub async fn connect(
    root_pem: &[u8],
) -> Result<(TrainlappcommsReceiver, TrainlappcommsSender), Error> {
    // TLS setup
    let root_cert = CertificateDer::from_pem_slice(root_pem).unwrap();
    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add(root_cert).unwrap();
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(std::sync::Arc::new(root_cert_store))
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(config));
    let domain = rustls::pki_types::ServerName::try_from("trainlag.ch")
        .expect("invalid dns name. this is always the same so should never return an error...")
        .to_owned();

    let (rx, tx) = tokio::io::split(
        connector
            .connect(
                domain,
                TcpStream::connect(
                    if cfg!(debug_assertions) || option_env!("TL_DEBUG").is_some() {
                        "trainlag.ch:42314"
                    } else {
                        "trainlag.ch:41314"
                    },
                )
                .await?,
            )
            .await?,
    );
    Ok((
        TrainlappcommsReceiver {
            receiver: FramedRead::new(rx, LengthDelimitedCodec::new()),
        },
        TrainlappcommsSender {
            sender: FramedWrite::new(tx, LengthDelimitedCodec::new()),
        },
    ))
}

pub async fn send_team_picture(
    picture: Vec<u8>,
    session: u64,
    team: usize,
) -> Result<(), std::io::Error> {
    let wrapper = PictureWrapper {
        kind: PictureKind::TeamProfile { session, team },
        picture,
    };
    send_picture(wrapper).await
}

pub async fn send_player_picture(picture: Vec<u8>, player: u64) -> Result<(), std::io::Error> {
    let wrapper = PictureWrapper {
        kind: PictureKind::PlayerProfile(player),
        picture,
    };
    send_picture(wrapper).await
}

pub async fn send_period_picture(
    picture: Vec<u8>,
    session: u64,
    team: usize,
    period_id: usize,
) -> Result<(), std::io::Error> {
    let wrapper = PictureWrapper {
        kind: PictureKind::Period {
            session,
            team,
            period_id,
        },
        picture,
    };
    send_picture(wrapper).await
}

async fn send_picture(pic: PictureWrapper) -> Result<(), std::io::Error> {
    let message = bincode::serialize(&pic).unwrap();
    let mut connection = TcpStream::connect(if cfg!(debug_assertions) {
        "trainlag.ch:42315"
    } else {
        "trainlag.ch:41315"
    })
    .await?;
    connection.write_all(&message).await?;
    connection.shutdown().await?;
    Ok(())
}
