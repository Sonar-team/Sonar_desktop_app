use byteorder_slice::option::ReadSlice;
use byteorder_slice::{BigEndian, LittleEndian};
use hex_literal::hex;

#[test]
fn u8() {
    let mut data = hex!("01 80").as_ref();
    assert!(matches!(data.read_u8(), Some(1)));
    assert!(matches!(data.read_u8(), Some(128)));
    assert!(matches!(data.read_u8(), None));
}

#[test]
fn i8() {
    let mut data = hex!("01 80").as_ref();
    assert!(matches!(data.read_i8(), Some(1)));
    assert!(matches!(data.read_i8(), Some(-128)));
    assert!(matches!(data.read_i8(), None));
}

#[test]
fn u16_be() {
    let mut data = hex!("0102 8000").as_ref();
    assert!(matches!(data.read_u16::<BigEndian>(), Some(258)));
    assert!(matches!(data.read_u16::<BigEndian>(), Some(32768)));
    assert!(matches!(data.read_u16::<BigEndian>(), None));
}

#[test]
fn u16_le() {
    let mut data = hex!("0201 0080").as_ref();
    assert!(matches!(data.read_u16::<LittleEndian>(), Some(258)));
    assert!(matches!(data.read_u16::<LittleEndian>(), Some(32768)));
    assert!(matches!(data.read_u16::<LittleEndian>(), None));
}

#[test]
fn i16_be() {
    let mut data = hex!("0102 8000").as_ref();
    assert!(matches!(data.read_i16::<BigEndian>(), Some(258)));
    assert!(matches!(data.read_i16::<BigEndian>(), Some(-32768)));
    assert!(matches!(data.read_i16::<BigEndian>(), None));
}

#[test]
fn i16_le() {
    let mut data = hex!("0201 0080").as_ref();
    assert!(matches!(data.read_i16::<LittleEndian>(), Some(258)));
    assert!(matches!(data.read_i16::<LittleEndian>(), Some(-32768)));
    assert!(matches!(data.read_i16::<LittleEndian>(), None));
}

#[test]
fn u32_be() {
    let mut data = hex!("01020304 80000000").as_ref();
    assert!(matches!(data.read_u32::<BigEndian>(), Some(16909060)));
    assert!(matches!(data.read_u32::<BigEndian>(), Some(2147483648)));
    assert!(matches!(data.read_u32::<BigEndian>(), None));
}

#[test]
fn u32_le() {
    let mut data = hex!("04030201 00000080").as_ref();
    assert!(matches!(data.read_u32::<LittleEndian>(), Some(16909060)));
    assert!(matches!(data.read_u32::<LittleEndian>(), Some(2147483648)));
    assert!(matches!(data.read_u32::<LittleEndian>(), None));
}

#[test]
fn i32_be() {
    let mut data = hex!("01020304 80000000").as_ref();
    assert!(matches!(data.read_i32::<BigEndian>(), Some(16909060)));
    assert!(matches!(data.read_i32::<BigEndian>(), Some(-2147483648)));
    assert!(matches!(data.read_i32::<BigEndian>(), None));
}

#[test]
fn i32_le() {
    let mut data = hex!("04030201 00000080").as_ref();
    assert!(matches!(data.read_i32::<LittleEndian>(), Some(16909060)));
    assert!(matches!(data.read_i32::<LittleEndian>(), Some(-2147483648)));
    assert!(matches!(data.read_i32::<LittleEndian>(), None));
}

#[test]
fn u64_be() {
    let mut data = hex!("0102030405060708 8000000000000000").as_ref();
    assert!(matches!(data.read_u64::<BigEndian>(), Some(72623859790382856)));
    assert!(matches!(data.read_u64::<BigEndian>(), Some(9223372036854775808)));
    assert!(matches!(data.read_u64::<BigEndian>(), None));
}

#[test]
fn u64_le() {
    let mut data = hex!("0807060504030201 0000000000000080").as_ref();
    assert!(matches!(data.read_u64::<LittleEndian>(), Some(72623859790382856)));
    assert!(matches!(data.read_u64::<LittleEndian>(), Some(9223372036854775808)));
    assert!(matches!(data.read_u64::<LittleEndian>(), None));
}

#[test]
fn i64_be() {
    let mut data = hex!("0102030405060708 8000000000000000").as_ref();
    assert!(matches!(data.read_i64::<BigEndian>(), Some(72623859790382856)));
    assert!(matches!(data.read_i64::<BigEndian>(), Some(-9223372036854775808)));
    assert!(matches!(data.read_i64::<BigEndian>(), None));
}

#[test]
fn i64_le() {
    let mut data = hex!("0807060504030201 0000000000000080").as_ref();
    assert!(matches!(data.read_i64::<LittleEndian>(), Some(72623859790382856)));
    assert!(matches!(data.read_i64::<LittleEndian>(), Some(-9223372036854775808)));
    assert!(matches!(data.read_i64::<LittleEndian>(), None));
}

