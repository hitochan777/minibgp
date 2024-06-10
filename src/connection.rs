use anyhow::{Context, Result};
use bytes::{BufMut, BytesMut};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::config::{Config, Mode};
use crate::error::CreateConnectionError;
use crate::packets::message::Message;

#[derive(Debug)]
pub struct Connection {
    conn: TcpStream,
    buffer: BytesMut,
}

const BGP_PORT: u16 = 179;

impl Connection {
    pub async fn connect(config: &Config) -> Result<Self, CreateConnectionError> {
        let conn = match config.mode {
            Mode::Active => Self::connect_to_remote_peer(config).await,
            Mode::Passive => Self::wait_connection_from_remote_peer(config).await,
        }?;
        let buffer = BytesMut::with_capacity(1500);
        Ok(Self { conn, buffer })
    }

    pub async fn send(&mut self, message: Message) {
        let bytes: BytesMut = message.into();
        self.conn.write_all(&bytes[..]).await;
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

    pub async fn get_message(&mut self) -> Option<Message> {
        self.read_data_from_tcp_connection().await;
        let buffer = self.split_buffer_at_message_separator()?;
        Message::try_from(buffer).ok()
    }

    async fn read_data_from_tcp_connection(&mut self) {
        loop {
            let mut buf: Vec<u8> = vec![];
            match self.conn.try_read_buf(&mut buf) {
                Ok(0) => (),
                Ok(n) => self.buffer.put(&buf[..]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("read data from tcp connectionでエラー{:?}が発生しました", e),
            }
        }
    }

    fn get_index_of_message_separator(&self) -> Result<usize> {
        let minimum_message_length = 19;
        if self.buffer.len() < 19 {
            return Err(anyhow::anyhow!(
                "messageのseparatorを表すデータまでbufferに入っていません。\
                 データの受信が半端であることが想定されます。"
            ));
        }
        Ok(u16::from_be_bytes([self.buffer[16], self.buffer[17]]) as usize)
    }

    fn split_buffer_at_message_separator(&mut self) -> Option<BytesMut> {
        let index = self.get_index_of_message_separator().ok()?;
        if self.buffer.len() < index {
            return None;
        }
        Some(self.buffer.split_to(index))
    }
}
