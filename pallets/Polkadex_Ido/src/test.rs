// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use crate::mock::*;
use frame_support::assert_noop;
use frame_support::traits::{OnInitialize,OnFinalize};
use polkadex_primitives::assets::AssetId;
use sp_core::H160;
use sp_runtime::traits::Hash;
use polkadex_primitives::assets::AssetId::POLKADEX;

#[test]
fn test_register_investor() {
    // Register new account
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(ALICE.clone())),
            Ok(())
        );
        assert_noop!(
            PolkadexIdo::register_investor(Origin::signed(ALICE.clone())),
            Error::<Test>::InvestorAlreadyRegistered
        );
    });
}

#[test]
fn test_attest_investor() {
    let bob: u64 = 5;
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PolkadexIdo::attest_investor(Origin::signed(bob), ALICE.clone(), KYCStatus::Tier1),
            Error::<Test>::InvestorDoesNotExist
        );
        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(ALICE.clone())),
            Ok(())
        );
        assert_eq!(
            PolkadexIdo::attest_investor(Origin::signed(bob), ALICE.clone(), KYCStatus::Tier1),
            Ok(())
        );
    });
}

#[test]
fn test_register_round() {
    let balance: Balance = 100;
    let funding_period = 10;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );
    });
}

#[test]
fn test_whitelist_investor() {
    let balance: Balance = 100;
    let investor_address: u64 = 4;
    let funding_period = 10;
    let round_id = create_hash_data(&1u32);
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PolkadexIdo::whitelist_investor(
                Origin::signed(ALICE.clone()),
                round_id,
                investor_address,
                balance
            ),
            Error::<Test>::FundingRoundDoesNotExist
        );

        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );
        let round_id = <InfoProjectTeam<Test>>::get(ALICE.clone());
        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let open_block_number = funding_round.start_block;
        PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id);
        system::Pallet::<Test>::set_block_number(open_block_number);
        assert_noop!(
            PolkadexIdo::whitelist_investor(
                Origin::signed(investor_address),
                round_id,
                ALICE.clone(),
                balance
            ),
            Error::<Test>::NotACreater
        );

        assert_noop!(
            PolkadexIdo::whitelist_investor(
                Origin::signed(ALICE.clone()),
                round_id,
                investor_address,
                balance
            ),
            Error::<Test>::InvestorDoesNotExist
        );

        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(investor_address)),
            Ok(())
        );

        assert_eq!(
            PolkadexIdo::whitelist_investor(
                Origin::signed(ALICE.clone()),
                round_id,
                investor_address,
                balance
            ),
            Ok(())
        );
    });
}

#[test]
fn test_participate_in_round() {
    let balance: Balance = 100;
    let investor_address: u64 = 4;
    let funding_period = 10;
    let amount: Balance = 200;
    let min_allocation: Balance = 100;
    let max_allocation: Balance = 400;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {

        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                min_allocation,
                max_allocation,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE.clone());
        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let open_block_number = funding_round.start_block;
        let closing_block_number = funding_round.close_round_block;
        assert_eq!(PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id), Ok(()));


        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(investor_address)),
            Ok(())
        );

        system::Pallet::<Test>::set_block_number(open_block_number);
        assert_eq!(
            PolkadexIdo::show_interest_in_round(Origin::signed(investor_address), round_id, amount),
            Ok(())
        );


        <PolkadexIdo as OnFinalize<u64>>::on_finalize(closing_block_number);

        // Check if FundingRound was successfully updated after investment
        let round_info = <WhitelistInfoFundingRound<Test>>::get(round_id);
        println!("{}", round_info.actual_raise);
        assert_eq!(round_info.actual_raise == amount, true);
    });
}

