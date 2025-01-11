use std::simd::u16x2;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
pub struct BytePacketBuffer {
    pub buffer: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> BytePacketBuffer {
        BytePacketBuffer { buffer: [0; 512], pos: 0 }
    }

    fn pos(&self) -> usize {
        self.pos
    } 

    fn step(&mut self, n: usize) {
        self.pos += n;
    }

    fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }
    //read a single byte
    fn read(&mut self) -> Result<u8> {
        if self.pos > 512 {
            return Err("exceede buffer size".into())
        }

        let res = self.buffer[self.pos];
        self.pos += 1;
        Ok(res)
    }

    fn get(&self, pos: usize) -> Result<u8> {
        if pos > 512 {
            return Err("exceede buffer size".into())
        }

        Ok(self.buffer[pos])
    }

    fn get_range(&self, start: usize, len: usize) -> Result<&[u8]> {
        if start > 512 || start + len > 512 {
            return Err("exceede buffer size".into())
        }

        Ok(&self.buffer[start..start + len])
    }

    /// Read two bytes, stepping two steps forward
    fn read_u16(&mut self) -> Result<u16> {
        let high_byte = self.read()? as u16;
        let low_byte = self.read()? as u16;
        let res = (high_byte << 8) | low_byte;

        Ok(res)
    }

    /// Read four bytes, stepping four steps forward
    fn read_u32(&mut self) -> Result<u32> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | ((self.read()? as u32) << 0);

        Ok(res)
    }

    fn read_query_name(&mut self) -> Result<String> {
        let mut res = String::new();
        let mut pos = self.pos();

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        loop {
            if jumps_performed > max_jumps {
                return Err("too many jumps in query name".into())
            }
            //Now we are at the beginning of the label, with the length byte starting at pos
            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos + 2);
                }
                // Read another byte, calculate offset and perform the jump by
                // updating our local position variable
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                // Indicate that a jump was performed.
                jumped = true;
                jumps_performed += 1;

                continue;
            } else {
                pos += 1;
                
                if len == 0 {
                    break;
                }
                //TODO: check if this is correct
                res.push_str("");
                let str_buffer = self.get_range(pos, len as usize)?;
                res.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());
            }
        }

        if !jumped {
            self.seek(pos);
        }
        Ok(res)
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl ResultCode {
    pub fn from_num(num: u8) -> ResultCode {
        match num {
            1 => ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            0 | _ => ResultCode::NOERROR,
        }
    }
}

pub struct DnsHeader {
    pub id: u16,

    pub recursion_desired: bool,    // 1 bit
    pub truncated_message: bool,    // 1 bit
    pub authoritative_answer: bool, // 1 bit
    pub opcode: u8,                 // 4 bits
    pub response: bool,             // 1 bit

    pub rescode: ResultCode,       // 4 bits
    pub checking_disabled: bool,   // 1 bit
    pub authed_data: bool,         // 1 bit
    pub z: bool,                   // 1 bit
    pub recursion_available: bool, // 1 bit

    pub questions: u16,             // 16 bits
    pub answers: u16,               // 16 bits
    pub authoritative_entries: u16, // 16 bits
    pub resource_entries: u16,      // 16 bits
}

impl DnsHeader {
    pub fn new() -> DnsHeader {
        DnsHeader {
            id: 0,
            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            opcode: 0,
            response: false,
            rescode: ResultCode::NOERROR,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,
            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            resource_entries: 0,
        }
    }
    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let a = (flags >> 8) as u8;
        let b = (flags & 0xFF) as u8;
        self.recursion_desired = (a & (1 << 0)) > 0;
        self.truncated_message = (a & (1 << 1)) > 0;
        self.authoritative_answer = (a & (1 << 2)) > 0;
        self.opcode = (a >> 3) & 0x0F;
        self.response = (a & (1 << 7)) > 0;

        self.rescode = ResultCode::from_num(b & 0x0F);
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.authed_data = (b & (1 << 5)) > 0;
        self.z = (b & (1 << 6)) > 0;
        self.recursion_available = (b & (1 << 7)) > 0;

        self.questions = buffer.read_u16()?;
        self.answers = buffer.read_u16()?;
        self.authoritative_entries = buffer.read_u16()?;
        self.resource_entries = buffer.read_u16()?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum QueryType {
    UNKNOWN(u16),
    A,
}
impl QueryType {
    pub fn from_num(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            _ => QueryType::UNKNOWN(num),
        }
    }
    pub fn to_num(&self) -> u16 {
        match *self {
            QueryType::UNKNOWN(num) => num,
            QueryType::A => 1,
        }
    }
}

pub struct DnsQuestion {
    pub
}