use byteorder::ByteOrder;

/// Extends &[u8] with methods for reading numbers and slices
pub trait ReadSlice<'a> {
    /// Try to read a u8 from self
    fn read_u8(&mut self) -> Option<u8>;

    /// Try to read a u16 from self
    fn read_u16<B: ByteOrder>(&mut self) -> Option<u16>;

    /// Try to read a u32 from self
    fn read_u32<B: ByteOrder>(&mut self) -> Option<u32>;

    /// Try to read a u64 from self
    fn read_u64<B: ByteOrder>(&mut self) -> Option<u64>;

    /// Try to read a u128 from self
    fn read_u128<B: ByteOrder>(&mut self) -> Option<u128>;

    /// Try to read an unsigned n-bytes integer from self
    fn read_uint32<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<u32>;

    /// Try to read an unsigned n-bytes integer from self
    fn read_uint64<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<u64>;

    /// Try to read an unsigned n-bytes integer from self
    fn read_uint128<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<u128>;

    /// Try to read a i8 from self
    fn read_i8(&mut self) -> Option<i8>;

    /// Try to read a i16 from self
    fn read_i16<B: ByteOrder>(&mut self) -> Option<i16>;

    /// Try to read a i32 from self
    fn read_i32<B: ByteOrder>(&mut self) -> Option<i32>;

    /// Try to read a i64 from self
    fn read_i64<B: ByteOrder>(&mut self) -> Option<i64>;

    /// Try to read a i128 from self
    fn read_i128<B: ByteOrder>(&mut self) -> Option<i128>;

    /// Try to read a signed n-bytes integer from self
    fn read_int32<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<i32>;

    /// Try to read a signed n-bytes integer from self
    fn read_int64<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<i64>;

    /// Try to read a signed n-bytes integer from self
    fn read_int128<B: ByteOrder>(&mut self, nb_bytes: usize) -> Option<i128>;

    /// Try to read a f32 from self
    fn read_f32<B: ByteOrder>(&mut self) -> Option<f32>;

    /// Try to read a f64 from self
    fn read_f64<B: ByteOrder>(&mut self) -> Option<f64>;

    /// Try to read a slice from self
    fn read_slice(&mut self, nb_bytes: usize) -> Option<&'a [u8]>;

    /// Read a slice from start to end of self
    fn read_slice_to_end(&mut self) -> &'a [u8];

    /// Try to move forward in self
    #[must_use]
    fn move_forward(&mut self, nb_bytes: usize) -> Option<()>;
}
