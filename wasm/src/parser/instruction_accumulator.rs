use std::io;

pub trait InstructionAccumulator {
    fn ensure_bytes(&mut self, bytes: usize) -> io::Result<()>;
    fn get_byte(&self, offset: usize) -> u8;

    fn ensure_leb_at(&mut self, offset: usize) -> io::Result<usize> {
        let mut number_length: usize = 1;
        loop {
            self.ensure_bytes(offset + number_length)?;

            if 0 == (self.get_byte(offset + number_length - 1) & 0x80) {
                return Ok(number_length);
            }

            number_length += 1;
        }
    }

    fn get_leb_u32_at(&self, offset: usize) -> u32 {
        let mut pos: usize = offset;
        let mut result: u32 = 0;
        let mut shift = 0;

        loop {
            let byte = self.get_byte(pos);
            pos += 1;
            result |= u32::from(byte & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                return result;
            }
            shift += 7;
        }
    }
}
