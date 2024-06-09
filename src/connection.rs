use anyhow::{Context, Result};
use tokio::net::{TcpListener, TcpStream};

use crate::config::{Config, Mode};
use crate::error::CreateConnectionError;

#[derive(Debug)]
pub struct Connection {
    conn: TcpStream,
}

const BGP_PORT: u16 = 179;

impl Connection {
    pub async fn connect(config: &Config) -> Result<Self, CreateConnectionError> {
        let conn = match config.mode {
            Mode::Active => Self::connect_to_remote_peer(config).await,
            Mode::Passive => Self::wait_connection_from_remote_peer(config).await,
        }?;
        Ok(Self { conn })
    }

    async fn connect_to_remote_peer(config: &Config) -> Result<TcpStream> {
        TcpStream::connect((config.remote_ip, BGP_PORT))
            .await
            .context(format!(
                "cannot connect to remote peer {0}:{1}",
                config.remote_ip, BGP_PORT
            ))
    }

    async fn wait_connection_from_remote_peer(config: &Config) -> Result<TcpStream> {
        let listener = TcpListener::bind((config.local_ip, BGP_PORT))
            .await
            .context(format!(
                "{0}:{1}にbindすることが出来ませんでした。",
                config.local_ip, BGP_PORT
            ))?;
        Ok(listener
            .accept()
            .await
            .context(format!(
                "{0}:{1}にてリモートからの\
                 TCP Connectionの要求を完遂することが出来ませんでした。\
                 リモートからTCP Connectionの要求が来ていない可能性が高いです。",
                config.local_ip, BGP_PORT
            ))?
            .0)
    }
}
