use std::str;
use tokio::codec::{Encoder, Decoder};
use tokio::io;
use bytes::BytesMut;


/// Codec here will just read a number of bytes and return it as a message.
pub struct Codec {
    len: usize,
}

impl Codec {
    pub fn new() -> Self {
        Codec { len: 7 }
    }
}

impl Encoder for Codec {

    type Item = String;
    type Error = io::Error;
    fn encode(&mut self, item: String, dst: &mut BytesMut) -> io::Result<()> {

        let to_send = format!("{:width$}", item, width=self.len); 
        dst.extend_from_slice(to_send.as_bytes());

        Ok(())
    }
}

impl Decoder for Codec {

    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<String>, io::Error> {
        if buf.len() >= self.len {
            let line = buf.split_to(self.len);
            let line = str::from_utf8(&line).expect("invalid utf8 data");

            Ok(Some(line.to_string()))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, PartialEq)]
enum ParserState {
    Length,
    Data,
}

/// Netstring is an easy way to frame data on TCP.
/// http://cr.yp.to/proto/netstrings.txt
pub struct NetstringCodec {
    state: ParserState,

    current_length: usize,

    /// Max length for the string. This is to avoid attacks by sending
    /// packets that are too large.
    max_length: usize,

    /// Will disconnect the peer on error if this is true.
    disconnect_on_error: bool,
}

impl NetstringCodec {
    pub fn new(max_length: usize, disconnect_on_error: bool) -> Self {
        NetstringCodec {
            state: ParserState::Length,
            current_length: 0,
            max_length,
            disconnect_on_error,
        }
    }

    fn parse_length(&mut self, buf: &mut BytesMut) -> Result<Option<Vec<u8>>, io::Error> {
        
        // Try to find the current length.
        if self.state == ParserState::Length {
            
            if let Some(colon_offset) = buf.iter().position(|b| *b == b':') {
                // try to extract the length here.
                let length = buf.split_to(colon_offset+1);
                let length = &length[..length.len()-1]; // remove colon from length
                //TODO better
                self.current_length = str::from_utf8(&length).unwrap().parse().unwrap();
                self.state = ParserState::Data;
            } else {
                return Ok(None);
            }
        }

        // In case we have already read the size of the data.
        if self.state == ParserState::Data {
            return self.parse_data(buf);
        }

        Ok(None)
    }

    fn parse_data(&mut self, buf: &mut BytesMut) -> Result<Option<Vec<u8>>, io::Error> {

        if buf.len() >= self.current_length+1 {

            let data = buf.split_to(self.current_length+1);
            // last char should be a comma.
            self.state = ParserState::Length;
            self.current_length = 0;

            return Ok(Some(data.to_vec()));
        }

        Ok(None)
    }
}

impl Encoder for NetstringCodec {

    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> io::Result<()> {
        let item_len = item.len().to_string();
        let len_string = item_len.as_bytes();
        dst.extend_from_slice(len_string);
        dst.extend_from_slice(":".to_string().as_bytes());
        dst.extend_from_slice(&item[..]);
        dst.extend_from_slice(",".to_string().as_bytes());

        Ok(())
    }
}

impl Decoder for NetstringCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Vec<u8>>, io::Error> {
       self.parse_length(buf) 
    }
}

