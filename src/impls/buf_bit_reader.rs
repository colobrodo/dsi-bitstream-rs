/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use common_traits::*;

use crate::codes::params::{DefaultReadParams, ReadParams};
use crate::codes::unary_tables;
use crate::traits::*;
use core::{mem, ptr};
use std::error::Error;

/// An internal shortcut to the double type of the word of a
/// [`WordRead`].
type BB<WR> = <<WR as WordRead>::Word as DoubleType>::DoubleType;

/// An implementation of [`BitRead`] and [`BitSeek`] for a [`WordRead`] and a
/// [`WordSeek`].
///
/// This implementation uses a bit buffer to store bits that are not yet read.
/// The buffer is sized as twice the word size of the underlying [`WordRead`].
/// Typically, the best choice is to have a buffer that is sized as `usize`,
/// which means that the word of the underlying [`WordRead`] should be half of
/// that (i.e., `u32` for a 64-bit architecture). However, results will vary
/// depending on the CPU.
///
/// The peek word is equal to the bit buffer. The value returned
/// by [`peek_bits`](crate::traits::BitRead::peek_bits) contains at least as
/// many bits as the word size plus one (extended with zeros beyond end of
/// stream).
///
/// This implementation is usually faster than
/// [`BitReader`](crate::impls::BitReader).
///
/// The additional type parameter `RP` is used to select the parameters for the
/// instantanous codes, but the casual user should be happy with the default
/// value. See [`ReadParams`] for more details.

#[derive(Debug)]
pub struct BufBitReader<E: Endianness, WR: WordRead, RP: ReadParams = DefaultReadParams>
where
    WR::Word: DoubleType,
{
    /// The [`WordRead`] used to fill the buffer.
    backend: WR,
    /// The 2-word bit buffer that is used to read the codes. It is never full,
    /// but it may be empty.
    buffer: BB<WR>,
    /// Number of valid bits in the buffer. It is always smaller than `BB::<WR>::BITS`.
    bits_in_buffer: usize,
    _marker: core::marker::PhantomData<(E, RP)>,
}

impl<E: Endianness, WR: WordRead + Clone, RP: ReadParams> core::clone::Clone
    for BufBitReader<E, WR, RP>
where
    WR::Word: DoubleType,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            buffer: self.buffer,
            bits_in_buffer: self.bits_in_buffer,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<E: Endianness, WR: WordRead, RP: ReadParams> BufBitReader<E, WR, RP>
where
    WR::Word: DoubleType,
    BufBitReader<E, WR, RP>: BitRead<E>,
{
    /// Create a new [`BufBitReader`] around a [`WordRead`].
    ///
    /// # Example
    /// ```
    /// use dsi_bitstream::prelude::*;
    /// let words: [u32; 2] = [0x0043b59f, 0xccf16077];
    /// let word_reader = MemWordReader::new(&words);
    /// let mut buf_bit_reader = <BufBitReader<BE, _>>::new(word_reader);
    /// ```
    #[must_use]
    pub fn new(backend: WR) -> Self {
        check_tables(WR::Word::BITS + 1);
        Self {
            backend,
            buffer: BB::<WR>::ZERO,
            bits_in_buffer: 0,
            _marker: core::marker::PhantomData,
        }
    }

    ///  Return the backend, consuming this reader.
    pub fn into_inner(self) -> Result<WR, <Self as BitRead<E>>::Error> {
        // SAFETY: forget(self) prevents double dropping inner
        let backend = unsafe { ptr::read(&self.backend) };
        mem::forget(self);
        Ok(backend)
    }
}

//
// Big-endian implementation
//

impl<WR: WordRead, RP: ReadParams> BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Ensure that in the buffer there are at least `WR::Word::BITS` bits to read.
    /// This method can be called only if there are at least
    /// `WR::Word::BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<(), <WR as WordRead>::Error> {
        debug_assert!(BB::<WR>::BITS - self.bits_in_buffer >= WR::Word::BITS);

        let new_word: BB<WR> = self.backend.read_word()?.to_be().upcast();
        self.bits_in_buffer += WR::Word::BITS;
        self.buffer |= new_word << (BB::<WR>::BITS - self.bits_in_buffer);
        Ok(())
    }
}