#[test]
fn u128_be() {
    let mut data = hex!("0102030405060708090A0B0C0D0E0F10 80000000000000000000000000000000").as_ref();
    assert!(matches!(data.read_u128::<BigEndian>(), Some(1339673755198158349044581307228491536)));
    assert!(matches!(data.read_u128::<BigEndian>(), Some(170141183460469231731687303715884105728)));
    assert!(matches!(data.read_u128::<BigEndian>(), None));
}

#[test]
fn u128_le() {
    let mut data = hex!("0102030405060708090A0B0C0D0E0F10 80000000000000000000000000000000").as_ref();
    assert!(matches!(data.read_u128::<BigEndian>(), Some(1339673755198158349044581307228491536)));
    assert!(matches!(data.read_u128::<BigEndian>(), Some(170141183460469231731687303715884105728)));
    assert!(matches!(data.read_u128::<BigEndian>(), None));
}

#[test]
fn i128_be() {
    let mut data = hex!("0102030405060708090A0B0C0D0E0F10 80000000000000000000000000000000").as_ref();
    assert!(matches!(data.read_i128::<BigEndian>(), Some(1339673755198158349044581307228491536)));
    assert!(matches!(data.read_i128::<BigEndian>(), Some(-170141183460469231731687303715884105728)));
    assert!(matches!(data.read_i128::<BigEndian>(), None));
}

#[test]
fn i128_le() {
    let mut data = hex!("100F0E0D0C0B0A090807060504030201 00000000000000000000000000000080").as_ref();
    assert!(matches!(data.read_i128::<LittleEndian>(), Some(1339673755198158349044581307228491536)));
    assert!(matches!(data.read_i128::<LittleEndian>(), Some(-170141183460469231731687303715884105728)));
    assert!(matches!(data.read_i128::<LittleEndian>(), None));
}

