use std::io::Result as IoResult;

use byteorder::ByteOrder;

/// Extends &[u8] with methods for reading numbers and slices
pub trait ReadSlice<'a> {
    /// Try to read a u8 from self
    fn read_u8(&mut self) -> IoResult<u8>;

    /// Try to read a u16 from self
    fn read_u16<B: ByteOrder>(&mut self) -> IoResult<u16>;

    /// Try to read a u32 from self
    fn read_u32<B: ByteOrder>(&mut self) -> IoResult<u32>;

    /// Try to read a u64 from self
    fn read_u64<B: ByteOrder>(&mut self) -> IoResult<u64>;

    /// Try to read a u128 from self
    fn read_u128<B: ByteOrder>(&mut self) -> IoResult<u128>;

    /// Try to read an unsigned n-bytes integer from self
    fn read_uint32<B: ByteOrder>(&mut self, nb_bytes: usize) -> IoResult<u32>;

    /// Try to read an unsigned n-bytes integer from self
    fn read_uint64<B: ByteOrder>(&mut self, nb_bytes: usize) -> IoResult<u64>;

    /// Try to read an unsigned n-bytes integer from self
    fn read_uint128<B: ByteOrder>(&mut self, nb_bytes: usize) -> IoResult<u128>;

    /// Try to read a i8 from self
    fn read_i8(&mut self) -> IoResult<i8>;

    /// Try to read a i16 from self
    fn read_i16<B: ByteOrder>(&mut self) -> IoResult<i16>;

    /// Try to read a i32 from self
    fn read_i32<B: ByteOrder>(&mut self) -> IoResult<i32>;

    /// Try to read a i64 from self
    fn read_i64<B: ByteOrder>(&mut self) -> IoResult<i64>;

    /// Try to read a i128 from self
    fn read_i128<B: ByteOrder>(&mut self) -> IoResult<i128>;

    /// Try to read a signed n-bytes integer from self
    fn read_int32<B: ByteOrder>(&mut self, nb_bytes: usize) -> IoResult<i32>;

    /// Try to read a signed n-bytes integer from self
    fn read_int64<B: ByteOrder>(&mut self, nb_bytes: usize) -> IoResult<i64>;

    /// Try to read a signed n-bytes integer from self
    fn read_int128<B: ByteOrder>(&mut self, nb_bytes: usize) -> IoResult<i128>;

    /// Try to read a f32 from self
    fn read_f32<B: ByteOrder>(&mut self) -> IoResult<f32>;

    /// Try to read a f64 from self
    fn read_f64<B: ByteOrder>(&mut self) -> IoResult<f64>;

    /// Try to read a slice from self
    fn read_slice(&mut self, nb_bytes: usize) -> IoResult<&'a [u8]>;

    /// Read a slice from start to end of self
    fn read_slice_to_end(&mut self) -> &'a [u8];

    /// Try to move forward in self
    fn move_forward(&mut self, nb_bytes: usize) -> IoResult<()>;
}
