use bitflags::bitflags;

// Status is a bit field used to indicate the message state. Status is a 1-byte unsigned char.
// The following Status bit flags are defined.
bitflags! {
    pub struct Status: u8 {
        // "Normal" message.
        const NORMAL = 0x00;

        // End of message (EOM). The packet is the last packet in the whole request.
        const END_OF_MESSAGE = 0x01;

        // (From client to server) Ignore this event (0x01 MUST also be set).
        const IGNORE_EVENT = 0x02;

        // RESETCONNECTION
        //
        // (Introduced in TDS 7.1)
        //
        // (From client to server) Reset this connection
        // before processing event. Only set for event types Batch, RPC, or Transaction Manager
        // request. If clients want to set this bit, it MUST be part of the first packet of the
        // message. This signals the server to clean up the environment state of the connection
        // back to the default environment setting, effectively simulating a logout and a
        // subsequent login, and provides server support for connection pooling. This bit SHOULD
        // be ignored if it is set in a packet that is not the first packet of the message.
        //
        // This status bit MUST NOT be set in conjunction with the RESETCONNECTIONSKIPTRAN bit.
        // Distributed transactions and isolation levels will not be reset.
        const RESET_CONN = 0x08;

        // RESETCONNECTIONSKIPTRAN
        //
        // (Introduced in TDS 7.3)
        //
        // (From client to server) Reset the
        // connection before processing event but do not modify the transaction state (the
        // state will remain the same before and after the reset). The transaction in the
        // session can be a local transaction that is started from the session or it can
        // be a distributed transaction in which the session is enlisted. This status bit
        // MUST NOT be set in conjunction with the RESETCONNECTION bit.
        // Otherwise identical to RESETCONNECTION.
        const RESET_CONN_SKIP_TRAN = 0x10;
    }
}