#[test]
fn test_claim_tokens() {
    let balance: Balance = 100;
    let investor_address: u64 = 4;
    let funding_period = 10;
    let round_id = create_hash_data(&1u32);
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PolkadexIdo::claim_tokens(Origin::signed(investor_address), round_id,),
            Error::<Test>::InvestorDoesNotExist
        );

        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(investor_address)),
            Ok(())
        );

        assert_noop!(
            PolkadexIdo::claim_tokens(Origin::signed(investor_address), round_id,),
            Error::<Test>::FundingRoundDoesNotExist
        );


        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE.clone());
        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let closing_block_number = funding_round.close_round_block;
        PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id);
        system::Pallet::<Test>::set_block_number(closing_block_number);

        assert_eq!(
            PolkadexIdo::claim_tokens(Origin::signed(investor_address), round_id,),
            Ok(())
        );

        assert_eq!(
            LastClaimBlockInfo::<Test>::contains_key(round_id, investor_address),
            true
        );

        assert_eq!(
            InfoClaimAmount::<Test>::contains_key(round_id, investor_address),
            true
        );
    });
}

#[test]
fn test_show_interest_in_round() {
    let balance: Balance = 500;
    let investor_address: u64 = 4;
    let amount: Balance = 200;
    let min_allocation: Balance = 100;
    let max_allocation: Balance = 400;
    let round_id = create_hash_data(&1u32);
    let funding_period = 10;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PolkadexIdo::show_interest_in_round(Origin::signed(investor_address), round_id, amount),
            Error::<Test>::InvestorDoesNotExist
        );

        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(investor_address)),
            Ok(())
        );

        assert_noop!(
            PolkadexIdo::show_interest_in_round(Origin::signed(investor_address), round_id, amount),
            Error::<Test>::FundingRoundDoesNotExist
        );

        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                min_allocation,
                max_allocation,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE.clone());
        PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id);
        let funding_round = <WhitelistInfoFundingRound<Test>>::get(round_id);

        let open_block_number = funding_round.start_block;
        frame_system::Pallet::<Test>::set_block_number(open_block_number);

        //Check investing with lower than minimum allocation
        assert_noop!(
            PolkadexIdo::show_interest_in_round(
                Origin::signed(investor_address),
                round_id,
                min_allocation - 1
            ),
            Error::<Test>::NotAValidAmount
        );
        //Check investing with more than max allocation
        assert_noop!(
            PolkadexIdo::show_interest_in_round(
                Origin::signed(investor_address),
                round_id,
                max_allocation + 1
            ),
            Error::<Test>::NotAValidAmount
        );

        assert_eq!(
            PolkadexIdo::show_interest_in_round(Origin::signed(investor_address), round_id, amount),
            Ok(())
        );
    });
}
// Show Interest
// add some investors and have them to show interest to participate
// One investor of lowest amount will be randomly evicted
// verify the most invested was not get evicted
#[test]
fn test_show_interest_in_round_randomized_participants() {
    let balance: Balance = 500;
    let min_allocation: Balance = 100;
    let max_allocation: Balance = 400;
    let funding_period = 10;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                min_allocation,
                max_allocation,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE.clone());
        PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id);
        let investors: Vec<(u64, Balance)> =
            vec![(4u64, 200), (2u64, 200), (5u64, 200), (6u64, 300)];

        let funding_round: FundingRound<Test> = <WhitelistInfoFundingRound<Test>>::get(round_id);

        system::Pallet::<Test>::set_block_number(funding_round.start_block);

        for (investor_address, amount) in investors {
            assert_eq!(
                PolkadexIdo::register_investor(Origin::signed(investor_address)),
                Ok(())
            );
            assert_eq!(
                PolkadexIdo::show_interest_in_round(
                    Origin::signed(investor_address),
                    round_id,
                    amount
                ),
                Ok(())
            );
        }





        let total_investment_amount: Balance =
            InterestedParticipants::<Test>::iter_prefix_values(round_id)
                .fold(0_u128, |sum, amount| sum.saturating_add(amount));
        let investors_count = InterestedParticipants::<Test>::iter_prefix_values(round_id).count();
        // Check if an investor was randomly evicted
        assert_eq!(investors_count <= 3, true);
        assert_eq!(
            InterestedParticipants::<Test>::contains_key(round_id, 6u64),
            true
        );
        // Check if maximum effective investors are selected
        assert_eq!(total_investment_amount >= funding_round.amount, true);
    });
}

