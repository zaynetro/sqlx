use std::net::Shutdown;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::io::{Buf, BufMut, BufStream, MaybeTlsStream};
use crate::mssql::protocol::server::error::Error;
use crate::mssql::protocol::{Decode, Encode, MessageType, PacketHeader, PacketType, Status};
use crate::mssql::MsSql;
use crate::mssql::MsSqlError;
use crate::url::Url;

pub(crate) struct MsSqlStream {
    stream: BufStream<MaybeTlsStream>,
    packet: (PacketType, usize),
    message: Option<(MessageType, usize)>,
}

impl MsSqlStream {
    pub(super) async fn new(url: &Url) -> crate::Result<Self> {
        let stream = MaybeTlsStream::connect(&url, 1433).await?;

        Ok(Self {
            stream: BufStream::new(stream),
            packet: (PacketType::TabularResult, 0),
            message: None,
        })
    }

    pub(super) async fn send<T>(&mut self, packet: T) -> crate::Result<()>
    where
        T: Encode,
    {
        self.write(packet);
        self.flush().await
    }

    pub(super) async fn flush(&mut self) -> crate::Result<()> {
        Ok(self.stream.flush().await?)
    }

    pub(super) fn write<T>(&mut self, packet: T)
    where
        T: Encode,
    {
        // TODO: Support packet chunking for large packet sizes

        let buf = self.stream.buffer_mut();

        // The way PacketHeader is handled here is we encode the entire packet header right away.
        // Then after encoding the packet itself we take the entire length of what we wrote,
        // including the packet header itself. At the end we simply index into the buffer and write
        // over the length. To put it simply, we reserve space for the header and then overwrite the
        // length after the packet has been written.
        let offset = PacketHeader {
            r#type: T::r#type(),
            status: Status::END_OF_MESSAGE,
            length: 0,
            server_process_id: 0,
            packet_id: 1,
            window: 0,
        }
        .encode(buf);

        // Encode the packet data
        packet.encode(buf);

        // Emplace the length of the packet into the header
        let len = buf.len();
        BigEndian::write_u16(&mut buf[offset..offset + 2], len as u16);
    }

    // Receive an object directly from the packet
    // Otherwise known as a token-less stream
    // The caller must know what it is going to receive
    pub(crate) async fn receive<'de, T>(&'de mut self) -> crate::Result<T>
    where
        T: Decode<'de>,
    {
        self.read().await?;

        T::decode(self.packet())
    }

    // Drop any pending tokens and packets
    pub(crate) fn clear(&mut self) {
        if self.packet.1 > 0 {
            self.stream.consume(self.packet.1);
            self.packet.1 = 0;
        }

        self.message = None;
    }

    // Receive the next token from a token stream
    pub(crate) async fn next(&mut self) -> crate::Result<Option<MessageType>> {
        loop {
            if let Some((ty, size)) = self.message.take() {
                if matches!(ty, MessageType::Done) {
                    // TODO: Use bitflags to check if this the FINAL DONE token
                    // If this is the FINAL done token, we should immediately flush
                    self.clear();
                    continue;
                }

                // Consume the packet stream by <size>
                self.stream.consume(size);
                self.packet.1 -= size;
            } else {
                // We have no message, this means that there is no
                // token stream currently being read
                self.read().await?;
            }

            if self.packet.1 == 0 {
                // No more data left in _this_ token stream
                return Ok(None);
            }

            // Decode the type and size of next message in the stream
            let message_ty = MessageType::try_from_u8(self.packet()[0])?;

            self.stream.consume(1);
            self.packet.1 -= 1;

            let message_size = message_ty.size(&mut self.stream, &mut self.packet.1)?;

            self.message = Some((message_ty, message_size as usize));

            match message_ty {
                MessageType::EnvChange | MessageType::Info => {
                    // Ignore ENV_CHANGE and INFO
                    //  ENV_CHANGE tells us when a server variable is changing
                    //  INFO I believe is intended for us to emit as a log
                    continue;
                }

                MessageType::Error => {
                    // Received an ERROR message
                    // This can be anything from a protocol misstep to the SQL query is invalid
                    let error = Error::decode(self.message())?;
                    return Err(MsSqlError(error).into());
                }

                _ => {}
            }

            return Ok(Some(message_ty));
        }
    }

    async fn read(&mut self) -> crate::Result<()> {
        if self.packet.1 > 0 {
            // If there is any data in our packet buffer we need to make sure we flush that
            // so reading will return the *next* message
            self.stream.consume(self.packet.1);
            self.packet.1 = 0;
        }

        let header = self.stream.peek(8).await?;
        let header = PacketHeader::decode(header)?;

        self.packet = (header.r#type, ((header.length as usize) - 8));
        self.stream.consume(8);

        if !matches!(header.r#type, PacketType::TabularResult) {
            // Server-sent messages should *all* have this packet type
            // Anything else is unexpected
            return Err(protocol_err!(
                "unexpected packet type {:?}, expecting TabularResult",
                header.r#type
            )
            .into());
        }

        // Wait until there is enough data in the stream for the entire result packet
        let _ = self.stream.peek(self.packet.1).await?;

        log::trace!("read: {:?}", self.packet());

        // let message_ty = MessageType::try_from_u8(self.packet()[0])?;

        Ok(())
    }

    #[inline]
    fn packet(&self) -> &[u8] {
        &self.stream.buffer()[..(self.packet.1 as usize)]
    }

    #[inline]
    pub(crate) fn message(&self) -> &[u8] {
        if let Some((_, size)) = &self.message {
            &self.stream.buffer()[..*size]
        } else {
            &[]
        }
    }
}
