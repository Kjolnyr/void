use crate::{errors::ProtocolError, make_enum};

make_enum!(Protocol<u8> {
    Helo                = 0x00,
    ReturnPhase1Secret  = 0x01,
    SetPhase2Secret     = 0x02,
    ReturnPhase2Secret  = 0x03,
    KeyExchangeOk       = 0x04,
    Ping                = 0x05,
    RequestInfo         = 0x06,
    ReverseShell        = 0x07,
    RequestCmd          = 0x08,
    ReturnCmd           = 0x09,
});