// THIS FILE HAS BEEN GENERATED WITH THE SCRIPT gen_code_tables.py
// ~~~~~~~~~~~~~~~~~~~ DO NOT MODIFY ~~~~~~~~~~~~~~~~~~~~~~
// Pre-computed constants used to speedup the reading and writing of zeta codes
use anyhow::Result;
use crate::traits::{BitRead, BitWrite, BE, LE};
use common_traits::*;
/// How many bits are needed to read the tables in this
pub const READ_BITS: usize = 1;
/// The len we assign to a code that cannot be decoded through the table
pub const MISSING_VALUE_LEN: u8 = 255;
/// Maximum value writable using the table(s)
pub const WRITE_MAX: u64 = 255;

#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn read_table_le<B: BitRead<LE>>(backend: &mut B) -> Result<Option<(u64, usize)>> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.upcast();
        let len = READ_LEN_LE[idx as usize];
        if len != MISSING_VALUE_LEN {
            backend.skip_bits_after_table_lookup(len as usize)?;
            return Ok(Some((READ_LE[idx as usize] as u64, len as usize)));
        }
    }
    Ok(None)
}
#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn len_table_le<B: BitRead<LE>>(backend: &mut B) -> Result<Option<usize>> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.upcast();
        let len = READ_LEN_LE[idx as usize];
        if len != MISSING_VALUE_LEN {
            backend.skip_bits_after_table_lookup(len as usize)?;
            return Ok(Some(len as usize));
        }
    }
    Ok(None)
}

#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn write_table_le<B: BitWrite<LE>>(backend: &mut B, value: u64) -> Result<Option<usize>> {
    Ok(if let Some(len) = WRITE_LE.get(value as usize) {
        let len = WRITE_LEN_LE[value as usize] as usize;
        backend.write_bits(len as u64, len)?;
        Some(len)
    } else {
        None
    })
}

#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn read_table_be<B: BitRead<BE>>(backend: &mut B) -> Result<Option<(u64, usize)>> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.upcast();
        let len = READ_LEN_BE[idx as usize];
        if len != MISSING_VALUE_LEN {
            backend.skip_bits_after_table_lookup(len as usize)?;
            return Ok(Some((READ_BE[idx as usize] as u64, len as usize)));
        }
    }
    Ok(None)
}
#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn len_table_be<B: BitRead<BE>>(backend: &mut B) -> Result<Option<usize>> {
    if let Ok(idx) = backend.peek_bits(READ_BITS) {
        let idx: u64 = idx.upcast();
        let len = READ_LEN_BE[idx as usize];
        if len != MISSING_VALUE_LEN {
            backend.skip_bits_after_table_lookup(len as usize)?;
            return Ok(Some(len as usize));
        }
    }
    Ok(None)
}

#[inline(always)]
/// Autogenerated function to lookup a read table, if the result is `Some` the
/// value was found, otherwise we were not able to decode the value and you
/// should fallback to the default implementation
///
/// # Errors
/// This function errors if it wasn't able to skip_bits
pub fn write_table_be<B: BitWrite<BE>>(backend: &mut B, value: u64) -> Result<Option<usize>> {
    Ok(if let Some(len) = WRITE_BE.get(value as usize) {
        let len = WRITE_LEN_BE[value as usize] as usize;
        backend.write_bits(len as u64, len)?;
        Some(len)
    } else {
        None
    })
}
///Table containing the values used to speed up the reading of zeta codes
pub const READ_BE: &[u8] = &[0, 0, ];
///Table contaings the lens used to speed up the reading of zeta codes
pub const READ_LEN_BE: &[u8] = &[255, 255, ];
///Table containing the values used to speed up the reading of zeta codes
pub const READ_LE: &[u8] = &[0, 0, ];
///Table contaings the lens used to speed up the reading of zeta codes
pub const READ_LEN_LE: &[u8] = &[255, 255, ];
///Table used to speed up the writing of zeta codes
pub const WRITE_BE: &[u16] = &[4,10,11,12,13,14,15,32,33,34,35,36,37,38,39,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,256,257,258,259,260,261,262,263,264,265,266,267,268,269,270,271,272,273,274,275,276,277,278,279,280,281,282,283,284,285,286,287,288,289,290,291,292,293,294,295,296,297,298,299,300,301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316,317,318,319,640,641,642,643,644,645,646,647,648,649,650,651,652,653,654,655,656,657,658,659,660,661,662,663,664,665,666,667,668,669,670,671,672,673,674,675,676,677,678,679,680,681,682,683,684,685,686,687,688,689,690,691,692,693,694,695,696,697,698,699,700,701,702,703,704,705,706,707,708,709,710,711,712,713,714,715,716,717,718,719,720,721,722,723,724,725,726,727,728,729,730,731,732,733,734,735,736,737,738,739,740,741,742,743,744,745,746,747,748,749,750,751,752,753,754,755,756,757,758,759,760,761,762,763,764,765,766,767,768,];
///Table used to speed up the writing of zeta codes
pub const WRITE_LEN_BE: &[u16] = &[3, 4, 4, 4, 4, 4, 4, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, ];
///Table used to speed up the writing of zeta codes
pub const WRITE_LE: &[u16] = &[1,3,11,5,13,7,15,2,6,10,14,18,22,26,30,34,162,38,166,42,170,46,174,50,178,54,182,58,186,62,190,66,194,70,198,74,202,78,206,82,210,86,214,90,218,94,222,98,226,102,230,106,234,110,238,114,242,118,246,122,250,126,254,4,12,20,28,36,44,52,60,68,76,84,92,100,108,116,124,132,140,148,156,164,172,180,188,196,204,212,220,228,236,244,252,260,268,276,284,292,300,308,316,324,332,340,348,356,364,372,380,388,396,404,412,420,428,436,444,452,460,468,476,484,492,500,508,516,2564,524,2572,532,2580,540,2588,548,2596,556,2604,564,2612,572,2620,580,2628,588,2636,596,2644,604,2652,612,2660,620,2668,628,2676,636,2684,644,2692,652,2700,660,2708,668,2716,676,2724,684,2732,692,2740,700,2748,708,2756,716,2764,724,2772,732,2780,740,2788,748,2796,756,2804,764,2812,772,2820,780,2828,788,2836,796,2844,804,2852,812,2860,820,2868,828,2876,836,2884,844,2892,852,2900,860,2908,868,2916,876,2924,884,2932,892,2940,900,2948,908,2956,916,2964,924,2972,932,2980,940,2988,948,2996,956,3004,964,3012,972,3020,980,3028,988,3036,996,3044,1004,3052,1012,3060,1020,3068,1028,];
///Table used to speed up the writing of zeta codes
pub const WRITE_LEN_LE: &[u16] = &[3, 4, 4, 4, 4, 4, 4, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, ];
///Table used to speed up the skipping of zeta codes
pub const LEN: &[u8] = &[3, 4, 4, 4, 4, 4, 4, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, ];
/// The K of the zeta codes for these tables
pub const K: u64 = 3;