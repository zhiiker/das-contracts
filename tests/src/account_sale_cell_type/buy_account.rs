use super::common::*;
use crate::util::{self, constants::*, error::Error, template_generator::*, template_parser::TemplateParser};
use ckb_testtool::context::Context;
use das_types::{constants::*, packed::*, prelude::*};
use serde_json::json;

fn push_input_account_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_input(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Selling as u8)
            }
        }),
        Some(2),
    );
    template.push_empty_witness();
}

fn push_input_account_sale_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_input(
        json!({
            "capacity": "20_100_000_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111"
            },
            "type": {
                "code_hash": "{{account-sale-cell-type}}"
            },
            "witness": {
                "account": "xxxxx.bit",
                "price": "20_000_000_000",
                "description": "This is some account description.",
                "started_at": timestamp
            }
        }),
        None,
    );
    template.push_empty_witness();
}

fn push_input_fee_cell(template: &mut TemplateGenerator) {
    template.push_input(
        json!({
            "capacity": "20_000_000_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "manager_lock_args": "0x050000000000000000000000000000000000002222",
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
    template.push_das_lock_witness("0000000000000000000000000000000000000000000000000000000000000000");
}

fn push_output_account_cell(template: &mut TemplateGenerator, timestamp: u64) {
    template.push_output(
        json!({
            "capacity": util::gen_account_cell_capacity(5),
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000002222",
                "manager_lock_args": "0x050000000000000000000000000000000000002222"
            },
            "type": {
                "code_hash": "{{account-cell-type}}"
            },
            "data": {
                "account": "xxxxx.bit",
                "next": "yyyyy.bit",
                "expired_at": (timestamp + YEAR_SEC),
            },
            "witness": {
                "account": "xxxxx.bit",
                "registered_at": (timestamp - MONTH_SEC),
                "last_transfer_account_at": 0,
                "last_edit_manager_at": 0,
                "last_edit_records_at": 0,
                "status": (AccountStatus::Normal as u8)
            }
        }),
        Some(2),
    );
}

fn push_output_income_cell(template: &mut TemplateGenerator) {
    template.push_output(
        json!({
            "lock": {
                "code_hash": "{{always_success}}"
            },
            "type": {
                "code_hash": "{{income-cell-type}}"
            },
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000008888", None)
                        },
                        "capacity": "200_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000009999", None)
                        },
                        "capacity": "200_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "200_000_000"
                    }
                ]
            }
        }),
        None,
    );
}

fn push_output_receive_cell(template: &mut TemplateGenerator) {
    template.push_output(
        json!({
            "capacity": "40_099_990_000",
            "lock": {
                "owner_lock_args": "0x050000000000000000000000000000000000001111",
                "manager_lock_args": "0x050000000000000000000000000000000000001111",
            },
            "type": {
                "code_hash": "{{balance-cell-type}}"
            }
        }),
        None,
    );
}

fn gen_inviter_and_channel_locks(inviter_args: &str, channel_args: &str) -> (Script, Script) {
    let inviter_lock = gen_fake_das_lock(&gen_das_lock_args(inviter_args, None));
    let channel_lock = gen_fake_das_lock(&gen_das_lock_args(channel_args, None));
    (inviter_lock, channel_lock)
}

fn gen_params(inviter_args: &str, channel_args: &str) -> String {
    let (inviter_lock, channel_lock) = gen_inviter_and_channel_locks(inviter_args, channel_args);

    format!(
        "0x{}{}",
        util::bytes_to_hex(inviter_lock.as_slice()),
        util::bytes_to_hex(channel_lock.as_slice())
    )
}

fn before_each() -> (TemplateGenerator, u64) {
    let params = gen_params(
        "0x050000000000000000000000000000000000008888",
        "0x050000000000000000000000000000000000009999",
    );
    let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));

    // inputs
    push_input_account_cell(&mut template, timestamp);
    push_input_account_sale_cell(&mut template, timestamp);
    push_input_fee_cell(&mut template);

    (template, timestamp)
}

