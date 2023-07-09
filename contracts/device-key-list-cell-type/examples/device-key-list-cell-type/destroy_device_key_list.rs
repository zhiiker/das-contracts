use das_core::{assert, code_to_error};
use das_types::packed::DeviceKeyListCellData;
use device_key_list_cell_type::error::ErrorCode;
use molecule::prelude::Entity;

use crate::traits::{Action, GetCellWitness, Rule};

pub fn action() -> Action {
    let mut destroy_action = Action::new("destroy_device_key_list");
    destroy_action.add_verification(Rule::new("Verify cell structure", |contract| {
        assert!(
            contract.get_input_inner_cells().len() == 1
                && contract.get_output_inner_cells().len() == 0
                && contract.get_input_inner_cells()[0].0 == 0,
            ErrorCode::InvalidTransactionStructure,
            "Should have 1 cell in input[0] and 0 cell in output"
        );
        Ok(())
    }));

    destroy_action.add_verification(Rule::new("Verify refund lock", |contract| {
        let input_cell_meta = contract.get_input_inner_cells()[0].get_meta();
        let key_list_in_input = contract
            .get_parser()
            .get_cell_witness::<DeviceKeyListCellData>(input_cell_meta)?;
        let refund_lock = key_list_in_input.refund_lock();
        assert!(
            contract
                .get_output_outer_cells()
                .iter()
                .all(|c| c.lock().as_slice() == refund_lock.as_slice()),
            ErrorCode::InconsistentBalanceCellLocks,
            "Should return capacity to refund_lock"
        );
        Ok(())
    }));

    destroy_action
}
