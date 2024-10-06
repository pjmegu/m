#[derive(thiserror::Error, Debug)]
pub enum SerializeError {
    #[error("other libraries error")]
    Other,
    #[error(transparent)]
    RmpSerError(#[from] rmp_serde::encode::Error),
    #[error(transparent)]
    RmpDesError(#[from] rmp_serde::decode::Error),
    #[error(transparent)]
    BitcodeSerError(#[from] bitcode::Error),
}

pub trait SerializeTag {
    fn get_abi_id() -> u8;
}
pub struct RmpTag;
impl SerializeTag for RmpTag {
    fn get_abi_id() -> u8 {
        RMP_ABI_ID
    }
}
pub struct BitcodeTag;
impl SerializeTag for BitcodeTag {
    fn get_abi_id() -> u8 {
        BITCODE_ABI_ID
    }
}

pub const ERROR_ABI_ID: u8 = 0xFF;
pub const RMP_ABI_ID: u8 = 0x00;
pub const BITCODE_ABI_ID: u8 = 0x01;

pub trait ToByte<T: SerializeTag> {
    fn to_byte(&self) -> Result<Vec<u8>, SerializeError>;
}

impl<T: bitcode::Encode> ToByte<BitcodeTag> for T {
    fn to_byte(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(bitcode::encode(self))
    }
}

impl<T: serde::Serialize> ToByte<RmpTag> for T {
    fn to_byte(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(rmp_serde::to_vec_named(self)?)
    }
}

pub trait FromByte<S: SerializeTag> {
    fn from_byte(bytes: &[u8]) -> Result<Self, SerializeError>
    where
        Self: Sized;
}

impl<T: bitcode::DecodeOwned> FromByte<BitcodeTag> for T {
    fn from_byte(bytes: &[u8]) -> Result<Self, SerializeError> {
        Ok(bitcode::decode(bytes)?)
    }
}

impl<T: serde::de::DeserializeOwned> FromByte<RmpTag> for T {
    fn from_byte(bytes: &[u8]) -> Result<Self, SerializeError> {
        Ok(rmp_serde::from_slice(bytes)?)
    }
}

macro_rules! foreach_func_sig {
    ($mac: ident) => {
        $mac!(P1);
        $mac!(P1, P2);
        $mac!(P1, P2, P3);
        $mac!(P1, P2, P3, P4);
        $mac!(P1, P2, P3, P4, P5);
        $mac!(P1, P2, P3, P4, P5, P6);
        $mac!(P1, P2, P3, P4, P5, P6, P7);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15);
        $mac!(P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16);
    };
}

pub trait ParamListFrom<S: SerializeTag>: FromByte<S> {}

macro_rules! gen_input_param_rmp {
    ($($type: ident),+) => {
        impl<$($type: serde::de::DeserializeOwned),+> ParamListFrom<RmpTag> for ($($type),+,) {}
    };
}

foreach_func_sig!(gen_input_param_rmp);

macro_rules! gen_input_param_bitcode {
    ($($type: ident),+) => {
        impl<$($type: bitcode::DecodeOwned),+> ParamListFrom<BitcodeTag> for ($($type),+,) {}
    };
}

foreach_func_sig!(gen_input_param_bitcode);

pub trait ParamListTo<S: SerializeTag>: ToByte<S> {}

macro_rules! gen_output_param_rmp {
    ($($type: ident),+) => {
        impl<$($type: serde::Serialize),+> ParamListTo<RmpTag> for ($($type),+,) {}
    };
}

foreach_func_sig!(gen_output_param_rmp);

macro_rules! gen_output_param_bitcode {
    ($($type: ident),+) => {
        impl<$($type: bitcode::Encode),+> ParamListTo<BitcodeTag> for ($($type),+,) {}
    };
}

foreach_func_sig!(gen_output_param_bitcode);