impl<WR: WordRead, RP: ReadParams> BitRead<BE> for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType + UpcastableInto<u64>,
    BB<WR>: CastableInto<u64>,
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = BB<WR>;

    #[inline(always)]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= Self::PeekWord::BITS);

        // A peek can do at most one refill, otherwise we might lose data
        if n_bits > self.bits_in_buffer {
            self.refill()?;
        }

        debug_assert!(n_bits <= self.bits_in_buffer);

        // Move the n_bits highest bits of the buffer to the lowest
        Ok(self.buffer >> (BB::<WR>::BITS - n_bits))
    }

    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n_bits: usize) {
        self.bits_in_buffer -= n_bits;
        self.buffer <<= n_bits;
    }

    #[inline]
    fn read_bits(&mut self, mut n_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);

        // most common path, we just read the buffer
        if n_bits <= self.bits_in_buffer {
            // Valid right shift of BB::<WR>::BITS - n_bits, even when n_bits is zero
            let result: u64 = (self.buffer >> (BB::<WR>::BITS - n_bits - 1) >> 1_u32).cast();
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return Ok(result);
        }

        let mut result: u64 =
            (self.buffer >> (BB::<WR>::BITS - 1 - self.bits_in_buffer) >> 1_u8).cast();
        n_bits -= self.bits_in_buffer;

        // Directly read to the result without updating the buffer
        while n_bits > WR::Word::BITS {
            let new_word: u64 = self.backend.read_word()?.to_be().upcast();
            result = (result << WR::Word::BITS) | new_word;
            n_bits -= WR::Word::BITS;
        }

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= WR::Word::BITS);

        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = WR::Word::BITS - n_bits;
        // compose the remaining bits
        let upcasted: u64 = new_word.upcast();
        let final_bits: u64 = (upcasted >> self.bits_in_buffer).downcast();
        result = (result << (n_bits - 1) << 1) | final_bits;
        // and put the rest in the buffer
        self.buffer = (UpcastableInto::<BB<WR>>::upcast(new_word)
            << (BB::<WR>::BITS - self.bits_in_buffer - 1))
            << 1;

        Ok(result)
    }

    #[inline]
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);
        if USE_TABLE {
            if let Some((res, _)) = unary_tables::read_table_be(self) {
                return Ok(res);
            }
        }

        // count the zeros from the left
        let zeros: usize = self.buffer.leading_zeros() as _;

        // if we encountered an 1 in the bits_in_buffer we can return
        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer << zeros << 1;
            self.bits_in_buffer -= zeros + 1;
            return Ok(zeros as u64);
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.backend.read_word()?.to_be();

            if new_word != WR::Word::ZERO {
                let zeros: usize = new_word.leading_zeros() as _;
                self.buffer =
                    UpcastableInto::<BB<WR>>::upcast(new_word) << (WR::Word::BITS + zeros) << 1;
                self.bits_in_buffer = WR::Word::BITS - zeros - 1;
                return Ok(result + zeros as u64);
            }
            result += WR::Word::BITS as u64;
        }
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<(), Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);
        // happy case, just shift the buffer
        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return Ok(());
        }

        n_bits -= self.bits_in_buffer;

        // skip words as needed
        while n_bits > WR::Word::BITS {
            let _ = self.backend.read_word()?;
            n_bits -= WR::Word::BITS;
        }

        // get the final word
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = WR::Word::BITS - n_bits;

        // TODO
        if self.bits_in_buffer == 0 {
            self.buffer = BB::<WR>::ZERO;
        } else {
            self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word)
                << (BB::<WR>::BITS - self.bits_in_buffer);
        }

        Ok(())
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_to<F: Endianness>(
        &mut self,
        bit_write: &mut impl BitWrite<F>,
        mut n: u64,
    ) -> Result<(), Box<dyn Error>> {
        let from_buffer = Ord::min(n, self.bits_in_buffer as _);
        self.buffer = self.buffer.rotate_left(from_buffer as _);

        bit_write.write_bits(self.buffer.cast(), from_buffer as usize)?;
        n -= from_buffer;

        if n == 0 {
            self.bits_in_buffer -= from_buffer as usize;
            return Ok(());
        }

        while n > WR::Word::BITS as u64 {
            bit_write.write_bits(self.backend.read_word()?.to_be().upcast(), WR::Word::BITS)?;
            n -= WR::Word::BITS as u64;
        }

        assert!(n > 0);
        let new_word = self.backend.read_word()?.to_be();
        self.bits_in_buffer = WR::Word::BITS - n as usize;
        bit_write.write_bits((new_word >> self.bits_in_buffer).upcast(), n as usize)?;
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word)
            .rotate_right(WR::Word::BITS as u32 - n as u32);

        Ok(())
    }
}

