use prelude::*;

pub struct BitField {
    data: Vec<u8>,
    len: usize,
}

const SET: &'static [u8] = &[
    (1 << 0),
    (1 << 1),
    (1 << 2),
    (1 << 3),
    (1 << 4),
    (1 << 5),
    (1 << 6),
    (1 << 7),
];

const UNSET: &'static [u8] = &[
    255 ^ (1 << 0),
    255 ^ (1 << 1),
    255 ^ (1 << 2),
    255 ^ (1 << 3),
    255 ^ (1 << 4),
    255 ^ (1 << 5),
    255 ^ (1 << 6),
    255 ^ (1 << 7),
];

impl BitField {
    /// Creates a new empty BitField
    pub fn new() -> Self {
        BitField {
            data: Vec::new(),
            len: 0,
        }
    }

    /// Sets a bit in the BifField
    #[inline]
    pub fn set(self: &mut Self, index: usize) {
        let (offset, byte) = self.position(index);
        self.grow(index);
        self.data[byte] |= SET[offset];
    }

    /// Sets a range of bits in the BitField
    pub fn set_range(self: &mut Self, index: usize, len: usize) {
        self.grow(index + len);

        // fill individual bits before full byte range
        let fill_to = cmp::min(Self::to_bits(Self::to_bytes(index) + 1), index + len);
        for i in index..fill_to {
            self.set(i);
        }

        // fill individual bits after full byte range
        if Self::to_bytes(index + len) > Self::to_bytes(index) {
            let fill_from = Self::to_bits(Self::to_bytes(index + len));
            for i in fill_from..(index + len) {
                self.set(i);
            }

            // fill full byte range
            if fill_from > fill_to {
                for i in Self::to_bytes(fill_to)..Self::to_bytes(fill_from) {
                    self.data[i] = 255;
                }
            }
        }
    }

    /// Unsets a bit in the BitField
    #[inline]
    pub fn unset(self: &mut Self, index: usize) {
        let (offset, byte) = self.position(index);
        self.grow(index);
        self.data[byte] &= UNSET[offset];
    }

    /// Unsets a range of bits in the BitField
    pub fn unset_range(self: &mut Self, index: usize, len: usize) {
        self.grow(index + len);

        // fill individual bits before full byte range
        let fill_to = cmp::min( Self::to_bits(Self::to_bytes(index) + 1), index + len );
        for i in index..fill_to {
            self.unset(i);
        }

        // fill individual bits after full byte range
        if Self::to_bytes(index + len) > Self::to_bytes(index) {
            let fill_from = Self::to_bits(Self::to_bytes(index + len));
            for i in fill_from..(index + len) {
                self.unset(i);
            }

            // fill full byte range
            if fill_from > fill_to {
                for i in Self::to_bytes(fill_to)..Self::to_bytes(fill_from) {
                    self.data[i] = 0;
                }
            }
        }
    }

    /// Retrieves a bit from the BitField
    #[inline]
    pub fn get(self: &Self, index: usize) -> bool {
        assert!(index < self.len, "Index out of bounds");
        let (offset, byte) = self.position(index);
        (self.data[byte] & SET[offset]) > 0
    }

    /// Returns the current BitField length
    pub fn len(self: &Self) -> usize {
        self.len
    }

    /// Finds the next matching bit starting at given bit-position.
    pub fn find(self: &Self, mut start: usize, state: bool) -> Option<usize> {
        let byte_match = if state { 0 } else { 255 };
        while let Some(offset) = self.data.iter().skip(Self::to_bytes(start)).position(|&byte| byte != byte_match) {
            let bit_offset = ::std::cmp::max(start, Self::to_bits(offset));
            let bit_next_byte = ::std::cmp::min(self.len, Self::to_bits(Self::to_bytes(bit_offset) + 1));
            for i in bit_offset..bit_next_byte {
                if self.get(i) == state {
                    return Some(i);
                }
            }
            start = Self::to_bits(Self::to_bytes(start) + 1);
        }
        None
    }

    /// Finds the next matching range of bits starting at given bit-position
    pub fn find_range(self: &Self, start: usize, state: bool) -> Option<(usize, usize)> {
        if let Some(range_start) = self.find(start, state) {
            if let Some(range_next) = self.find(range_start + 1, !state) {
                Some((range_start, range_next - range_start))
            } else {
                Some((range_start, self.len() - range_start))
            }
        } else {
            None
        }
    }