#[test]
fn uint_be() {
    let mut data = hex!("01 0102 010203 01020304 0102030405 010203040506 01020304050607 0102030405060708
                         010203040506070809 0102030405060708090A 0102030405060708090A0B 0102030405060708090A0B0C
                         0102030405060708090A0B0C0D 0102030405060708090A0B0C0D0E 0102030405060708090A0B0C0D0E0F 0102030405060708090A0B0C0D0E0F10
                         0102030405060708090A0B0C0D0E0F1011").as_ref();

    assert!(matches!(data.read_uint128::<BigEndian>(1), Some(1)));
    assert!(matches!(data.read_uint128::<BigEndian>(2), Some(258)));
    assert!(matches!(data.read_uint128::<BigEndian>(3), Some(66051)));
    assert!(matches!(data.read_uint128::<BigEndian>(4), Some(16909060)));
    assert!(matches!(data.read_uint128::<BigEndian>(5), Some(4328719365)));
    assert!(matches!(data.read_uint128::<BigEndian>(6), Some(1108152157446)));
    assert!(matches!(data.read_uint128::<BigEndian>(7), Some(283686952306183)));
    assert!(matches!(data.read_uint128::<BigEndian>(8), Some(72623859790382856)));
    assert!(matches!(data.read_uint128::<BigEndian>(9), Some(18591708106338011145)));
    assert!(matches!(data.read_uint128::<BigEndian>(10), Some(4759477275222530853130)));
    assert!(matches!(data.read_uint128::<BigEndian>(11), Some(1218426182456967898401291)));
    assert!(matches!(data.read_uint128::<BigEndian>(12), Some(311917102708983781990730508)));
    assert!(matches!(data.read_uint128::<BigEndian>(13), Some(79850778293499848189627010061)));
    assert!(matches!(data.read_uint128::<BigEndian>(14), Some(20441799243135961136544514575630)));
    assert!(matches!(data.read_uint128::<BigEndian>(15), Some(5233100606242806050955395731361295)));
    assert!(matches!(data.read_uint128::<BigEndian>(16), Some(1339673755198158349044581307228491536)));
}

#[test]
fn uint_le() {
    let mut data = hex!("01 0201 030201 04030201 0504030201 060504030201 07060504030201 0807060504030201
                         090807060504030201 0A090807060504030201 0B0A090807060504030201 0C0B0A090807060504030201
                         0D0C0B0A090807060504030201 0E0D0C0B0A090807060504030201 0F0E0D0C0B0A090807060504030201 100F0E0D0C0B0A090807060504030201
                         11100F0E0D0C0B0A090807060504030201").as_ref();

    assert!(matches!(data.read_uint128::<LittleEndian>(1), Some(1)));
    assert!(matches!(data.read_uint128::<LittleEndian>(2), Some(258)));
    assert!(matches!(data.read_uint128::<LittleEndian>(3), Some(66051)));
    assert!(matches!(data.read_uint128::<LittleEndian>(4), Some(16909060)));
    assert!(matches!(data.read_uint128::<LittleEndian>(5), Some(4328719365)));
    assert!(matches!(data.read_uint128::<LittleEndian>(6), Some(1108152157446)));
    assert!(matches!(data.read_uint128::<LittleEndian>(7), Some(283686952306183)));
    assert!(matches!(data.read_uint128::<LittleEndian>(8), Some(72623859790382856)));
    assert!(matches!(data.read_uint128::<LittleEndian>(9), Some(18591708106338011145)));
    assert!(matches!(data.read_uint128::<LittleEndian>(10), Some(4759477275222530853130)));
    assert!(matches!(data.read_uint128::<LittleEndian>(11), Some(1218426182456967898401291)));
    assert!(matches!(data.read_uint128::<LittleEndian>(12), Some(311917102708983781990730508)));
    assert!(matches!(data.read_uint128::<LittleEndian>(13), Some(79850778293499848189627010061)));
    assert!(matches!(data.read_uint128::<LittleEndian>(14), Some(20441799243135961136544514575630)));
    assert!(matches!(data.read_uint128::<LittleEndian>(15), Some(5233100606242806050955395731361295)));
    assert!(matches!(data.read_uint128::<LittleEndian>(16), Some(1339673755198158349044581307228491536)));
}

#[test]
fn int_be() {
    let mut data = hex!("81 8182 818283 81828384 8182838485 818283848586 81828384858687 8182838485868788
                         818283848586878889 8182838485868788898A 8182838485868788898A8B 8182838485868788898A8B8C
                         8182838485868788898A8B8C8D 8182838485868788898A8B8C8D8E 8182838485868788898A8B8C8D8E8F 8182838485868788898A8B8C8D8E8F18
                         8182838485868788898A8B8C8D8E8F1811").as_ref();

    assert!(matches!(data.read_int128::<BigEndian>(1), Some(-127)));
    assert!(matches!(data.read_int128::<BigEndian>(2), Some(-32382)));
    assert!(matches!(data.read_int128::<BigEndian>(3), Some(-8289661)));
    assert!(matches!(data.read_int128::<BigEndian>(4), Some(-2122153084)));
    assert!(matches!(data.read_int128::<BigEndian>(5), Some(-543271189371)));
    assert!(matches!(data.read_int128::<BigEndian>(6), Some(-139077424478842)));
    assert!(matches!(data.read_int128::<BigEndian>(7), Some(-35603820666583417)));
    assert!(matches!(data.read_int128::<BigEndian>(8), Some(-9114578090645354616)));
    assert!(matches!(data.read_int128::<BigEndian>(9), Some(-2333331991205210781559)));
    assert!(matches!(data.read_int128::<BigEndian>(10), Some(-597332989748533960078966)));
    assert!(matches!(data.read_int128::<BigEndian>(11), Some(-152917245375624693780215157)));
    assert!(matches!(data.read_int128::<BigEndian>(12), Some(-39146814816159921607735080052)));
    assert!(matches!(data.read_int128::<BigEndian>(13), Some(-10021584592936939931580180493171)));
    assert!(matches!(data.read_int128::<BigEndian>(14), Some(-2565525655791856622484526206251634)));
    assert!(matches!(data.read_int128::<BigEndian>(15), Some(-656774567882715295356038708800418161)));
    assert!(matches!(data.read_int128::<BigEndian>(16), Some(-168134289377975115611145909452907049192)));

    assert!(matches!(hex!("FF7F").as_ref().read_int128::<BigEndian>(2), Some(-129)));
}

#[test]
fn int_le() {
    let mut data = hex!("81 8281 838281 84838281 8584838281 868584838281 87868584838281 8887868584838281
                         898887868584838281 8A898887868584838281 8B8A898887868584838281 8C8B8A898887868584838281
                         8D8C8B8A898887868584838281 8E8D8C8B8A898887868584838281 8F8E8D8C8B8A898887868584838281 188F8E8D8C8B8A898887868584838281
                         11188F8E8D8C8B8A898887868584838281").as_ref();

    assert!(matches!(data.read_int128::<LittleEndian>(1), Some(-127)));
    assert!(matches!(data.read_int128::<LittleEndian>(2), Some(-32382)));
    assert!(matches!(data.read_int128::<LittleEndian>(3), Some(-8289661)));
    assert!(matches!(data.read_int128::<LittleEndian>(4), Some(-2122153084)));
    assert!(matches!(data.read_int128::<LittleEndian>(5), Some(-543271189371)));
    assert!(matches!(data.read_int128::<LittleEndian>(6), Some(-139077424478842)));
    assert!(matches!(data.read_int128::<LittleEndian>(7), Some(-35603820666583417)));
    assert!(matches!(data.read_int128::<LittleEndian>(8), Some(-9114578090645354616)));
    assert!(matches!(data.read_int128::<LittleEndian>(9), Some(-2333331991205210781559)));
    assert!(matches!(data.read_int128::<LittleEndian>(10), Some(-597332989748533960078966)));
    assert!(matches!(data.read_int128::<LittleEndian>(11), Some(-152917245375624693780215157)));
    assert!(matches!(data.read_int128::<LittleEndian>(12), Some(-39146814816159921607735080052)));
    assert!(matches!(data.read_int128::<LittleEndian>(13), Some(-10021584592936939931580180493171)));
    assert!(matches!(data.read_int128::<LittleEndian>(14), Some(-2565525655791856622484526206251634)));
    assert!(matches!(data.read_int128::<LittleEndian>(15), Some(-656774567882715295356038708800418161)));
    assert!(matches!(data.read_int128::<LittleEndian>(16), Some(-168134289377975115611145909452907049192)));
}