impl<
        E: Error + Send + Sync + 'static,
        WR: WordRead<Error = E> + WordSeek<Error = E>,
        RP: ReadParams,
    > BitSeek for BufBitReader<BE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordSeek>::Error;

    #[inline]
    fn get_bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.backend.get_word_pos()? * WR::Word::BITS as u64 - self.bits_in_buffer as u64)
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.backend
            .set_word_pos(bit_index / WR::Word::BITS as u64)?;
        let bit_offset = (bit_index % WR::Word::BITS as u64) as usize;
        self.buffer = BB::<WR>::ZERO;
        self.bits_in_buffer = 0;
        if bit_offset != 0 {
            let new_word: BB<WR> = self.backend.read_word()?.to_be().upcast();
            self.bits_in_buffer = WR::Word::BITS - bit_offset;
            self.buffer = new_word << (BB::<WR>::BITS - self.bits_in_buffer);
        }
        Ok(())
    }
}

//
// Little-endian implementation
//

impl<WR: WordRead, RP: ReadParams> BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    /// Ensure that in the buffer there are at least `WR::Word::BITS` bits to read.
    /// This method can be called only if there are at least
    /// `WR::Word::BITS` free bits in the buffer.
    #[inline(always)]
    fn refill(&mut self) -> Result<(), <WR as WordRead>::Error> {
        debug_assert!(BB::<WR>::BITS - self.bits_in_buffer >= WR::Word::BITS);

        let new_word: BB<WR> = self.backend.read_word()?.to_le().upcast();
        self.buffer |= new_word << self.bits_in_buffer;
        self.bits_in_buffer += WR::Word::BITS;
        Ok(())
    }
}