    /// Removes all 0 bits and returns a vector that maps each bit's previous position to its new position.
    pub fn compress(self: &mut Self) -> Option<Vec<(usize, usize)>> {
        let mut result = Vec::new();
        let mut position = 0;
        let mut offset = 0;
        let len = self.len();
        while let Some(range) = self.find_range(position, false) {
            offset += range.1;
            position = range.0 + range.1;
            if position < len {
                result.push((position, offset));
            }
            position += 1;
            if range.0 < self.len - offset {
                self.set_range(range.0, range.1);
            }
        }
        let len = self.len;
        self.resize(len - offset);
        if result.len() > 0 { Some(result) } else { None }
    }

    /// Resizes the bitfield to given size. New bits will be zeroed.
    pub fn resize(self: &mut Self, new_size: usize) {
        self.data.resize(Self::to_bytes(new_size + 7), 0);
        self.len = new_size;
    }

    /// Returns byte/offset positions for given index
    #[inline(always)]
    fn position(self: &Self, index: usize) -> (usize, usize) {
        (index & 0b111, Self::to_bytes(index))
    }

    /// Convert given bit count to byte count.
    #[inline(always)]
    fn to_bytes(bytes: usize) -> usize {
        bytes >> 3
    }

    /// Convert given byte count to bit count.
    #[inline(always)]
    fn to_bits(bits: usize) -> usize {
        bits << 3
    }

    /// Grows the unterlying vector to fit max_index +1 bits.
    #[inline(always)]
    fn grow(self: &mut Self, max_index: usize) {
        if max_index > self.len {
            let size = self.data.len();
            let max_byte = Self::to_bytes(max_index);
            if max_byte >= size {
                for _ in size..(max_byte+1) {
                    self.data.push(0);
                }
            }
            self.len = max_index;
        }
    }
}

impl fmt::Debug for BitField {
    fn fmt(self: &Self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BitField [").unwrap();
        if self.len() > 8 {
            write!(f, "\n  ").unwrap();
        }
        for i in 0..self.len() {
            if i > 0 {
                write!(f, ", ").unwrap();
                if (i % 8) == 0 {
                    write!(f, "\n  ").unwrap();
                }
            }
            write!(f, "{:?}", if self.get(i) { 1 } else { 0 }).unwrap();
        }
        if self.len() > 8 {
            write!(f, "\n").unwrap();
        }
        write!(f, "]")
    }
}

mod tests {

    use std::cmp;
    use super::*;
    use super::super::rng::Rng;
/*
    fn init_bitfield() -> BitField {
        let mut bitfield = BitField::new();
        bitfield.set_range(0, 23);
        bitfield.unset_range(23, 3);
        bitfield.set_range(26, 10);
        bitfield
    }

    #[test]
    fn find() {
        let bitfield = init_bitfield();
    }
*/
    #[test]
    fn compress_random() {
        let mut rng = Rng::new(12345.);
        let mut has_one_end = false;
        let mut has_zero_end = false;
        let mut num_ranges = 0;
        for i in 0..100000 {
            let mut bitfield = BitField::new();
            bitfield.resize(rng.range(1, 523));
            let mut next_start = 0;
            let mut total_set = 0;
            //println!("-- RUN {} --", i);
            while next_start < bitfield.len() - cmp::min(bitfield.len(), 7)  {
                let start = rng.range(next_start, cmp::min(bitfield.len() - 1, next_start + 31));
                let len = rng.range(1, bitfield.len() - start + 1 /* +1:Rng.range max is exclusive*/);
                //println!("start: {}, len: {}", start, len);
                bitfield.set_range(start, len);
                next_start = start + len + 1;
                total_set += len;
            }
            if bitfield.get(bitfield.len() -1) { has_one_end = true; }
            if !bitfield.get(bitfield.len() -1) { has_zero_end = true; }
            //println!("{:?} {}", bitfield, bitfield.len());
            let ranges = bitfield.compress();
            num_ranges += ranges.len();
            //println!("{:?} {} (expect: {})", bitfield, bitfield.len(), total_set);
            assert!(bitfield.len() == total_set, "compressed bitfield has invalid length");
            assert!(bitfield.find(0, false) == None, "compressed bitfield contains zeroes");
        }
        assert!(has_one_end, "tests did not generate a single case of a bitfield ending on a 1");
        assert!(has_zero_end, "tests did not generate a single case of a bitfield ending on a 0");
        assert!(num_ranges >= 10000, "tests generated less than 10000 ranges");
    }

}
