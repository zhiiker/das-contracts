use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::packed,
    debug,
    high_level::{
        load_cell_capacity, load_cell_lock_hash, load_cell_occupied_capacity, load_script,
    },
};
use das_core::{
    assert,
    constants::{
        wallet_maker_lock, ScriptType, ACCOUNT_ID_LENGTH, ALWAYS_SUCCESS_LOCK, CELL_BASIC_CAPACITY,
    },
    data_parser,
    error::Error,
    util,
};
use das_types::{constants::ConfigID, prelude::Entity};

pub fn main() -> Result<(), Error> {
    debug!("====== Running wallet-cell-type ======");

    debug!("Find out WalletCells ...");

    let this_type_script = load_script().map_err(|e| Error::from(e))?;
    let input_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Input)?;
    let output_cells =
        util::find_cells_by_script(ScriptType::Type, &this_type_script, Source::Output)?;

    match util::load_das_action() {
        Ok(action_data) => {
            let action = action_data.as_reader().action().raw_data();
            if action == b"create_wallet" {
                debug!("Route to create_wallet action ...");

                let always_success_script = util::script_literal_to_script(ALWAYS_SUCCESS_LOCK);
                let always_success_script_hash =
                    util::blake2b_256(always_success_script.as_slice());

                debug!("Check if wallet maker lock has been used in inputs ...");

                let expected_lock = wallet_maker_lock();
                let has_expected_lock =
                    util::find_cells_by_script(ScriptType::Lock, &expected_lock, Source::Input)?
                        .len()
                        > 0;
                assert!(
                    has_expected_lock,
                    Error::WalletRequireWalletMakerLock,
                    "This transaction require wallet-maker-lock in inputs."
                );

                assert!(
                    input_cells.len() == 0 && output_cells.len() > 0,
                    Error::WalletFoundInvalidTransaction,
                    "There should be none WalletCell in inputs and 1 or more WalletCells in outputs."
                );

                debug!("Check if all WalletCell structure ...");

                for i in output_cells {
                    let lock_script_hash =
                        load_cell_lock_hash(i, Source::Output).map_err(|e| Error::from(e))?;

                    assert!(
                        lock_script_hash == always_success_script_hash,
                        Error::WalletRequireAlwaysSuccess,
                        "WalletCell can be only created with always-success lock script: {}",
                        always_success_script.code_hash()
                    );

                    let data = util::load_cell_data(i, Source::Output)?;
                    assert!(
                        data.len() == ACCOUNT_ID_LENGTH,
                        Error::WalletRequireAccountId,
                        "The data of WalletCell should contains {} bytes long account ID.",
                        ACCOUNT_ID_LENGTH
                    );
                }
            } else if action == b"recycle_wallet" {
                debug!("Route to recycle_wallet action ...");

                let mut parser = util::load_das_witnesses(None)?;
                parser.parse_all_data()?;
                parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
                let config_main_reader = parser.configs().main()?;

                debug!("Check if wallet maker lock has been used in inputs ...");

                let expected_lock = wallet_maker_lock();
                let has_expected_lock =
                    util::find_cells_by_script(ScriptType::Lock, &expected_lock, Source::Input)?
                        .len()
                        > 0;
                assert!(
                    has_expected_lock,
                    Error::WalletRequireWalletMakerLock,
                    "This transaction require wallet-maker-lock in inputs."
                );

                assert!(
                    input_cells.len() > 0 && output_cells.len() == 0,
                    Error::WalletFoundInvalidTransaction,
                    "There should be 1 or more WalletCells in inputs and none WalletCell in outputs."
                );

                // Create an account_id->refund map for later verification.
                let mut refunds_list: Vec<(Vec<u8>, u64)> = Vec::new();
                for input_index in input_cells {
                    let wallet_cell_data = util::load_cell_data(input_index, Source::Input)?;
                    let account_id = data_parser::wallet_cell::get_id(&wallet_cell_data).to_vec();

                    debug!(
                        "Calculate total refund of the WalletCell[0x{}] ...",
                        util::hex_string(account_id.as_ref())
                    );

                    // A user may have more than one WalletCells.
                    let ret = refunds_list.iter().position(|item| item.0 == account_id);
                    let total_capacity = load_cell_capacity(input_index, Source::Input)
                        .map_err(|e| Error::from(e))?;
                    let occupied_capacity = load_cell_occupied_capacity(input_index, Source::Input)
                        .map_err(|e| Error::from(e))?;
                    if let Some(i) = ret {
                        refunds_list[i].1 += total_capacity - occupied_capacity;
                    } else {
                        refunds_list.push((account_id, total_capacity - occupied_capacity));
                    }
                }

                // TODO If balance is too low, there is no need to include AccountCell in transaction.
                // TODO WalletCell should be able to recycle after AccountCell is recycled.

                // Create an account_id->account_cell_index map for later verification.
                let mut account_indexs_grouped_by_id: Vec<(Vec<u8>, usize)> = Vec::new();
                let account_cells = util::find_cells_by_type_id(
                    ScriptType::Type,
                    config_main_reader.type_id_table().account_cell(),
                    Source::Input,
                )?;
                for index in account_cells {
                    let data = util::load_cell_data(index, Source::Input)?;
                    let id = data_parser::account_cell::get_id(&data).to_vec();
                    account_indexs_grouped_by_id.push((id, index));
                }

                for (account_id, expected_refund) in refunds_list.into_iter() {
                    if expected_refund >= CELL_BASIC_CAPACITY {
                        debug!("Check if the major capacity of the WalletCell[0x{}] has been refund to owner lock.", util::hex_string(account_id.as_ref()));

                        // Find out the AccountCell which has the same account ID with the WalletCell.
                        let ret = account_indexs_grouped_by_id
                            .iter()
                            .find(|item| item.0 == account_id);
                        assert!(
                            ret.is_some(),
                            Error::WalletFoundInvalidTransaction,
                            "There should be 1 AccountCell in the inputs which has the same account ID as the WalletCell."
                        );

                        let (_, account_cell_index) = ret.unwrap();
                        let (_, _, _entity) =
                            parser.verify_and_get(account_cell_index.to_owned(), Source::Input)?;

                        // TODO Retrive owner lock from AccountCell's lock script.
                        let expected_lock = packed::Script::default();
                        let refund_cells = util::find_cells_by_script(
                            ScriptType::Lock,
                            &expected_lock,
                            Source::Output,
                        )?;

                        assert!(
                            refund_cells.len() == 1,
                            Error::WalletFoundInvalidTransaction,
                            "All refunds of the same lock script should be stored in the same cell."
                        );

                        let refund = load_cell_capacity(refund_cells[0], Source::Output)
                            .map_err(|e| Error::from(e))?;

                        assert!(
                            expected_refund == refund,
                            Error::WalletRefundError,
                            "The refund should be calculated correctly. ( expected: {}, current: {} )",
                            expected_refund,
                            refund
                        );
                    }
                }
            } else if action == b"withdraw_from_wallet" {
                debug!("Route to withdraw_from_wallet action ...");

                let mut parser = util::load_das_witnesses(None)?;
                parser.parse_only_config(&[ConfigID::ConfigCellMain])?;
                parser.parse_all_data()?;
                let config = parser.configs().main()?;

                assert!(
                    input_cells.len() == 1 && output_cells.len() == 1,
                    Error::WalletFoundInvalidTransaction,
                    "There should be only one WalletCell in inputs and outputs."
                );

                debug!("For WalletCell, check if only capacity is reduced ...");

                let input_cell_index = input_cells[0];
                let output_cell_index = output_cells[0];
                util::is_cell_consistent(
                    (input_cell_index, Source::Input),
                    (output_cell_index, Source::Output),
                )?;
                util::is_cell_capacity_gt(
                    (input_cell_index, Source::Input),
                    (output_cell_index, Source::Output),
                )?;

                let wallet_cell_data = util::load_cell_data(input_cell_index, Source::Input)?;
                let account_id = data_parser::wallet_cell::get_id(&wallet_cell_data).to_vec();
                let _id_in_wallet = account_id.as_slice();

                debug!("Check if AccountCell exists ...");

                // Find out AccountCells in current transaction.
                let input_account_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.type_id_table().account_cell(),
                    Source::Input,
                )?;
                let output_account_index = util::find_only_cell_by_type_id(
                    ScriptType::Type,
                    config.type_id_table().account_cell(),
                    Source::Output,
                )?;
                util::is_cell_consistent(
                    (input_account_index, Source::Input),
                    (output_account_index, Source::Output),
                )?;
                util::is_cell_capacity_equal(
                    (input_account_index, Source::Input),
                    (output_account_index, Source::Output),
                )?;

                // TODO check if AccountCell is unlocked by owner's signature.
            } else {
                debug!("Route to other action ...");

                verify_if_only_capacity_increased(input_cells, output_cells)?;
            }
        }
        // WalletCell can be also used in any transactions without the ActionData of DAS.
        Err(Error::WitnessActionNotFound) => {
            debug!("Route to non-action ...");

            verify_if_only_capacity_increased(input_cells, output_cells)?;
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

fn verify_if_only_capacity_increased(
    input_cells: Vec<usize>,
    output_cells: Vec<usize>,
) -> Result<(), Error> {
    debug!("Check if WalletCell is consistent and has more capacity than before.");

    assert!(
        input_cells.len() == output_cells.len(),
        Error::CellsMustHaveSameOrderAndNumber,
        "The WalletCells in inputs should has the same number and order with those in outputs."
    );

    for (i, input_cell_index) in input_cells.into_iter().enumerate() {
        let output_cell_index = output_cells[i];
        util::is_cell_capacity_gt(
            (output_cell_index, Source::Output),
            (input_cell_index, Source::Input),
        )?;
        util::is_cell_consistent(
            (input_cell_index, Source::Input),
            (output_cell_index, Source::Output),
        )?;
    }

    Ok(())
}
