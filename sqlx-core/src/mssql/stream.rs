use std::net::Shutdown;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::io::{Buf, BufMut, BufStream, MaybeTlsStream};
use crate::mssql::protocol::{Decode, Encode, PacketHeader, PacketType, Status};
use crate::mssql::MsSql;
use crate::mssql::MsSqlError;
use crate::url::Url;

pub(crate) struct MsSqlStream {
    stream: BufStream<MaybeTlsStream>,
    packet: usize,
}

impl MsSqlStream {
    pub(super) async fn new(url: &Url) -> crate::Result<Self> {
        let stream = MaybeTlsStream::connect(&url, 1433).await?;

        Ok(Self {
            stream: BufStream::new(stream),
            packet: 0,
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

    pub(super) async fn receive<'s, T>(&'s mut self) -> crate::Result<T>
    where
        T: Decode<'s>,
    {
        let ty = self.read().await?;

        match ty {
            PacketType::TabularResult => T::decode(self.packet()),

            ty => Err(protocol_err!("unexpected packet type {:?}", ty).into()),
        }
    }

    #[inline]
    pub(super) fn decode<'s, T>(&'s self) -> crate::Result<T>
    where
        T: Decode<'s>,
    {
        T::decode(self.packet())
    }

    pub(super) async fn read(&mut self) -> crate::Result<PacketType> {
        if self.packet > 0 {
            // If there is any data in our packet buffer we need to make sure we flush that
            // so reading will return the *next* message
            self.stream.consume(self.packet);
            self.packet = 0;
        }

        let header = self.stream.peek(8).await?;
        let header = PacketHeader::decode(header)?;

        eprintln!("read[header]: {:?}", header);

        self.packet = (header.length as usize) - 8;
        self.stream.consume(8);

        // Wait until there is enough data in the stream.
        let _ = self.stream.peek(self.packet).await?;

        eprintln!("read: {:?}", crate::io::ByteStr(self.packet()));

        Ok(header.r#type)
    }

    #[inline]
    pub(super) fn packet(&self) -> &[u8] {
        &self.stream.buffer()[..(self.packet as usize)]
    }
}
