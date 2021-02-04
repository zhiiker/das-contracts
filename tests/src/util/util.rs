use super::super::Loader;
use super::constants::SECP_SIGNATURE_SIZE;
use ckb_testtool::context::Context;
use ckb_tool::{
    ckb_chain_spec::consensus::TYPE_ID_CODE_HASH,
    ckb_hash::{blake2b_256, new_blake2b},
    ckb_jsonrpc_types as rpc_types,
    ckb_types::core::Capacity,
    ckb_types::{
        bytes,
        core::{ScriptHashType, TransactionView},
        h256,
        packed::*,
        prelude::*,
        H160, H256,
    },
};
use das_types::packed as das_packed;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::error::Error;
use std::str::FromStr;

lazy_static! {
    pub static ref SECP256K1: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
}

pub fn hex_to_bytes(input: &str) -> Result<bytes::Bytes, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Ok(bytes::Bytes::default())
    } else {
        Ok(bytes::Bytes::from(hex::decode(hex)?))
    }
}

pub fn hex_to_byte32(input: &str) -> Result<Byte32, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    let data = hex::decode(hex)?
        .into_iter()
        .map(Byte::new)
        .collect::<Vec<_>>();
    let mut inner = [Byte::new(0); 32];
    inner.copy_from_slice(&data);

    Ok(Byte32::new_builder().set(inner).build())
}

pub fn hex_to_hash(input: &str) -> Result<das_packed::Hash, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    let data = hex::decode(hex)?
        .into_iter()
        .map(Byte::new)
        .collect::<Vec<_>>();
    let mut inner = [Byte::new(0); 32];
    inner.copy_from_slice(&data);

    Ok(das_packed::Hash::new_builder().set(inner).build())
}

pub fn hex_to_u64(input: &str) -> Result<u64, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Ok(0u64)
    } else {
        Ok(u64::from_str_radix(hex, 16)?)
    }
}

pub fn account_bytes_to_id(input: das_packed::Bytes) -> das_packed::Bytes {
    let account = input.as_reader().raw_data();
    let hash = blake2b_256(account);
    das_packed::Bytes::from(hash.get(..20).unwrap())
}

pub fn account_to_id(input: Vec<u8>) -> Vec<u8> {
    let hash = blake2b_256(input);
    hash.get(..20).unwrap().to_vec()
}

pub fn deploy_dev_contract(
    context: &mut Context,
    binary_name: &str,
) -> (Byte32, OutPoint, CellDep) {
    let contract_bin: bytes::Bytes = Loader::default().load_binary(binary_name);

    deploy_contract(context, binary_name, contract_bin)
}

pub fn deploy_builtin_contract(
    context: &mut Context,
    binary_name: &str,
) -> (Byte32, OutPoint, CellDep) {
    let contract_bin: bytes::Bytes = Loader::with_deployed_scripts().load_binary(binary_name);

    deploy_contract(context, binary_name, contract_bin)
}

fn deploy_contract(
    context: &mut Context,
    binary_name: &str,
    contract_bin: bytes::Bytes,
) -> (Byte32, OutPoint, CellDep) {
    let args = binary_name
        .as_bytes()
        .to_vec()
        .into_iter()
        .map(Byte::new)
        .collect::<Vec<_>>();
    let type_ = Script::new_builder()
        .code_hash(Byte32::new_unchecked(bytes::Bytes::from(
            TYPE_ID_CODE_HASH.as_bytes(),
        )))
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::new_builder().set(args).build())
        .build();
    let type_id = type_.calc_script_hash();
    let cell = CellOutput::new_builder()
        .capacity(Capacity::bytes(contract_bin.len()).unwrap().pack())
        .type_(ScriptOpt::new_builder().set(Some(type_)).build())
        .build();
    let out_point = context.create_cell(cell, contract_bin);
    let cell_dep = CellDep::new_builder().out_point(out_point.clone()).build();

    (type_id, out_point, cell_dep)
}

pub fn mock_script(context: &mut Context, out_point: OutPoint, args: bytes::Bytes) -> Script {
    context
        .build_script(&out_point, args)
        .expect("Build script failed, can not find cell of script.")
}

pub fn mock_cell(
    context: &mut Context,
    capacity: u64,
    lock_script: Script,
    type_script: Option<Script>,
    bytes: Option<bytes::Bytes>,
) -> OutPoint {
    let data;
    if bytes.is_some() {
        data = bytes.unwrap();
    } else {
        data = bytes::Bytes::new();
    }

    context.create_cell(
        CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script)
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build(),
        data,
    )
}

