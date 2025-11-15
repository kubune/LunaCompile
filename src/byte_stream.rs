use std::str;

use crate::encrypter::Encrypter;

pub struct ByteStream {
    pub buffer: Vec<u8>,
    pub offset: usize,
    pub bit_offset: u8,
}

impl ByteStream {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            bit_offset: 0,
        }
    }

    pub fn ensure_capacity(&mut self, capacity: usize) {
        let buffer_length = self.buffer.len();
        if self.offset + capacity > buffer_length {
            let mut new_buffer = vec![0; buffer_length + capacity];
            new_buffer[..buffer_length].copy_from_slice(&self.buffer);
            self.buffer = new_buffer;
        }
    }

    pub fn read_int(&mut self) -> i32 {
        self.bit_offset = 0;
        let val = ((self.buffer[self.offset] as i32) << 24)
            | ((self.buffer[self.offset + 1] as i32) << 16)
            | ((self.buffer[self.offset + 2] as i32) << 8)
            | (self.buffer[self.offset + 3] as i32);
        self.offset += 4;
        val
    }

    pub fn skip(&mut self, len: usize) {
        self.bit_offset += len as u8;
    }

    pub fn read_short(&mut self) -> i32 {
        self.bit_offset = 0;
        let val = ((self.buffer[self.offset] as i32) << 8)
            | (self.buffer[self.offset + 1] as i32);
        self.offset += 2;
        val
    }

    pub fn write_short(&mut self, value: i32) {
        self.bit_offset = 0;
        self.ensure_capacity(2);
        self.buffer[self.offset] = ((value >> 8) & 0xFF) as u8;
        self.buffer[self.offset + 1] = (value & 0xFF) as u8;
        self.offset += 2;
    }

    pub fn write_int(&mut self, value: i32) {
        self.bit_offset = 0;
        self.ensure_capacity(4);
        self.buffer[self.offset] = ((value >> 24) & 0xFF) as u8;
        self.buffer[self.offset + 1] = ((value >> 16) & 0xFF) as u8;
        self.buffer[self.offset + 2] = ((value >> 8) & 0xFF) as u8;
        self.buffer[self.offset + 3] = (value & 0xFF) as u8;
        self.offset += 4;
    }

    pub fn write_int_zero(&mut self) {
        self.write_int(0);
    }

    pub fn write_string(&mut self, string: &str) {
        let encrypted_string = Encrypter::encrypt_xor_str(string, b"LUNALOAD1");
        let value: Option<&str> = Some(&encrypted_string);

        if value.is_none() || value.unwrap().len() > 90000 {
            self.write_int(-1);
            return;
        }

        let str_bytes = value.unwrap().as_bytes();
        self.write_int(str_bytes.len() as i32);
        self.ensure_capacity(str_bytes.len());
        for b in str_bytes {
            self.buffer[self.offset] = *b;
            self.offset += 1;
        }
    }

    pub fn write_string_empty(&mut self) {
        self.write_string("");
    }

    pub fn read_string(&mut self) -> String {
        let length = self.read_int();
        if length > 0 && length < 90000 {
            if self.offset + length as usize > self.buffer.len() {
                return "".to_string();
            }
            let s = str::from_utf8(&self.buffer[self.offset..self.offset + length as usize])
                .unwrap_or("")
                .to_string();
            self.offset += length as usize;
            s
        } else {
            "".to_string()
        }
    }

    pub fn read_data_reference(&mut self) -> [i32; 2] {
        let a1 = self.read_vint();
        [a1, if a1 == 0 { 0 } else { self.read_vint() }]
    }

    pub fn write_data_reference(&mut self, value1: i32, value2: i32) {
        if value1 < 1 {
            self.write_vint(0);
        } else {
            self.write_vint(value1);
            self.write_vint(value2);
        }
    }

    pub fn write_vint(&mut self, mut value: i32) {
        self.bit_offset = 0;
        let mut temp = ((value >> 25) & 0x40) as u8;
        let mut flipped = value ^ (value >> 31);
        temp |= (value & 0x3F) as u8;
        value >>= 6;
        flipped >>= 6;

        if flipped == 0 {
            self.write_byte(temp);
            return;
        }

        self.write_byte(temp | 0x80);
        flipped >>= 7;
        let mut r = if flipped != 0 { 0x80 } else { 0 };
        self.write_byte(((value & 0x7F) as u8) | r);
        value >>= 7;

        while flipped != 0 {
            flipped >>= 7;
            r = if flipped != 0 { 0x80 } else { 0 };
            self.write_byte(((value & 0x7F) as u8) | r);
            value >>= 7;
        }
    }

    pub fn write_vint_zero(&mut self) {
        self.write_vint(0);
    }

    pub fn read_vint(&mut self) -> i32 {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let mut b = self.buffer[self.offset] as i32;
            self.offset += 1;

            if shift == 0 {
                let a1 = (b & 0x40) >> 6;
                let a2 = (b & 0x80) >> 7;
                let s = (b << 1) & !0x181;
                b = s | (a2 << 7) | a1;
            }

            result |= (b & 0x7F) << shift;
            shift += 7;

            if (b & 0x80) == 0 {
                break;
            }
        }

        (result >> 1) ^ (-(result & 1))
    }

    pub fn write_boolean(&mut self, value: bool) {
        if self.bit_offset == 0 {
            self.ensure_capacity(1);
            self.buffer[self.offset] = 0;
            self.offset += 1;
        }
        if value {
            let idx = self.offset - 1;
            self.buffer[idx] |= 1 << self.bit_offset;
        }
        self.bit_offset = (self.bit_offset + 1) & 7;
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_vint() >= 1
    }

    pub fn write_hex(&mut self, hex: Option<&str>) {
        self.bit_offset = 0;
        if let Some(mut data) = hex {
            if data.starts_with("0x") {
                data = &data[2..];
            }
            let cleaned_data = data.replace(&[' ', '-'][..], "");
            self.ensure_capacity(cleaned_data.len() / 2);
            let mut i = 0;
            while i < cleaned_data.len() {
                let byte = u8::from_str_radix(&cleaned_data[i..i + 2], 16).unwrap();
                self.write_byte(byte);
                i += 2;
            }
        }
    }

    pub fn write_string_reference(&mut self, value: &str) {
        self.write_string(value);
    }

    pub fn write_string_reference_empty(&mut self) {
        self.write_string("");
    }

    pub fn write_long_long(&mut self, value: i64) {
        self.write_int((value >> 32) as i32);
        self.write_int(value as i32);
    }

    pub fn write_logic_long(&mut self, value1: i32, value2: i32) {
        self.write_vint(value1);
        self.write_vint(value2);
    }

    pub fn read_logic_long(&mut self) -> [i32; 2] {
        [self.read_vint(), self.read_vint()]
    }

    pub fn write_long(&mut self, value1: i32, value2: i32) {
        self.write_int(value1);
        self.write_int(value2);
    }

    pub fn read_long(&mut self) -> [i32; 2] {
        [self.read_int(), self.read_int()]
    }

    pub fn write_byte(&mut self, value: u8) {
        self.bit_offset = 0;
        self.ensure_capacity(1);
        self.buffer[self.offset] = value;
        self.offset += 1;
    }

    pub fn write_bytes(&mut self, bytes: Option<&[u8]>) {
        match bytes {
            Some(b) => {
                self.write_int(b.len() as i32);
                self.ensure_capacity(b.len());
                for byte in b {
                    self.buffer[self.offset] = *byte;
                    self.offset += 1;
                }
            }
            None => self.write_int(-1),
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.offset = 0;
        self.bit_offset = 0;
    }

    pub fn get_length(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn replace_buffer(&mut self, b: Vec<u8>) -> &Vec<u8> {
        self.offset = 0;
        self.buffer = b;
        &self.buffer
    }
}
