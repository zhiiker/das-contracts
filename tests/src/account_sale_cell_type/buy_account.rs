use super::common::*;
use crate::util::{
    self, constants::*, error::Error, template_common_cell::*, template_generator::*, template_parser::TemplateParser,
};
use ckb_testtool::context::Context;
use das_types::{constants::*, packed::*, prelude::*};
use serde_json::json;

fn push_simple_output_income_cell(template: &mut TemplateGenerator) {
    push_output_income_cell(
        template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": COMMON_INCOME_CREATOR_LOCK_ARGS
                        },
                        "capacity": "20_000_000_000"
                    },
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
        "0x{}{}00",
        util::bytes_to_hex(inviter_lock.as_slice()),
        util::bytes_to_hex(channel_lock.as_slice())
    )
}

fn before_each(price: u64, paied: u64) -> (TemplateGenerator, u64, &'static str, &'static str, &'static str) {
    let params = gen_params(
        "0x050000000000000000000000000000000000008888",
        "0x050000000000000000000000000000000000009999",
    );
    let (mut template, timestamp) = init_with_profit_rate("buy_account", Some(&params));
    let seller = "0x050000000000000000000000000000000000001111";
    let buyer = "0x050000000000000000000000000000000000002222";
    let account = "xxxxx.bit";

    // inputs
    push_input_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": seller,
                "manager_lock_args": seller
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
                "status": (AccountStatus::Selling as u8)
            }
        }),
    );
    push_input_account_sale_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": seller,
                "manager_lock_args": seller
            },
            "witness": {
                "account": account,
                "price": price.to_string(),
            }
        }),
    );
    push_input_balance_cell(&mut template, paied, buyer);

    (template, timestamp, seller, buyer, account)
}

test_with_generator!(test_account_sale_buy_create_income_cell, || {
    let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": buyer,
                "manager_lock_args": buyer
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    // 194 CKB(price) + 20_099_990_000(refund of AccountSaleCell)
    push_output_balance_cell(&mut template, 39_499_990_000, seller);
    push_output_balance_cell(&mut template, 20_000_000_000, buyer);

    template.as_json()
});

test_with_generator!(test_account_sale_buy_not_create_income_cell, || {
    let (mut template, _timestamp, seller, buyer, account) = before_each(1_000_000_000_000, 2_000_000_000_000);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": buyer,
                "manager_lock_args": buyer
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );

    push_output_income_cell(
        &mut template,
        json!({
            "witness": {
                "records": [
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000008888", None)
                        },
                        "capacity": "10_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-das-lock}}",
                            "args": gen_das_lock_args("0x050000000000000000000000000000000000009999", None)
                        },
                        "capacity": "10_000_000_000"
                    },
                    {
                        "belong_to": {
                            "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                            "args": DAS_WALLET_LOCK_ARGS
                        },
                        "capacity": "10_000_000_000"
                    }
                ]
            }
        }),
    );

    push_output_balance_cell(&mut template, 990099990000, seller);
    push_output_balance_cell(&mut template, 1_000_000_000_000, buyer);

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
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "data": {
                    "account": account,
                    // Simulate the situation AccountCell has expired.
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - YEAR_SEC),
                    "status": (AccountStatus::Selling as u8)
                }
            }),
        );
        push_input_account_sale_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "witness": {
                    "account": account,
                    "price": 20_000_000_000u64.to_string(),
                }
            }),
        );
        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                    "expired_at": (timestamp - 1),
                },
                "witness": {
                    "account": account,
                    "registered_at": (timestamp - YEAR_SEC),
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );

        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_account_capacity,
    Error::AccountCellChangeCapacityError,
    || {
        let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                // Simulate the AccountCell.capacity has been modified accidentally.
                "capacity": util::gen_account_cell_capacity(5) - 1,
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );

        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

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
        let (mut template, _timestamp) = init_with_profit_rate("buy_account", Some(&params));
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    // Simulate the AccountCell.status is wrong in inputs.
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );
        push_input_account_sale_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "witness": {
                    "account": account,
                    "price": 20_000_000_000u64.to_string(),
                }
            }),
        );
        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_output_account_status,
    Error::AccountCellStatusLocked,
    || {
        let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    // Simulate the AccountCell.status is wrong in outputs.
                    "status": (AccountStatus::Selling as u8)
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

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
        let (mut template, _timestamp) = init_with_profit_rate("buy_account", Some(&params));
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Selling as u8)
                }
            }),
        );
        push_input_account_sale_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "witness": {
                    // Simulate the AccountSaleCell.account is wrong in inputs.
                    "account": "zzzzz.bit",
                    "price": 20_000_000_000u64.to_string(),
                }
            }),
        );
        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_wrong_owner,
    Error::AccountSaleCellNewOwnerError,
    || {
        let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    // Simulate transferring AccountCell to unknown owner.
                    "owner_lock_args": "0x050000000000000000000000000000000000003333",
                    "manager_lock_args": "0x050000000000000000000000000000000000003333"
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(challenge_account_sale_buy_change_owner, Error::ChangeError, || {
    let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": buyer,
                "manager_lock_args": buyer
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 39_499_990_000, seller);
    // Simulate transfer changes to another lock.
    push_output_balance_cell(
        &mut template,
        20_000_000_000,
        "0x050000000000000000000000000000000000003333",
    );

    template.as_json()
});

