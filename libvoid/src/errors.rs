// This will handle all errors from the cli part, the "front-end" stuff (ew!)
#[derive(Debug)]
#[allow(dead_code)]
pub enum CliError {
    CommandInvalid(String),
    CommandError(String)
}

impl std::error::Error for CliError {}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CliError::CommandInvalid(cmd) => write!(f, "CommandInvalid: The command `{}' is not known", cmd),
            CliError::CommandError(err) => write!(f, "CommandError: {}", err),
        }
    }
} 


// Handles the errors related to the custom communication protocol
#[derive(Debug)]
#[allow(dead_code)]
pub enum ProtocolError {
    InvalidKey(u32),
    InvalidProtocol(u8),
    PhaseMismatch(u8),
    Encryption(String),
    Command(String),
    ProtocolIssue(String),
    Ping
}

impl std::error::Error for ProtocolError {}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidKey(key) => write!(f, "InvalidKey: The key `{:#08x?}' is invalid", key),
            ProtocolError::InvalidProtocol(id) => write!(f, "InvalidProtocol: The protocol `{:#02x?}' is not known", id),
            ProtocolError::PhaseMismatch(phase) => write!(f, "PhaseMismatch: The phase {} is unknown", phase),
            ProtocolError::Encryption(msg) => write!(f, "Encryption: {}", msg),
            ProtocolError::Command(msg) => write!(f, "Command: {}", msg),
            ProtocolError::ProtocolIssue(msg) => write!(f, "ProtocolIssue: {}", msg),
            ProtocolError::Ping => write!(f, "Error when trying to ping")
        }
    }
} 


// A container to be able to use '?' in adequate functions
#[derive(Debug)]
pub enum VoidError {
    Cli(CliError),
    Protocol(ProtocolError)
}

impl std::error::Error for VoidError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            VoidError::Cli(ref e) => Some(e),
            VoidError::Protocol(ref e) => Some(e)
        }
    }
}

impl std::fmt::Display for VoidError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VoidError::Cli(cli_error) => write!(f, "{}", cli_error),
            VoidError::Protocol(cnc_error) => write!(f, "{}", cnc_error),
        }
    }
}

impl<'a> From<CliError> for VoidError {
    fn from(err: CliError) -> Self {
        VoidError::Cli(err)
    }
}

impl From<ProtocolError> for VoidError {
    fn from(err: ProtocolError) -> Self {
        VoidError::Protocol(err)
    }
}


// result types for cleaner code
#[allow(dead_code)]
pub type VoidResult<T> = Result<T, VoidError>;
#[allow(dead_code)]
pub type CliResult<T> = Result<T, CliError>;
#[allow(dead_code)]
pub type ProtocolResult<T> = Result<T, ProtocolError>;