test_with_generator!(test_account_sale_buy, || {
    let (mut template, timestamp) = before_each();

    // outputs
    push_output_account_cell(&mut template, timestamp);
    push_output_income_cell(&mut template);
    push_output_receive_cell(&mut template);

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_buy_account_expired,
    Error::AccountCellInExpirationGracePeriod,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));

        // inputs
        template.push_input(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - YEAR_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );
        template.push_empty_witness();

        push_input_account_sale_cell(&mut template, timestamp);
        push_input_fee_cell(&mut template);

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000002222",
                    "manager_lock_args": "0x050000000000000000000000000000000000002222"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - YEAR_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_receive_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_account_capacity,
    Error::AccountCellChangeCapacityError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                // Simulate the AccountCell.capacity has been modified accidentally.
                "capacity": util::gen_account_cell_capacity(5) - 1,
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000002222",
                    "manager_lock_args": "0x050000000000000000000000000000000000002222"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_receive_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_input_account_status,
    Error::AccountCellStatusLocked,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));

        // inputs
        template.push_input(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell.status is wrong in inputs.
                    "status": (AccountStatus::Normal as u8)
                }
            }),
            Some(2),
        );
        template.push_empty_witness();

        push_input_account_sale_cell(&mut template, timestamp);
        push_input_fee_cell(&mut template);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_income_cell(&mut template);
        push_output_receive_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_output_account_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        template.push_output(
            json!({
                "capacity": util::gen_account_cell_capacity(5),
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000002222",
                    "manager_lock_args": "0x050000000000000000000000000000000000002222"
                },
                "type": {
                    "code_hash": "{{account-cell-type}}"
                },
                "data": {
                    "account": "xxxxx.bit",
                    "next": "yyyyy.bit",
                    "expired_at": (timestamp + YEAR_SEC),
                },
                "witness": {
                    "account": "xxxxx.bit",
                    "registered_at": (timestamp - MONTH_SEC),
                    "last_transfer_account_at": 0,
                    "last_edit_manager_at": 0,
                    "last_edit_records_at": 0,
                    // Simulate the AccountCell.status is wrong in outputs.
                    "status": (AccountStatus::Selling as u8)
                }
            }),
            Some(2),
        );

        push_output_income_cell(&mut template);
        push_output_receive_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_sale_account,
    Error::AccountSaleCellAccountIdInvalid,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));

        // inputs
        push_input_account_cell(&mut template, timestamp);

        template.push_input(
            json!({
                "capacity": "20_100_000_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111"
                },
                "type": {
                    "code_hash": "{{account-sale-cell-type}}"
                },
                "witness": {
                    // Simulate the AccountSaleCell.account is wrong in inputs.
                    "account": "zzzzz.bit",
                    "price": "20_000_000_000",
                    "description": "This is some account description.",
                    "started_at": timestamp
                }
            }),
            None,
        );
        template.push_empty_witness();

        push_input_fee_cell(&mut template);

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_income_cell(&mut template);
        push_output_receive_cell(&mut template);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_seller_profit_owner,
    Error::AccountSaleCellProfitError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_income_cell(&mut template);

        template.push_output(
            json!({
                "capacity": "40_099_990_000",
                "lock": {
                    // Simulate the lock of the cell which carrying profit and refund to seller is wrong in outputs.
                    "owner_lock_args": "0x050000000000000000000000000000000000003333",
                    "manager_lock_args": "0x050000000000000000000000000000000000003333",
                },
                "type": {
                    "code_hash": "{{balance-cell-type}}"
                }
            }),
            None,
        );

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_seller_profit_capacity,
    Error::AccountSaleCellProfitError,
    || {
        let (mut template, timestamp) = before_each();

        // outputs
        push_output_account_cell(&mut template, timestamp);
        push_output_income_cell(&mut template);

        template.push_output(
            json!({
                // Simulate the capacity of the cell which carrying profit and refund to seller is wrong in outputs.
                "capacity": "40_099_980_000",
                "lock": {
                    "owner_lock_args": "0x050000000000000000000000000000000000001111",
                    "manager_lock_args": "0x050000000000000000000000000000000000001111",
                },
                "type": {
                    "code_hash": "{{balance-cell-type}}"
                }
            }),
            None,
        );

        template.as_json()
    }
);
