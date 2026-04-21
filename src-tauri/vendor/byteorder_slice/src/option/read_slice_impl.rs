use std::ops::Not;

use byteorder::ByteOrder;

use super::traits::ReadSlice;

impl<'a> ReadSlice<'a> for &'a [u8] {
    fn read_u8(&mut self) -> Option<u8> {
        if self.is_empty().not() {
            let b = self[0];
            *self = &self[1..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_u16<B: ByteOrder>(&mut self) -> Option<u16> {
        if self.len() >= 2 {
            let b = B::read_u16(self);
            *self = &self[2..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_u32<B: ByteOrder>(&mut self) -> Option<u32> {
        if self.len() >= 4 {
            let b = B::read_u32(self);
            *self = &self[4..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_u64<B: ByteOrder>(&mut self) -> Option<u64> {
        if self.len() >= 8 {
            let b = B::read_u64(self);
            *self = &self[8..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_u128<B: ByteOrder>(&mut self) -> Option<u128> {
        if self.len() >= 16 {
            let b = B::read_u128(self);
            *self = &self[16..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_uint32<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<u32> {
        if nb_bytes != 0 && nb_bytes <= 4 && self.len() >= nb_bytes {
            let b = B::read_uint(self, nb_bytes) as u32;
            *self = &self[nb_bytes..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_uint64<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<u64> {
        if nb_bytes != 0 && nb_bytes <= 8 && self.len() >= nb_bytes {
            let b = B::read_uint(self, nb_bytes);
            *self = &self[nb_bytes..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_uint128<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<u128> {
        if nb_bytes != 0 && nb_bytes <= 16 && self.len() >= nb_bytes {
            let b = B::read_uint128(self, nb_bytes);
            *self = &self[nb_bytes..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_i8(&mut self) -> Option<i8> {
        if self.is_empty().not() {
            let b = self[0];
            *self = &self[1..];
            Some(b as i8)
        }
        else {
            None
        }
    }

    fn read_i16<B: ByteOrder>(&mut self) -> Option<i16> {
        if self.len() >= 2 {
            let b = B::read_i16(self);
            *self = &self[2..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_i32<B: ByteOrder>(&mut self) -> Option<i32> {
        if self.len() >= 4 {
            let b = B::read_i32(self);
            *self = &self[4..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_i64<B: ByteOrder>(&mut self) -> Option<i64> {
        if self.len() >= 8 {
            let b = B::read_i64(self);
            *self = &self[8..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_i128<B: ByteOrder>(&mut self) -> Option<i128> {
        if self.len() >= 16 {
            let b = B::read_i128(self);
            *self = &self[16..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_int32<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<i32> {
        if nb_bytes != 0 && nb_bytes <= 4 && self.len() >= nb_bytes {
            let b = B::read_int(self, nb_bytes) as i32;
            *self = &self[nb_bytes..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_int64<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<i64> {
        if nb_bytes != 0 && nb_bytes <= 8 && self.len() >= nb_bytes {
            let b = B::read_int(self, nb_bytes);
            *self = &self[nb_bytes..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_int128<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<i128> {
        if nb_bytes != 0 && nb_bytes <= 16 && self.len() >= nb_bytes {
            let b = B::read_int128(self, nb_bytes);
            *self = &self[nb_bytes..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_f32<B: ByteOrder>(&mut self) -> Option<f32> {
        if self.len() >= 4 {
            let b = B::read_f32(self);
            *self = &self[4..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_f64<B: ByteOrder>(&mut self) -> Option<f64> {
        if self.len() >= 8 {
            let b = B::read_f64(self);
            *self = &self[8..];
            Some(b)
        }
        else {
            None
        }
    }

    fn read_slice(&mut self, nb_bytes: usize) -> Option<&'a [u8]> {
        if self.len() >= nb_bytes {
            let res = &self[..nb_bytes];
            *self = &self[nb_bytes..];
            Some(res)
        }
        else {
            None
        }
    }

    fn read_slice_to_end(&mut self) -> &'a [u8] {
        self.read_slice(self.len()).unwrap()
    }

    fn move_forward(&mut self, nb_bytes: usize) -> Option<()> {
        if self.len() >= nb_bytes {
            *self = &self[nb_bytes..];
            Some(())
        }
        else {
            None
        }
    }
}