impl<WR: WordRead, RP: ReadParams> BitRead<LE> for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType + UpcastableInto<u64>,
    BB<WR>: CastableInto<u64>,
{
    type Error = <WR as WordRead>::Error;
    type PeekWord = BB<WR>;

    #[inline(always)]
    fn peek_bits(&mut self, n_bits: usize) -> Result<Self::PeekWord, Self::Error> {
        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= Self::PeekWord::BITS);

        // A peek can do at most one refill, otherwise we might lose data
        if n_bits > self.bits_in_buffer {
            self.refill()?;
        }

        debug_assert!(n_bits <= self.bits_in_buffer);

        // Keep the n_bits lowest bits of the buffer
        let shamt = BB::<WR>::BITS - n_bits;
        Ok((self.buffer << shamt) >> shamt)
    }

    #[inline(always)]
    fn skip_bits_after_table_lookup(&mut self, n_bits: usize) {
        self.bits_in_buffer -= n_bits;
        self.buffer >>= n_bits;
    }

    #[inline]
    fn read_bits(&mut self, mut n_bits: usize) -> Result<u64, Self::Error> {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);

        // most common path, we just read the buffer
        if n_bits <= self.bits_in_buffer {
            let result: u64 = (self.buffer & ((BB::<WR>::ONE << n_bits) - BB::<WR>::ONE)).cast();
            self.bits_in_buffer -= n_bits;
            self.buffer >>= n_bits;
            return Ok(result);
        }

        let mut result: u64 = self.buffer.cast();
        let mut bits_in_res = self.bits_in_buffer;

        // Directly read to the result without updating the buffer
        while n_bits > WR::Word::BITS + bits_in_res {
            let new_word: u64 = self.backend.read_word()?.to_le().upcast();
            result |= new_word << bits_in_res;
            bits_in_res += WR::Word::BITS;
        }

        n_bits -= bits_in_res;

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= WR::Word::BITS);

        // get the final word
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = WR::Word::BITS - n_bits;
        // compose the remaining bits
        let shamt = 64 - n_bits;
        let upcasted: u64 = new_word.upcast();
        let final_bits: u64 = ((upcasted << shamt) >> shamt).downcast();
        result |= final_bits << bits_in_res;
        // and put the rest in the buffer
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> n_bits;

        Ok(result)
    }

    #[inline]
    fn read_unary_param<const USE_TABLE: bool>(&mut self) -> Result<u64, Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);
        if USE_TABLE {
            if let Some((res, _)) = unary_tables::read_table_le(self) {
                return Ok(res);
            }
        }

        // count the zeros from the right
        let zeros: usize = self.buffer.trailing_zeros() as usize;

        // if we encountered an 1 in the bits_in_buffer we can return
        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer >> zeros >> 1;
            self.bits_in_buffer -= zeros + 1;
            return Ok(zeros as u64);
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.backend.read_word()?.to_le();

            if new_word != WR::Word::ZERO {
                let zeros: usize = new_word.trailing_zeros() as _;
                self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> zeros >> 1;
                self.bits_in_buffer = WR::Word::BITS - zeros - 1;
                return Ok(result + zeros as u64);
            }
            result += WR::Word::BITS as u64;
        }
    }

    #[inline]
    fn skip_bits(&mut self, mut n_bits: usize) -> Result<(), Self::Error> {
        debug_assert!(self.bits_in_buffer < BB::<WR>::BITS);
        // happy case, just shift the buffer
        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer >>= n_bits;
            return Ok(());
        }

        n_bits -= self.bits_in_buffer;

        // skip words as needed
        while n_bits > WR::Word::BITS {
            let _ = self.backend.read_word()?;
            n_bits -= WR::Word::BITS;
        }

        // get the final word
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = WR::Word::BITS - n_bits;
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> n_bits;

        Ok(())
    }

    #[cfg(not(feature = "no_copy_impls"))]
    fn copy_to<F: Endianness>(
        &mut self,
        bit_write: &mut impl BitWrite<F>,
        mut n: u64,
    ) -> Result<(), Box<dyn Error>> {
        let from_buffer = Ord::min(n, self.bits_in_buffer as _);

        bit_write.write_bits(self.buffer.cast(), from_buffer as usize)?;

        self.buffer >>= from_buffer;
        n -= from_buffer;

        if n == 0 {
            self.bits_in_buffer -= from_buffer as usize;
            return Ok(());
        }

        while n > WR::Word::BITS as u64 {
            bit_write.write_bits(self.backend.read_word()?.to_le().upcast(), WR::Word::BITS)?;
            n -= WR::Word::BITS as u64;
        }

        assert!(n > 0);
        let new_word = self.backend.read_word()?.to_le();
        self.bits_in_buffer = WR::Word::BITS - n as usize;
        bit_write.write_bits(new_word.upcast(), n as usize)?;
        self.buffer = UpcastableInto::<BB<WR>>::upcast(new_word) >> n;
        Ok(())
    }
}

impl<
        E: Error + Send + Sync + 'static,
        WR: WordRead<Error = E> + WordSeek<Error = E>,
        RP: ReadParams,
    > BitSeek for BufBitReader<LE, WR, RP>
where
    WR::Word: DoubleType,
{
    type Error = <WR as WordSeek>::Error;

    #[inline]
    fn get_bit_pos(&mut self) -> Result<u64, Self::Error> {
        Ok(self.backend.get_word_pos()? * WR::Word::BITS as u64 - self.bits_in_buffer as u64)
    }

    #[inline]
    fn set_bit_pos(&mut self, bit_index: u64) -> Result<(), Self::Error> {
        self.backend
            .set_word_pos(bit_index / WR::Word::BITS as u64)?;

        let bit_offset = (bit_index % WR::Word::BITS as u64) as usize;
        self.buffer = BB::<WR>::ZERO;
        self.bits_in_buffer = 0;
        if bit_offset != 0 {
            let new_word: BB<WR> = self.backend.read_word()?.to_le().upcast();
            self.bits_in_buffer = WR::Word::BITS - bit_offset;
            self.buffer = new_word >> bit_offset;
        }
        Ok(())
    }
}