#[test]
fn test_withdraw_raise() {
    let balance: Balance = 100;
    let investor_address: u64 = 4;
    let funding_period = 10;
    let round_id = create_hash_data(&1u32);
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        system::Pallet::<Test>::set_block_number(0);
        assert_noop!(
            PolkadexIdo::withdraw_raise(Origin::signed(ALICE), round_id, investor_address),
            Error::<Test>::InvestorDoesNotExist
        );
        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(investor_address)),
            Ok(())
        );

        assert_noop!(
            PolkadexIdo::withdraw_raise(Origin::signed(ALICE), round_id, investor_address),
            Error::<Test>::FundingRoundDoesNotExist
        );

        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid.clone(),
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE);
        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let open_block_number = funding_round.start_block;
        let closing_block_number = funding_round.close_round_block;
        PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id);
        system::Pallet::<Test>::set_block_number(open_block_number);

        assert_noop!(
            PolkadexIdo::withdraw_raise(Origin::signed(3), round_id, investor_address),
            Error::<Test>::NotACreater
        );

        assert_eq!(PolkadexIdo::register_investor(Origin::signed(2)), Ok(()));
        <Test as Config>::Currency::deposit(
            AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
            &4_u64,
            100000,
        );
        let vote_period = match <VotingPeriod<Test>>::try_get() {
            Ok(voting_period ) => voting_period,
            Err(_) => <Test as Config>::DefaultVotingPeriod::get()
        };
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(4),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );


        assert_noop!(
            PolkadexIdo::withdraw_raise(Origin::signed(4), round_id, 2),
            Error::<Test>::NotACreater
        );

        //Test creator withdraw during when fundraising is not close should return error : WithdrawalBlocked
        system::Pallet::<Test>::set_block_number(closing_block_number - 1);
        assert_noop!(
            PolkadexIdo::withdraw_raise(Origin::signed(ALICE), round_id, investor_address),
            Error::<Test>::WithdrawalBlocked
        );

        //Test creator withdraw during when fundraising is closed should be successful
        system::Pallet::<Test>::set_block_number(closing_block_number);
        assert_eq!(
            PolkadexIdo::withdraw_raise(Origin::signed(ALICE), round_id, investor_address),
            Ok(())
        );
    });
}

#[test]
fn test_withdraw_token() {
    let balance: Balance = 100;
    let investor_address: u64 = 4;
    let funding_period = 10;
    let round_id = create_hash_data(&1u32);
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {

        assert_noop!(
            PolkadexIdo::withdraw_token(Origin::signed(ALICE), round_id, investor_address),
            Error::<Test>::InvestorDoesNotExist
        );
        assert_eq!(
            PolkadexIdo::register_investor(Origin::signed(investor_address)),
            Ok(())
        );

        assert_noop!(
            PolkadexIdo::withdraw_token(Origin::signed(ALICE), round_id, investor_address),
            Error::<Test>::FundingRoundDoesNotExist
        );



        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid.clone(),
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE);
        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let open_block_number = funding_round.start_block;
        let closing_block_number = funding_round.close_round_block;
        PolkadexIdo::approve_ido_round(Origin::signed(1_u64), round_id);
        system::Pallet::<Test>::set_block_number(open_block_number);

        assert_noop!(
            PolkadexIdo::withdraw_token(Origin::signed(3), round_id, investor_address),
            Error::<Test>::NotACreater
        );

        assert_eq!(PolkadexIdo::register_investor(Origin::signed(2)), Ok(()));
        <Test as Config>::Currency::deposit(
            AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
            &4_u64,
            100000,
        );
        let vote_period = match <VotingPeriod<Test>>::try_get() {
            Ok(voting_period ) => voting_period,
            Err(_) => <Test as Config>::DefaultVotingPeriod::get()
        };
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(4),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        assert_noop!(
            PolkadexIdo::withdraw_token(Origin::signed(4), round_id, 2),
            Error::<Test>::NotACreater
        );

        //Test creator withdraw tokens during when fundraising is not close should return error : WithdrawalBlocked
        system::Pallet::<Test>::set_block_number(closing_block_number - 1);
        assert_noop!(
            PolkadexIdo::withdraw_token(Origin::signed(ALICE), round_id, investor_address),
            Error::<Test>::WithdrawalBlocked
        );

        //Test creator withdraw tokens during when fundraising is closed should be successful
        system::Pallet::<Test>::set_block_number(closing_block_number);
        assert_eq!(
            PolkadexIdo::withdraw_token(Origin::signed(ALICE), round_id, investor_address),
            Ok(())
        );
    });
}

