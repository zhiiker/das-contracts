pub use ckb_std::ckb_types::core::ScriptHashType;
use das_types::constants::{config_cell_type, das_lock, DataType};
use das_types::packed as types_packed;
use molecule::prelude::{Builder, Entity};

use crate::traits::Blake2BHash;
use crate::witness_parser::general_witness_parser::{get_witness_parser, Condition};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ScriptType {
    Lock,
    Type,
}

#[derive(Debug)]
pub enum LockScript {
    AlwaysSuccessLock,
    DasLock,
    Secp256k1Blake160SignhashLock,
    Secp256k1Blake160MultisigLock,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OracleCellType {
    Quote = 0,
    Time = 1,
    Height = 2,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum CellField {
    Capacity,
    Lock,
    Type,
    Data,
}

pub const CKB_HASH_DIGEST: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const ONE_CKB: u64 = 100_000_000;
pub const CELL_BASIC_CAPACITY: u64 = 6_1 * ONE_CKB;
pub const ONE_USD: u64 = 1_000_000;
pub const DPOINT_MAX_LIMIT: u64 = 10_000_000 * ONE_USD;

pub const RATE_BASE: u64 = 10_000;

pub const ACCOUNT_SUFFIX: &str = ".bit";
pub const ACCOUNT_MAX_PRICED_LENGTH: u8 = 8;

pub const CUSTOM_KEYS_NAMESPACE: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz_";
pub const COIN_TYPE_DIGITS: &[u8] = b"0123456789";

pub const SECP_SIGNATURE_SIZE: usize = 65;
// This is smaller than the real data type in solidity, but it is enough for now.
pub const EIP712_CHAINID_SIZE: usize = 8;

pub const DAY_SEC: u64 = 86400;
pub const DAYS_OF_YEAR: u64 = 365;
pub const YEAR_SEC: u64 = DAY_SEC * DAYS_OF_YEAR;

pub const PRE_ACCOUNT_CELL_TIMEOUT: u64 = DAY_SEC;
pub const PRE_ACCOUNT_CELL_SHORT_TIMEOUT: u64 = 3600;

pub const CROSS_CHAIN_BLACK_ARGS: [u8; 20] = [0; 20];

pub const TYPE_ID_CODE_HASH: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 84, 89, 80, 69, 95, 73, 68,
];

// pub fn get_config_cell_main() -> types_packed::ConfigCellMain {
//     let config = get_witness_parser()
//         .find_unique::<types_packed::ConfigCellMain>()
//         .unwrap();
//     let trusted_lock = das_lock();
//     let trusted_type = config_cell_type()
//         .clone()
//         .as_builder()
//         .args(types_packed::Bytes::from_slice((DataType::ConfigCellMain as u32).to_le_bytes().as_ref()).unwrap())
//         .build();
//     config
//         .verify_unique(
//             ckb_std::ckb_constants::Source::CellDep,
//             &[
//                 Condition::LockHash(&trusted_lock.blake2b_256()),
//                 Condition::TypeIs(&trusted_type.into()),
//                 Condition::DataIs(&config.hash.unwrap()),
//             ],
//         )
//         .unwrap();
//
//     config.result
// }
