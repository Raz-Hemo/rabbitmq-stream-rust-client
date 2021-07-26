use std::{collections::HashMap, io::Write};

use crate::{
    codec::{Decoder, Encoder},
    error::{DecodeError, EncodeError},
    protocol::commands::COMMAND_OPEN,
    response::ResponseCode,
    types::CorrelationId,
};

use super::Command;

#[derive(PartialEq, Debug)]
pub struct OpenCommand {
    correlation_id: CorrelationId,
    virtual_host: String,
}

impl OpenCommand {
    pub fn new(correlation_id: CorrelationId, virtual_host: String) -> Self {
        Self {
            correlation_id,
            virtual_host,
        }
    }
}

impl Encoder for OpenCommand {
    fn encode(&self, writer: &mut impl Write) -> Result<(), EncodeError> {
        self.correlation_id.encode(writer)?;
        self.virtual_host.as_str().encode(writer)?;
        Ok(())
    }

    fn encoded_size(&self) -> u32 {
        self.correlation_id.encoded_size() + self.virtual_host.len() as u32
    }
}

impl Command for OpenCommand {
    fn key(&self) -> u16 {
        COMMAND_OPEN
    }
}

#[derive(Debug, PartialEq)]
pub struct OpenResponse {
    pub(crate) correlation_id: CorrelationId,
    pub(crate) code: ResponseCode,
    pub(crate) connection_properties: HashMap<String, String>,
}

impl OpenResponse {
    /// Get a reference to the open response's connection properties.
    pub fn connection_properties(&self) -> &HashMap<String, String> {
        &self.connection_properties
    }
}

impl Decoder for OpenResponse {
    fn decode(input: &[u8]) -> Result<(&[u8], Self), DecodeError> {
        let (input, correlation_id) = CorrelationId::decode(input)?;
        let (input, response_code) = ResponseCode::decode(input)?;
        let (input, connection_properties) = Self::decode_map(input)?;

        Ok((
            input,
            OpenResponse {
                correlation_id,
                code: response_code,
                connection_properties,
            },
        ))
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::OpenCommand;
    use crate::{
        codec::{read_u32, Decoder, Encoder},
        commands::open::OpenResponse,
        error::DecodeError,
        ResponseCode,
    };

    impl Decoder for OpenCommand {
        fn decode(input: &[u8]) -> Result<(&[u8], Self), DecodeError> {
            let (input, correlation_id) = read_u32(input)?;
            let (input, virtual_host) = Self::decode_str(input)?;

            Ok((
                input,
                OpenCommand {
                    correlation_id: correlation_id.into(),
                    virtual_host: virtual_host.unwrap(),
                },
            ))
        }
    }

    #[test]
    fn open_request_test() {
        let mut buffer = vec![];

        let open = OpenCommand {
            correlation_id: 1.into(),
            virtual_host: "test".to_owned(),
        };

        let _ = open.encode(&mut buffer);

        let (remaining, decoded) = OpenCommand::decode(&buffer).unwrap();

        assert_eq!(open, decoded);

        assert!(remaining.is_empty());
    }

    impl Encoder for OpenResponse {
        fn encode(
            &self,
            writer: &mut impl std::io::Write,
        ) -> Result<(), crate::error::EncodeError> {
            self.correlation_id.encode(writer)?;
            self.code.encode(writer)?;
            self.connection_properties.encode(writer)?;
            Ok(())
        }

        fn encoded_size(&self) -> u32 {
            0
        }
    }

    #[test]
    fn open_response_test() {
        let mut buffer = vec![];

        let mut properties = HashMap::new();

        properties.insert("test".to_owned(), "test".to_owned());

        let open_response = OpenResponse {
            correlation_id: 1.into(),
            code: ResponseCode::Ok,
            connection_properties: properties,
        };

        let _ = open_response.encode(&mut buffer);

        let (remaining, decoded) = OpenResponse::decode(&buffer).unwrap();

        assert_eq!(open_response, decoded);

        assert!(remaining.is_empty());
    }
}