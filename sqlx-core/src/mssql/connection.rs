use crate::connection::{Connect, Connection};
use crate::executor::Executor;
use crate::mssql::protocol::client::login::Login7;
use crate::mssql::protocol::client::pre_login::{Encrypt, PreLogin, Version};
use crate::mssql::protocol::server::done::Done;
use crate::mssql::protocol::server::env_change::EnvChange;
use crate::mssql::protocol::server::info::Info;
use crate::mssql::protocol::server::login_ack::LoginAck;
use crate::mssql::protocol::{Decode, MessageType};
use crate::mssql::stream::MsSqlStream;
use crate::mssql::MsSql;
use crate::url::Url;
use futures_core::future::BoxFuture;
use std::borrow::Cow;
use std::convert::TryInto;

pub struct MsSqlConnection {
    // pub(super) stream: MsSqlStream,
// pub(super) is_ready: bool,
// pub(super) cache_statement: HashMap<Box<str>, u32>,
}

impl MsSqlConnection {
    pub async fn new(url: std::result::Result<Url, url::ParseError>) -> crate::Result<Self> {
        let url = url?;
        let mut stream = MsSqlStream::new(&url).await?;

        establish(&mut stream, &url).await?;

        Ok(MsSqlConnection {})
    }
}

impl Connect for MsSqlConnection {
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<MsSqlConnection>>
    where
        T: TryInto<Url, Error = url::ParseError>,
        Self: Sized,
    {
        Box::pin(MsSqlConnection::new(url.try_into()))
    }
}

impl Connection for MsSqlConnection {
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}

async fn establish(stream: &mut MsSqlStream, url: &Url) -> crate::Result<()> {
    stream
        .send(PreLogin {
            version: Version::default(),
            encryption: Encrypt::NOT_SUPPORTED,

            ..Default::default()
        })
        .await?;

    let pl: PreLogin = stream.receive().await?;

    log::trace!(
        "received PRELOGIN from MSSQL v{}.{}.{}",
        pl.version.major,
        pl.version.minor,
        pl.version.build
    );

    stream
        .send(Login7 {
            version: 0x74000004, // SQL Server 2012 - SQL Server 2019
            client_program_version: 0,
            client_pid: 0,
            packet_size: 4096,
            hostname: "",
            username: "sa",
            password: "Password123!",
            app_name: "",
            server_name: "",
            client_interface_name: "",
            language: "",
            database: "",
            client_id: [0; 6],
        })
        .await?;

    // Wait until a DONE message
    while let Some(ty) = stream.next().await? {
        match ty {
            MessageType::LoginAck => {
                let ack = LoginAck::decode(stream.message())?;

                log::trace!(
                    "established connection to {} {}",
                    ack.program_name,
                    ack.program_version
                );
            }

            MessageType::Done => {
                break;
            }

            ty => {
                return Err(protocol_err!("login: unexpected message {:?}", ty).into());
            }
        }
    }

    Ok(())
}
