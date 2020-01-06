use std::convert::TryFrom;
use std::io;

pub trait InstructionAccumulator {
    fn ensure_bytes(&mut self, bytes: usize) -> io::Result<()>;
    fn get_bytes(&self, offset: usize, length: usize) -> &[u8];

    fn get_byte(&self, offset: usize) -> u8 {
        self.get_bytes(offset, 1)[0]
    }

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
        // To encode a 32 bit number in LEB form can use a maximum of 5 chunks, of which
        // the highest must only use 4 bits
        static HIGHEST_CHUNK: usize = 4;
        static HIGHEST_CHUNK_MASK: u8 = 0x0F;

        let mut pos: usize = offset;
        let mut result: u32 = 0;
        let mut shift = 0;

        loop {
            // It is impossible to go past this point because of the mask check.
            assert!(pos <= (offset + HIGHEST_CHUNK));

            let byte = self.get_byte(pos);

            if pos == (offset + HIGHEST_CHUNK) && (byte & HIGHEST_CHUNK_MASK) != byte {
                panic!("LEB integer is too big");
            }

            pos += 1;
            result |= u32::from(byte & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                return result;
            }
            shift += 7;
        }
    }

    fn get_leb_i32_at(&self, offset: usize) -> i32 {
        // To encode a 32 bit number in LEB form can use a maximum of 5 chunks, of which
        // the highest must only use 4 bits
        static HIGHEST_CHUNK: usize = 4;
        static HIGHEST_CHUNK_MASK: u8 = 0x0F;

        let mut pos: usize = offset;
        let mut result: u32 = 0;
        let mut shift = 0;

        loop {
            // It is impossible to go past this point because of the mask check.
            assert!(pos <= (offset + HIGHEST_CHUNK));

            let byte = self.get_byte(pos);

            if pos == (offset + HIGHEST_CHUNK) && (byte & HIGHEST_CHUNK_MASK) != byte {
                panic!("LEB integer is too big");
            }

            pos += 1;
            result |= u32::from(byte & 0x7f) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                // At this point we have a shift bit unsigned number, so we need to sign extend it.
                // This ought to work.
                let mut result = unsafe { std::mem::transmute(result) };

                if shift < 32 {
                    result = result << (32 - shift);
                    result = result >> (32 - shift);
                }

                return result;
            }
        }
    }

    fn get_leb_u64_at(&self, offset: usize) -> u64 {
        // To encode a 64 bit number in LEB form can use a maximum of 10 chunks, of which
        // the highest must only use 1 bits
        static HIGHEST_CHUNK: usize = 9;
        static HIGHEST_CHUNK_MASK: u8 = 0x01;

        let mut pos: usize = offset;
        let mut result: u64 = 0;
        let mut shift = 0;

        loop {
            // It is impossible to go past this point because of the mask check.
            assert!(pos <= (offset + HIGHEST_CHUNK));

            let byte = self.get_byte(pos);

            if pos == (offset + HIGHEST_CHUNK) && (byte & HIGHEST_CHUNK_MASK) != byte {
                panic!("LEB integer is too big");
            }

            pos += 1;
            result |= u64::from(byte & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                return result;
            }
            shift += 7;
        }
    }

    fn get_leb_i64_at(&self, offset: usize) -> i64 {
        // To encode a 64 bit number in LEB form can use a maximum of 10 chunks, of which
        // the highest must only use 1 bits
        static HIGHEST_CHUNK: usize = 9;
        static HIGHEST_CHUNK_MASK: u8 = 0x01;

        let mut pos: usize = offset;
        let mut result: u64 = 0;
        let mut shift = 0;

        loop {
            // It is impossible to go past this point because of the mask check.
            assert!(pos <= (offset + HIGHEST_CHUNK));

            let byte = self.get_byte(pos);

            if pos == (offset + HIGHEST_CHUNK) && (byte & HIGHEST_CHUNK_MASK) != byte {
                panic!("LEB integer is too big");
            }

            pos += 1;
            result |= u64::from(byte & 0x7f) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                // At this point we have a shift bit unsigned number, so we need to sign extend it.
                // This ought to work.
                let mut result = unsafe { std::mem::transmute(result) };

                if shift < 64 {
                    result = result << (64 - shift);
                    result = result >> (64 - shift);
                }

                return result;
            }
        }
    }

    fn get_leb_usize_at(&self, offset: usize) -> usize {
        usize::try_from(self.get_leb_u32_at(offset)).unwrap()
    }

    fn get_f32_at(&self, offset: usize) -> f32 {
        let mut bytes: [u8; 4] = Default::default();
        bytes.copy_from_slice(self.get_bytes(offset, 4));
        f32::from_le_bytes(bytes.clone())
    }

    fn get_f64_at(&self, offset: usize) -> f64 {
        let mut bytes: [u8; 8] = Default::default();
        bytes.copy_from_slice(self.get_bytes(offset, 8));
        f64::from_le_bytes(bytes.clone())
    }
}

#[derive(Debug)]
pub struct SliceInstructionAccumulator<'a> {
    slice: &'a [u8],
}

impl<'a> InstructionAccumulator for SliceInstructionAccumulator<'a> {
    fn ensure_bytes(&mut self, bytes: usize) -> io::Result<()> {
        if bytes > self.slice.len() {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not enough instruction bytes in expression",
            ))
        } else {
            Ok(())
        }
    }

    fn get_bytes(&self, offset: usize, length: usize) -> &[u8] {
        assert!(offset + length <= self.slice.len());
        &self.slice[offset..offset + length]
    }
}

pub fn make_slice_accumulator<'a>(slice: &'a [u8]) -> SliceInstructionAccumulator<'a> {
    SliceInstructionAccumulator { slice }
}
