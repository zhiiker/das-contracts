use super::common::init;
use crate::util::{accounts::*, error::Error, template_common_cell::*, template_generator::*, template_parser::*};
use serde_json::json;

fn before_each() -> (TemplateGenerator, u64) {
    let (mut template, timestamp) = init("edit_manager", Some("0x00"));

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": SENDER
            }
        }),
    );

    (template, timestamp)
}

#[test]
fn test_account_edit_manager() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_edit_manager_at": timestamp,
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn test_account_edit_manager_and_upgrade_lock_type() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": "0x050000000000000000000000000000000000002222"
            },
            "witness": {
                "last_edit_manager_at": timestamp,
            }
        }),
    );

    test_tx(template.as_json());
}

#[test]
fn challenge_account_edit_manager_multiple_cells() {
    let (mut template, timestamp) = init("edit_manager", Some("0x00"));

    // Simulate editing multiple AccountCells at one time.
    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": SENDER
            }
        }),
    );
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": SENDER
            }
        }),
    );

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_edit_manager_at": timestamp,
            }
        }),
    );
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_edit_manager_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_edit_manager_with_other_cells() {
    let (mut template, timestamp) = init("edit_manager", Some("0x00"));

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
                "owner_lock_args": SENDER, // The owner lock should not be modified here.
                "manager_lock_args": RECEIVER
            },
            "witness": {
                "last_edit_manager_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::InvalidTransactionStructure)
}

#[test]
fn challenge_account_edit_manager_not_modified() {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                // Simulate not modifying the manager.
                "manager_lock_args": SENDER
            },
            "witness": {
                "last_edit_manager_at": timestamp,
            }
        }),
    );

    challenge_tx(template.as_json(), Error::AccountCellManagerLockShouldBeModified)
}