challenge_with_generator!(challenge_account_sale_buy_change_capacity, Error::ChangeError, || {
    let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

    // outputs
    push_output_account_cell(
        &mut template,
        json!({
            "lock": {
                "owner_lock_args": buyer,
                "manager_lock_args": buyer
            },
            "data": {
                "account": account,
            },
            "witness": {
                "account": account,
                "status": (AccountStatus::Normal as u8)
            }
        }),
    );
    push_simple_output_income_cell(&mut template);
    push_output_balance_cell(&mut template, 39_499_990_000, seller);
    // Simulate transfer changes less than the user should get.
    push_output_balance_cell(&mut template, 20_000_000_000 - 1, buyer);

    template.as_json()
});

challenge_with_generator!(
    challenge_account_sale_buy_seller_profit_owner,
    Error::ChangeError,
    || {
        let (mut template, _timestamp, _, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        // Simulate transfer profit to another lock.
        push_output_balance_cell(
            &mut template,
            39_499_990_000,
            "0x050000000000000000000000000000000000003333",
        );
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_seller_profit_capacity,
    Error::ChangeError,
    || {
        let (mut template, _timestamp, seller, buyer, account) = before_each(20_000_000_000, 40_000_000_000);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8)
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        // Simulate transfer profit less than the seller should get.
        push_output_balance_cell(&mut template, 39_499_980_000, seller);
        push_output_balance_cell(&mut template, 20_000_000_000, buyer);

        template.as_json()
    }
);

challenge_with_generator!(
    challenge_account_sale_buy_not_clear_records,
    Error::AccountCellRecordNotEmpty,
    || {
        let params = gen_params(
            "0x050000000000000000000000000000000000008888",
            "0x050000000000000000000000000000000000009999",
        );
        let (mut template, _timestamp) = init_with_profit_rate("buy_account", Some(&params));
        let seller = "0x050000000000000000000000000000000000001111";
        let buyer = "0x050000000000000000000000000000000000002222";
        let account = "xxxxx.bit";

        // inputs
        push_input_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Selling as u8),
                    "records": json!([
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
                    ])
                }
            }),
        );
        push_input_account_sale_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": seller,
                    "manager_lock_args": seller
                },
                "witness": {
                    "account": account,
                    "price": 20_000_000_000u64.to_string(),
                }
            }),
        );
        push_input_balance_cell(&mut template, 20_000_000_000, buyer);

        // outputs
        push_output_account_cell(
            &mut template,
            json!({
                "lock": {
                    "owner_lock_args": buyer,
                    "manager_lock_args": buyer
                },
                "data": {
                    "account": account,
                },
                "witness": {
                    "account": account,
                    "status": (AccountStatus::Normal as u8),
                    // Simulate not clearing all records when transferring.
                    "records": json!([
                        {
                            "type": "address",
                            "key": "eth",
                            "label": "Personal",
                            "value": "0x0000000000000000000000000000000000000000",
                        }
                    ])
                }
            }),
        );
        push_simple_output_income_cell(&mut template);
        push_output_balance_cell(&mut template, 39_499_990_000, seller);

        template.as_json()
    }
);
