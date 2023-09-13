use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use ckb_std::ckb_constants::Source;
use ckb_std::syscalls::load_witness;
use das_types::constants::{DataType, WITNESS_HEADER_BYTES, WITNESS_TYPE_BYTES};
use molecule::prelude::Entity;

use crate::error::{ErrorCode, ScriptError};
use crate::traits::Blake2BHash;

#[derive(Default)]
struct GeneralWitnessParser {
    witnesses: Vec<Witness>,
    hashes: BTreeMap<[u8; 32], usize>,
}

struct PartialWitness {
    buf: Vec<u8>,
    actual_size: usize,
}
struct CompleteWitness {
    buf: Vec<u8>,
    parsed: bool,
}

struct WithMeta<T> {
    item: T,
    index: usize,
    source: Source,
}

enum Witness {
    Loading(WithMeta<PartialWitness>),
    Loaded(WithMeta<CompleteWitness>),
}

trait FromWitness {
    type Error;
    fn from_witness(witness: &Witness) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn parsable(witness: &Witness) -> bool;
}

struct ParsedWithHash<T> {
    result: T,
    hash: Option<[u8; 32]>,
}

impl<T> FromWitness for T
where
    T: Entity + 'static,
{
    type Error = Box<dyn ScriptError>;
    fn from_witness(witness: &Witness) -> Result<Self, Box<dyn ScriptError>> {
        if let Witness::Loaded(WithMeta { item, .. }) = witness {
            let type_constant = T::get_type_constant();
            das_assert!(
                Self::parsable(witness),
                ErrorCode::WitnessDataDecodingError,
                "The data type constant: {:?} and the actual molecule structure: {} does not match",
                type_constant,
                T::NAME
            );
            Ok(
                T::from_compatible_slice(&item.buf[WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES..])
                    .map_err(|_| code_to_error!(ErrorCode::WitnessDataDecodingError))?,
            )
        } else {
            panic!("Witness is still parsing")
        }
    }

    fn parsable(witness: &Witness) -> bool {
        let type_constant = T::get_type_constant() as u32;
        match witness {
            Witness::Loaded(WithMeta { item, .. }) => {
                type_constant
                    == u32::from_be_bytes(
                        item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
                            .try_into()
                            .unwrap(),
                    )
            }
            Witness::Loading(WithMeta { item, .. }) => {
                type_constant
                    == u32::from_be_bytes(
                        item.buf[WITNESS_HEADER_BYTES..WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES]
                            .try_into()
                            .unwrap(),
                    )
            }
        }
    }
}

impl Witness {
    fn parse<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &'static mut self,
    ) -> Result<ParsedWithHash<T>, Box<dyn ScriptError>> {
        let res = match self {
            Witness::Loaded(_) => T::from_witness(self).map_err(|e| e.into())?,
            Witness::Loading(parsing_witness) => {
                let mut buf_vec = vec![0u8; parsing_witness.item.actual_size];
                let loaded_len = parsing_witness.item.buf.len();
                buf_vec[..loaded_len].copy_from_slice(&parsing_witness.item.buf.as_slice());
                load_witness(
                    &mut buf_vec[loaded_len..],
                    loaded_len,
                    parsing_witness.index,
                    Source::Input,
                )?;
                *self = Self::Loaded(WithMeta {
                    item: CompleteWitness {
                        buf: buf_vec,
                        parsed: true,
                    },
                    index: parsing_witness.index,
                    source: parsing_witness.source,
                });
                T::from_witness(self).map_err(|e| e.into())?
            }
        };

        use core::any::Any;
        let hash = (&res as &dyn Any)
            .downcast_ref::<&dyn Blake2BHash>()
            .map(|res| res.blake2b_256());
        Ok(ParsedWithHash { result: res, hash })
    }
}

impl GeneralWitnessParser {
    #[allow(dead_code)]
    fn init(&mut self) -> Result<(), Box<dyn ScriptError>> {
        let mut i = 0;
        let mut buf = [0u8; WITNESS_HEADER_BYTES + WITNESS_TYPE_BYTES];
        let mut witnesses: Vec<Witness> = Vec::new();
        loop {
            let res = match load_witness(&mut buf, 0, i, Source::Input) {
                Err(ckb_std::syscalls::SysError::IndexOutOfBound) => break,
                Ok(actual_size) => Witness::Loaded(WithMeta {
                    item: CompleteWitness {
                        buf: buf[..actual_size].to_owned(),
                        parsed: false,
                    },
                    index: i,
                    source: Source::Input,
                }),
                Err(ckb_std::syscalls::SysError::LengthNotEnough(actual_size)) => Witness::Loading(WithMeta {
                    item: PartialWitness {
                        buf: buf.to_vec(),
                        actual_size,
                    },
                    index: i,
                    source: Source::Input,
                }),
                Err(e) => return Err(e.into()),
            };
            witnesses.push(res);
            i += 1;
        }
        self.witnesses = witnesses;
        Ok(())
    }

    #[allow(dead_code)]
    fn parse_witness<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &'static mut self,
        index: usize,
    ) -> Result<T, Box<dyn ScriptError>> {
        let res = self.witnesses[index].parse::<T>()?;
        if let Some(hash) = res.hash {
            let _ = self
                .hashes
                .insert(hash, index)
                .is_some_and(|original| panic!("Witness {} and {} have same hash!", index, original));
        }
        Ok(res.result)
    }

    #[allow(dead_code)]
    fn find_by_hash<T: FromWitness<Error = impl Into<Box<dyn ScriptError>>> + 'static>(
        &'static mut self,
        hash: &[u8; 32],
    ) -> Result<Option<T>, Box<dyn ScriptError>> {
        if let Some(index) = self.hashes.get(hash) {
            return self.parse_witness(*index).map(Option::Some);
        }
        for witness in self.witnesses.iter_mut() {
            let parsed = match witness {
                Witness::Loaded(w) => w.item.parsed,
                _ => false,
            };
            if parsed {
                continue;
            }
            if !T::parsable(witness) {
                continue;
            }
            let res = witness.parse::<T>()?;
            match res.hash {
                Some(h) if &h == hash => {
                    return Ok(Some(res.result));
                },
                _ => continue,
            }
        }

        Ok(None)
    }
}

pub trait GetDataType {
    fn get_type_constant() -> DataType;
}

impl<T> GetDataType for T
where
    T: Entity,
{
    fn get_type_constant() -> DataType {
        match T::NAME {
            "DeviceKeyListCellData" => DataType::DeviceKeyListEntityData,
            _ => unimplemented!(),
        }
    }
}
