use super::common::*;
use crate::util::{
    accounts::*, constants::*, error::Error, template_common_cell::*, template_generator::TemplateGenerator,
    template_parser::*,
};
use das_types_std::constants::AccountStatus;
use serde_json::json;

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );

    (template, timestamp)
}

#[test]
fn test_account_transfer() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    test_tx(template.as_json())
}

#[test]
fn challenge_account_transfer_account_multiple_cells() {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    // Simulate transferring multiple AccountCells at one time.
    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
        }),
    );
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_transfer_account_with_other_cells() {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    template.push_contract_cell("balance-cell-type", false);

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            }
        }),
    );
    // Simulate transferring some balance of the user at the same time.
    push_input_balance_cell(&mut template, 100_000_000_000, SENDER);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_transfer_account_not_modified() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate owner not change after the transaction
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellOwnerLockShouldBeModified)
}

#[test]
fn challenge_account_transfer_too_often() {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                // Simulate transferring multiple times in a day.
                "last_transfer_account_at": timestamp - 86400 + 1,
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellThrottle)
}

#[test]
fn challenge_account_transfer_not_clear_records() {
    let (mut template, timestamp) = init("transfer_account", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": SENDER,
                "manager_lock_args": SENDER
            },
            "witness": {
                "records": [
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Personal",
                        "value": "0x0000000000000000000000000000000000000000",
                    },
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
                // Simulate not clearing all records when transferring.
                "records": [
                    {
                        "type": "address",
                        "key": "eth",
                        "label": "Company",
                        "value": "0x0000000000000000000000000000000000001111",
                    }
                ]
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellRecordNotEmpty)
}

#[test]
fn challenge_account_transfer_modify_data_account() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "data": {
                // Simulate the account field has been modified accidentally.
                "account": "zzzzz.bit",
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_transfer_modify_data_next() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "data": {
                // Simulate the next field has been modified accidentally.
                "next": "ooooo.bit",
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_transfer_modify_data_expired_at() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "data": {
                // Simulate the expired_at field has been modified accidentally.
                "expired_at": timestamp + YEAR_SEC * 2,
            },
            "witness": {
                "last_transfer_account_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellDataNotConsistent)
}

#[test]
fn challenge_account_transfer_modify_witness_account() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                // Simulate the account field has been modified accidentally.
                "account": "zzzzz.bit",
                "last_transfer_account_at": timestamp
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_transfer_modify_witness_registered_at() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                // Simulate the registered_at field has been modified accidentally.
                "registered_at": 1234,
                "last_transfer_account_at": timestamp
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_transfer_modify_witness_last_edit_manager_at() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
                // Simulate the last_edit_manager_at field has been modified accidentally.
                "last_edit_manager_at": 1234
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_transfer_modify_witness_last_edit_records_at() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
                // Simulate the last_edit_records_at field has been modified accidentally.
                "last_edit_records_at": 1234
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}

#[test]
fn challenge_account_transfer_modify_witness_status() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": RECEIVER,
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_transfer_account_at": timestamp,
                // Simulate the status field has been modified accidentally.
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellProtectFieldIsModified)
}
