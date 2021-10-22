use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::constants::*;
use serde_json::json;

fn push_dep_account_cell(template: &mut TemplateGenerator) {
    template.push_dep(
        json!({
            "capacity": util::gen_account_cell_capacity(8),
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "yyyyy.bit",
                "next": "zzzzz.bit",
                "expired_at": 0,
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": 0,
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
}

fn push_input_reverse_record_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    manager: &str,
    account: &str,
) {
    template.push_input(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager,
            },
            "type": {
                "code_hash": "{{reverse-record-cell-type}}"
            },
            "data": {
                "account": account
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_reverse_record_cell(
    template: &mut TemplateGenerator,
    capacity: u64,
    owner: &str,
    manager: &str,
    account: &str,
) {
    template.push_output(
        json!({
            "capacity": capacity.to_string(),
            "lock": {
                "owner_lock_args": owner,
                "manager_lock_args": manager,
            },
            "type": {
                "code_hash": "{{reverse-record-cell-type}}"
            },
            "data": {
                "account": account
            }
        }),
        None,
    );
}

fn before_each() -> (TemplateGenerator, &'static str, &'static str) {
    let mut template = init("redeclare_reverse_record");
    let owner = "0x050000000000000000000000000000000000001111";
    let manager = "0x050000000000000000000000000000000000001111";

    // cell_deps
    push_dep_account_cell(&mut template);

    // inputs
    push_input_reverse_record_cell(&mut template, 20_100_000_000, owner, manager, "xxxxx.bit");

    (template, owner, manager)
}

test_with_generator!(test_reverse_record_redeclare, || {
    let (mut template, owner, manager) = before_each();

    push_output_reverse_record_cell(&mut template, 20_099_990_000, owner, manager, "yyyyy.bit");

    template.as_json()
});