macro_rules! test_buf_bit_reader {
    ($f: ident, $word:ty) => {
        #[test]
        fn $f() -> Result<(), Box<dyn Error>> {
            use super::MemWordWriterVec;
            use crate::{
                codes::{GammaRead, GammaWrite},
                prelude::{
                    len_delta, len_gamma, BufBitWriter, DeltaRead, DeltaWrite, MemWordReader,
                },
            };
            use rand::Rng;
            use rand::{rngs::SmallRng, SeedableRng};

            let mut buffer_be: Vec<$word> = vec![];
            let mut buffer_le: Vec<$word> = vec![];
            let mut big = super::BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut buffer_be));
            let mut little = BufBitWriter::<LE, _>::new(MemWordWriterVec::new(&mut buffer_le));

            let mut r = SmallRng::seed_from_u64(0);
            const ITER: usize = 1_000_000;

            for _ in 0..ITER {
                let value = r.gen_range(0..128);
                assert_eq!(big.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_gamma(value)?, len_gamma(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_delta(value)?, len_delta(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_delta(value)?, len_delta(value));
                let value = r.gen_range(0..128);
                assert_eq!(big.write_delta(value)?, len_delta(value));
                let value = r.gen_range(0..128);
                assert_eq!(little.write_delta(value)?, len_delta(value));
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    big.write_bits(0, 0)?;
                } else {
                    big.write_bits(1, n_bits)?;
                }
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    little.write_bits(0, 0)?;
                } else {
                    little.write_bits(1, n_bits)?;
                }
                let value = r.gen_range(0..128);
                assert_eq!(big.write_unary_param::<false>(value)?, value as usize + 1);
                let value = r.gen_range(0..128);
                assert_eq!(
                    little.write_unary_param::<false>(value)?,
                    value as usize + 1
                );
                let value = r.gen_range(0..128);
                assert_eq!(big.write_unary_param::<true>(value)?, value as usize + 1);
                let value = r.gen_range(0..128);
                assert_eq!(little.write_unary_param::<true>(value)?, value as usize + 1);
            }

            drop(big);
            drop(little);

            type ReadWord = $word;
            let be_trans: &[ReadWord] = unsafe {
                core::slice::from_raw_parts(
                    buffer_be.as_ptr() as *const ReadWord,
                    buffer_be.len()
                        * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                )
            };
            let le_trans: &[ReadWord] = unsafe {
                core::slice::from_raw_parts(
                    buffer_le.as_ptr() as *const ReadWord,
                    buffer_le.len()
                        * (core::mem::size_of::<$word>() / core::mem::size_of::<ReadWord>()),
                )
            };

            let mut big_buff = BufBitReader::<BE, _>::new(MemWordReader::new(be_trans));
            let mut little_buff = BufBitReader::<LE, _>::new(MemWordReader::new(le_trans));

            let mut r = SmallRng::seed_from_u64(0);

            for _ in 0..ITER {
                assert_eq!(big_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_gamma()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_delta()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_delta()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_delta()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_delta()?, r.gen_range(0..128));
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    assert_eq!(big_buff.read_bits(0)?, 0);
                } else {
                    assert_eq!(big_buff.read_bits(n_bits)?, 1);
                }
                let n_bits = r.gen_range(0..=64);
                if n_bits == 0 {
                    assert_eq!(little_buff.read_bits(0)?, 0);
                } else {
                    assert_eq!(little_buff.read_bits(n_bits)?, 1);
                }

                assert_eq!(big_buff.read_unary()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_unary()?, r.gen_range(0..128));
                assert_eq!(big_buff.read_unary()?, r.gen_range(0..128));
                assert_eq!(little_buff.read_unary()?, r.gen_range(0..128));
            }

            Ok(())
        }
    };
}

test_buf_bit_reader!(test_u64, u64);
test_buf_bit_reader!(test_u32, u32);

test_buf_bit_reader!(test_u16, u16);
