use crate::types::string::McString;
use crate::types::McWrite;
use crate::utils::RWStreamWithLimit;
use std::io::{Read, Write};

pub struct StatusProtocol {}

impl StatusProtocol {
    pub fn handle<T: Read + Write>(
        stream: &mut RWStreamWithLimit<T>,
        // bytes_left_in_package: &mut i32,
    ) -> Result<(), ()> {
        McString(Self::get_sample_result())
            .write_stream(stream)
            .map_err(|x| {
                dbg!(x);
            })?;
        stream.discard_unread().map_err(|x| {
            dbg!(x);
        })?;
        // *bytes_left_in_package = 0;
        Ok(())
    }
    fn get_sample_result() -> String {
        "{
            \"version\": {
                \"name\": \"1.21.2\",
                \"protocol\": 768
            },
            \"players\": {
                \"max\": 100,
                \"online\": 5,
                \"sample\": [
                    {
                        \"name\": \"thinkofdeath\",
                        \"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"
                    }
                ]
            },
            \"description\": {
                \"text\": \"Hello, world!\"
            },
            \"favicon\": \"data:image/png;base64,<data>\",
            \"enforcesSecureChat\": false
        }"
        .to_string()
    }
}
