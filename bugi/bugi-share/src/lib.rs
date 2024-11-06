#[derive(thiserror::Error, Debug)]
pub enum SerializeError {
    #[error("other libraries error")]
    Other(Box<dyn std::any::Any + Send + Sync>),

    #[cfg(feature = "ser-rmp")]
    #[error(transparent)]
    RmpSerError(#[from] rmp_serde::encode::Error),

    #[cfg(feature = "ser-rmp")]
    #[error(transparent)]
    RmpDesError(#[from] rmp_serde::decode::Error),

    #[cfg(feature = "ser-bitcode")]
    #[error(transparent)]
    BitcodeSerError(#[from] bitcode::Error),
}

pub trait SerializeTag {
    fn get_abi_id() -> u64;
}

#[cfg(feature = "ser-rmp")]
pub struct RmpTag;

#[cfg(feature = "ser-rmp")]
impl SerializeTag for RmpTag {
    fn get_abi_id() -> u64 {
        RMP_ABI_ID
    }
}

#[cfg(feature = "ser-bitcode")]
pub struct BitcodeTag;

#[cfg(feature = "ser-bitcode")]
impl SerializeTag for BitcodeTag {
    fn get_abi_id() -> u64 {
        BITCODE_ABI_ID
    }
}

pub const ERROR_ABI_ID: u64 = 0xFF;

#[cfg(feature = "ser-rmp")]
pub const RMP_ABI_ID: u64 = 0x00;

#[cfg(feature = "ser-bitcode")]
pub const BITCODE_ABI_ID: u64 = 0x01;

pub trait ToByte<T: SerializeTag> {
    fn to_byte(&self) -> Result<Vec<u8>, SerializeError>;
}

#[cfg(feature = "ser-bitcode")]
impl<T: bitcode::Encode> ToByte<BitcodeTag> for T {
    fn to_byte(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(bitcode::encode(self))
    }
}

#[cfg(feature = "ser-rmp")]
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

#[cfg(feature = "ser-bitcode")]
impl<T: bitcode::DecodeOwned> FromByte<BitcodeTag> for T {
    fn from_byte(bytes: &[u8]) -> Result<Self, SerializeError> {
        Ok(bitcode::decode(bytes)?)
    }
}

#[cfg(feature = "ser-rmp")]
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

#[cfg(feature = "ser-rmp")]
macro_rules! gen_input_param_rmp {
    ($($type: ident),+) => {
        impl<$($type: serde::de::DeserializeOwned),+> ParamListFrom<RmpTag> for ($($type),+,) {}
    };
}

#[cfg(feature = "ser-rmp")]
foreach_func_sig!(gen_input_param_rmp);

#[cfg(feature = "ser-bitcode")]
macro_rules! gen_input_param_bitcode {
    ($($type: ident),+) => {
        impl<$($type: bitcode::DecodeOwned),+> ParamListFrom<BitcodeTag> for ($($type),+,) {}
    };
}

#[cfg(feature = "ser-bitcode")]
foreach_func_sig!(gen_input_param_bitcode);

pub trait ParamListTo<S: SerializeTag>: ToByte<S> {}

#[cfg(feature = "ser-rmp")]
macro_rules! gen_output_param_rmp {
    ($($type: ident),+) => {
        impl<$($type: serde::Serialize),+> ParamListTo<RmpTag> for ($($type),+,) {}
    };
}

#[cfg(feature = "ser-rmp")]
foreach_func_sig!(gen_output_param_rmp);

#[cfg(feature = "ser-bitcode")]
macro_rules! gen_output_param_bitcode {
    ($($type: ident),+) => {
        impl<$($type: bitcode::Encode),+> ParamListTo<BitcodeTag> for ($($type),+,) {}
    };
}

#[cfg(feature = "ser-bitcode")]
foreach_func_sig!(gen_output_param_bitcode);