pub fn mock_cell_with_outpoint(
    context: &mut Context,
    out_point: OutPoint,
    capacity: u64,
    lock_script: Script,
    type_script: Option<Script>,
    bytes: Option<bytes::Bytes>,
) -> OutPoint {
    let data;
    if bytes.is_some() {
        data = bytes.unwrap();
    } else {
        data = bytes::Bytes::new();
    }

    context.create_cell_with_out_point(
        out_point.clone(),
        CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script)
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build(),
        data,
    );

    out_point
}

pub fn mock_input(out_point: OutPoint, since: Option<u64>) -> CellInput {
    let mut builder = CellInput::new_builder().previous_output(out_point);

    if let Some(data) = since {
        builder = builder.since(data.pack());
    }

    builder.build()
}

pub fn mock_output(capacity: u64, lock_script: Script, type_script: Option<Script>) -> CellOutput {
    CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script)
        .type_(ScriptOpt::new_builder().set(type_script).build())
        .build()
}

pub fn serialize_signature(signature: &secp256k1::recovery::RecoverableSignature) -> [u8; 65] {
    let (recov_id, data) = signature.serialize_compact();
    let mut signature_bytes = [0u8; 65];
    signature_bytes[0..64].copy_from_slice(&data[0..64]);
    signature_bytes[64] = recov_id.to_i32() as u8;
    signature_bytes
}

pub type SignerFn = Box<
    dyn FnMut(&HashSet<H160>, &H256, &rpc_types::Transaction) -> Result<Option<[u8; 65]>, String>,
>;

pub fn get_privkey_signer(input: &str) -> SignerFn {
    let privkey = secp256k1::SecretKey::from_str(input.trim_start_matches("0x")).unwrap();
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
    let lock_arg = H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20])
        .expect("Generate hash(H160) from pubkey failed");
    Box::new(
        move |lock_args: &HashSet<H160>, message: &H256, _tx: &rpc_types::Transaction| {
            if lock_args.contains(&lock_arg) {
                if message == &h256!("0x0") {
                    Ok(Some([0u8; 65]))
                } else {
                    let message = secp256k1::Message::from_slice(message.as_bytes())
                        .expect("Convert to secp256k1 message failed");
                    let signature = SECP256K1.sign_recoverable(&message, &privkey);
                    Ok(Some(serialize_signature(&signature)))
                }
            } else {
                Ok(None)
            }
        },
    )
}

pub fn build_signature<
    S: FnMut(&H256, &rpc_types::Transaction) -> Result<[u8; SECP_SIGNATURE_SIZE], String>,
>(
    tx: &TransactionView,
    input_size: usize,
    input_group_idxs: &[usize],
    witnesses: &[Bytes],
    mut signer: S,
) -> Result<bytes::Bytes, String> {
    let init_witness_idx = input_group_idxs[0];
    let init_witness = if witnesses[init_witness_idx].raw_data().is_empty() {
        WitnessArgs::default()
    } else {
        WitnessArgs::from_slice(witnesses[init_witness_idx].raw_data().as_ref())
            .map_err(|err| err.to_string())?
    };

    let init_witness = init_witness
        .as_builder()
        .lock(Some(bytes::Bytes::from(vec![0u8; SECP_SIGNATURE_SIZE])).pack())
        .build();

    let mut blake2b = new_blake2b();
    blake2b.update(tx.hash().as_slice());
    blake2b.update(&(init_witness.as_bytes().len() as u64).to_le_bytes());
    blake2b.update(&init_witness.as_bytes());
    for idx in input_group_idxs.iter().skip(1).cloned() {
        let other_witness: &Bytes = &witnesses[idx];
        blake2b.update(&(other_witness.len() as u64).to_le_bytes());
        blake2b.update(&other_witness.raw_data());
    }
    for outter_witness in &witnesses[input_size..witnesses.len()] {
        blake2b.update(&(outter_witness.len() as u64).to_le_bytes());
        blake2b.update(&outter_witness.raw_data());
    }
    let mut message = [0u8; 32];
    blake2b.finalize(&mut message);
    let message = H256::from(message);
    signer(&message, &tx.data().into()).map(|data| bytes::Bytes::from(data.to_vec()))
}