fn create_hash_data(data: &u32) -> <mock::Test as frame_system::Config>::Hash {
    data.using_encoded(<Test as frame_system::Config>::Hashing::hash)
}

#[test]
fn test_vote_for_round() {
    let balance: Balance = 100;
    let funding_period : BlockNumber = 10;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE);
        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let open_block_number = funding_round.start_block;
        let yes_voters : Vec<u64> = [6,7,8,9].to_vec();
        let no_voters : Vec<u64> = [4,2,5].to_vec();
        yes_voters.iter().for_each(|voter| {
            assert_eq!(
                PolkadexIdo::vote(Origin::signed(*voter),round_id.clone(), balance, 2, true),
                Ok(())
            );
        });
        no_voters.iter().for_each(|voter| {
            assert_eq!(
                PolkadexIdo::vote(Origin::signed(*voter),round_id.clone(), balance, 2, false),
                Ok(())
            );
        });
        <PolkadexIdo as OnInitialize<u64>>::on_initialize(open_block_number);
        assert_eq!(
            WhitelistInfoFundingRound::<Test>::contains_key(round_id),
            true
        );
        assert_eq!(
            InfoFundingRound::<Test>::contains_key(round_id),
            false
        );
    });
}

#[test]
fn test_vote_for_round_no_vote_majority() {
    let balance: Balance = 100;
    let funding_period : BlockNumber = 10;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE);

        let funding_round = <InfoFundingRound<Test>>::get(&round_id);
        let open_block_number = funding_round.start_block;

        let no_voters : Vec<u64> = [6,7,8,9].to_vec();
        let yes_voters : Vec<u64> = [4,2,5].to_vec();
        yes_voters.iter().for_each(|voter| {
            assert_eq!(
                PolkadexIdo::vote(Origin::signed(*voter),round_id.clone(), balance, 2, true),
                Ok(())
            );
        });
        no_voters.iter().for_each(|voter| {
            assert_eq!(
                PolkadexIdo::vote(Origin::signed(*voter),round_id.clone(), balance, 2, false),
                Ok(())
            );
        });
        <PolkadexIdo as OnInitialize<u64>>::on_initialize(open_block_number);
        assert_eq!(
            WhitelistInfoFundingRound::<Test>::contains_key(round_id),
            false
        );
        assert_eq!(
            InfoFundingRound::<Test>::contains_key(round_id),
            false
        );
    });
}


/// Test whether the voter will receive amount when the vote stake period ends
/// voter adds vote with amount
/// chain processed to on_initialize for the unlocking_block of the voters amount staked
/// check whether the total_balance - free_balance is zero
#[test]
fn test_get_reserve_amount() {
    let balance: Balance = 100;
    let funding_period : BlockNumber = 10;
    let cid = [0_u8;32].to_vec();
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(
            PolkadexIdo::register_round(
                Origin::signed(ALICE),
                cid,
                AssetId::CHAINSAFE(H160::from_low_u64_be(24)),
                balance,
                AssetId::POLKADEX,
                balance,
                funding_period,
                balance,
                balance,
                10.saturated_into(),
            ),
            Ok(())
        );

        let round_id = <InfoProjectTeam<Test>>::get(ALICE);
        PolkadexIdo::vote(Origin::signed(4),round_id, balance, 2, false);
        let unlocking_block = PolkadexIdo::vote_multiplier_to_block_number(2);
        let reserve_balance = <Test as Config>::Currency::total_balance(AssetId::POLKADEX,&4_u64 ) - <Test as Config>::Currency::free_balance(AssetId::POLKADEX,&4_u64 );

        assert_eq!(reserve_balance, balance);
        <PolkadexIdo as OnInitialize<u64>>::on_initialize(unlocking_block);
        let reserve_balance = <Test as Config>::Currency::total_balance(AssetId::POLKADEX,&4_u64 ) - <Test as Config>::Currency::free_balance(AssetId::POLKADEX,&4_u64 );
        assert_eq!(reserve_balance, 0);
    });
}