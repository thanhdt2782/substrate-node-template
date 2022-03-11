#![cfg(test)]

use crate::{mock::*, pallet::Error, *};
use frame_support::{assert_noop, assert_ok};

// In mock.rs, we've created 2 kitties in genesis:
// a Female and Male owned by account 1 and 2, respectively.

// This function checks that kitty ownership is set correctly in storage.
// This will panic if things are not correct.
fn assert_ownership(owner: u64, kitty_id: [u8; 16]) {
	// For a kitty to be owned it should exist.
	let kitty = Kitties::<Test>::get(kitty_id).unwrap();
	// The kitty's owner is set correctly.
	assert_eq!(kitty.owner, owner);

	for (check_owner, owned) in KittiesOwned::<Test>::iter() {
		if owner == check_owner {
			// Owner should have this kitty.
			assert!(owned.contains(&kitty_id));
		} else {
			// Everyone else should not.
			assert!(!owned.contains(&kitty_id));
		}
	}
}

#[test]
fn should_build_genesis_kitties() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check we have 2 kitties, as specified in genesis
		assert_eq!(CountForKitties::<Test>::get(), 2);

		// Check owners own the correct amount of kitties
		let kitties_owned_by_1 = KittiesOwned::<Test>::get(1);
		assert_eq!(kitties_owned_by_1.len(), 1);

		let kitties_owned_by_2 = KittiesOwned::<Test>::get(2);
		assert_eq!(kitties_owned_by_2.len(), 1);

		// Check that kitties are owned by the correct owners
		let kitty_1 = kitties_owned_by_1[0];
		assert_ownership(1, kitty_1);

		let kitty_2 = kitties_owned_by_2[0];
		assert_ownership(2, kitty_2);
	});
}

#[test]
fn create_kitty_should_work() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Create a kitty with account #10
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));

		// Check that 3 kitties exists (together with the 2 from genesis)
		assert_eq!(CountForKitties::<Test>::get(), 3);

		// Check that account #10 owns 1 kitty
		let kitties_owned = KittiesOwned::<Test>::get(10);
		assert_eq!(kitties_owned.len(), 1);
		let id = kitties_owned.last().unwrap();
		assert_ownership(10, *id);

		// Check that this kitty is specifically owned by account #10
		let kitty = Kitties::<Test>::get(id).unwrap();
		assert_eq!(kitty.owner, 10);
		assert_eq!(kitty.price, None);
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check that account 10 own a kitty
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
		let id = KittiesOwned::<Test>::get(10)[0];

		// Account 10 send kitty to account 3
		assert_ok!(SubstrateKitties::transfer(Origin::signed(10), 3, id));

		// Account 10 now has nothing
		assert_eq!(KittiesOwned::<Test>::get(10).len(), 0);

		// but account 3 does
		assert_eq!(KittiesOwned::<Test>::get(3).len(), 1);
		assert_ownership(3, id);
	});
}

#[test]
fn transfer_non_owned_kitty_should_fail() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		let hash = KittiesOwned::<Test>::get(1)[0];

		// Account 9 cannot transfer a kitty with this hash.
		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(9), 2, hash),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn mint_should_fail() {
	// Check mint fails when kitty id already exists.
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		let id = [0u8; 16];

		// Mint new kitty with `id`
		assert_ok!(SubstrateKitties::mint(&1, id, Gender::Male));

		// Mint another kitty with the same `id` should fail
		assert_noop!(SubstrateKitties::mint(&1, id, Gender::Male), Error::<Test>::DuplicateKitty);
	});
}


#[test]
fn multiple_kitties_in_one_block() {
	// Check that multiple create_kitty calls work in a single block.
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	]).execute_with(|| {

		// Create a kitty with account #10
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));

		// Increment extrinsic index
		frame_system::Pallet::<Test>::set_extrinsic_index(1);

		// Create a kitty with account #10
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
	});
}

#[test]
fn breed_kitty_works() {
	// Check that breed kitty works as expected.
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Get mom and dad kitties from account #1
		let mom = [0u8; 16];
		assert_ok!(SubstrateKitties::mint(&1, mom, Gender::Female));

		// Mint male kitty for account #1
		let dad = [1u8; 16];
		assert_ok!(SubstrateKitties::mint(&1, dad, Gender::Male));

		// Breeder can only breed kitties they own
		assert_ok!(SubstrateKitties::breed_kitty(Origin::signed(1), mom, dad));

		// Check that newly bred kitty exists
		assert_ok!(KittiesOwned::<Test>::get(1)[3]);

		// Check the new DNA is from the mom and dad
		let new_dna = KittiesOwned::<Test>::get(1)[3];
		for &i in new_dna.iter() {
			assert!(i == 0u8 || i == 1u8)
		}

		// Kitty cant breed with itself
		assert_noop!(
			SubstrateKitties::breed_kitty(Origin::signed(1), mom, mom),
			Error::<Test>::CantBreed
		);
	});
}

#[test]
fn cant_exceed_max_kitties() {
	// Check that create_kitty fails when user owns too many kitties.
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Create `MaxKittiesOwned` kitties with account #10
		for _i in 0..<Test as Config>::MaxKittiesOwned::get() {
			assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
			// We do this because the hash of the kitty depends on this for seed,
			// so changing this allows you to have a different kitty id
			System::set_block_number(System::block_number() + 1);
		}

		// Can't create 1 more
		assert_noop!(
			SubstrateKitties::create_kitty(Origin::signed(10)),
			Error::<Test>::TooManyOwned
		);
	});
}

#[test]
fn breed_kitty_checks_same_owner() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check breed kitty checks the same owner.
		// Get the kitty owned by account #1
		let kitty_1 = KittiesOwned::<Test>::get(1)[0];

		// Another kitty from another owner
		let kitty_2 = KittiesOwned::<Test>::get(2)[0];

		// Breeder can only breed kitties they own
		assert_noop!(
			SubstrateKitties::breed_kitty(Origin::signed(1), kitty_1, kitty_2),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn ensure_opposite_gender() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check that breed kitty checks opposite gender
		let kitty_1 = [1u8; 16];
		let kitty_2 = [3u8; 16];

		// Mint a Female kitty
		assert_ok!(SubstrateKitties::mint(&3, kitty_1, Gender::Female));

		// Mint another Female kitty
		assert_ok!(SubstrateKitties::mint(&3, kitty_2, Gender::Female));

		// Same gender kitty can't breed
		assert_noop!(
			SubstrateKitties::breed_kitty(Origin::signed(3), kitty_1, kitty_2),
			Error::<Test>::CantBreed
		);
	});
}

#[test]
fn dna_helpers_should_work() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Test gen_dna and other dna functions behave as expected
		// Get two kitty dnas
		let dna_1 = [1u8; 16];
		let dna_2 = [2u8; 16];

		// Generate unique Gender and DNA
		let (dna, gender) = SubstrateKitties::breed_dna(&dna_1, &dna_2);

		// Check that the new kitty is actually a child of one of its parents
		// DNA bytes must be a mix of mom or dad's DNA
		for &i in dna.iter() {
			assert!(i == 1u8 || i == 2u8)
		}

		// Test that randomness works in same block
		let (random_dna_1, _) = SubstrateKitties::gen_dna();
		// increment extrinsic index
		frame_system::Pallet::<Test>::set_extrinsic_index(1);
		let (random_dna_2, _) = SubstrateKitties::gen_dna();

		// Random values should not be equal
		assert_ne!(random_dna_1, random_dna_2);
	});
}

#[test]
fn transfer_fails_to_self() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check transfer fails when transferring to self
		// Get kitty info from account 1
		let id = KittiesOwned::<Test>::get(1)[0];

		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(1), 1, id),
			Error::<Test>::TransferToSelf
		);

		// Check transfer fails when no kitty exists
		let random_id = [0u8; 16];

		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(2), 1, random_id),
			Error::<Test>::NoKitty
		);

		// Check that transfer fails when max kitty is reached
		// Create `MaxKittiesOwned` kitties for account #10
		for _i in 0..<Test as Config>::MaxKittiesOwned::get() {
			assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
			System::set_block_number(System::block_number() + 1);
		}

		// Get a kitty to transfer
		let kitty_1 = KittiesOwned::<Test>::get(1)[0];

		// Account #10 should not be able to receive a new kitty
		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(1), 10, kitty_1),
			Error::<Test>::TooManyOwned
		);
	});
}

#[test]
fn buy_kitty_works() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check buy_kitty works as expected
		let id = KittiesOwned::<Test>::get(2)[0];
		let set_price = 4;
		let balance_1_before = Balances::free_balance(&1);
		let balance_2_before = Balances::free_balance(&2);

		// Account #2 sets a price of 4 for their kitty
		assert_ok!(SubstrateKitties::set_price(Origin::signed(2), id, Some(set_price)));

		// Account #1 can buy account #2's kitty
		assert_ok!(SubstrateKitties::buy_kitty(Origin::signed(1), id, set_price));

		// Balance transfer works as expected
		let balance_1_after = Balances::free_balance(&1);
		let balance_2_after = Balances::free_balance(&2);

		assert!(balance_1_before - set_price == balance_1_after);
		assert!(balance_2_before + set_price == balance_2_after);

		// Kitty is not for sale
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(10), id, 2),
			Error::<Test>::NotForSale
		);
	});
}

#[test]
fn price_too_low() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check buy_kitty fails when bid price is too low

		// New price is set to 4
		let id = KittiesOwned::<Test>::get(2)[0];
		let set_price = 4;
		assert_ok!(SubstrateKitties::set_price(Origin::signed(2), id, Some(set_price)));

		// Account #10 cant buy this kitty for this price
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(10), id, 2),
			Error::<Test>::BidPriceTooLow
		);
	});
}

#[test]
fn high_bid_transfers_correctly() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check buy_kitty transfers the right amount when bid price is too high
		// First get the balances of each account
		let balance_1_before = Balances::free_balance(&1);
		let balance_2_before = Balances::free_balance(&2);

		// Account #2 sets new price to 4
		let id = KittiesOwned::<Test>::get(2)[0];
		let set_price = 4;
		assert_ok!(SubstrateKitties::set_price(Origin::signed(2), id, Some(set_price)));

		// Account #1 buys kitty at 10x the price
		assert_ok!(SubstrateKitties::buy_kitty(Origin::signed(1), id, set_price * 10));

		// Balance transfer worked as expected
		let balance_1_after = Balances::free_balance(&1);
		let balance_2_after = Balances::free_balance(&2);

		assert!(balance_1_before - set_price * 10  == balance_1_after);
		assert!(balance_2_before + set_price * 10 == balance_2_after);


		// Check that it's not possible to buy a kitty if max is reached
		// Set max kitties for account #10
		for _i in 0..<Test as Config>::MaxKittiesOwned::get() {
			assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
			System::set_block_number(System::block_number() + 1);
		}
		// Account #10 should not be able to buy a new kitty
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(10), id, set_price * 10),
			Error::<Test>::TooManyOwned
		);
	});
}

#[test]
fn too_low_balance_should_fail() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check buy_kitty fails when balance is too low

		// Use some kitty in storage owned by account 2 and set a high price
		let id = KittiesOwned::<Test>::get(2)[0];
		let price = u64::MAX;
		assert_ok!(SubstrateKitties::set_price(Origin::signed(2), id, Some(price)));

		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(1), id, price),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn kitty_not_for_sale() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check buy_kitty fails when kitty is not for sale
		let id = KittiesOwned::<Test>::get(1)[0];
		// Kitty is not for sale
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(2), id, 2),
			Error::<Test>::NotForSale
		);
	});
}

#[test]
fn set_price_works() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Check set_price works as expected

		// New price is set to 4
		let id = KittiesOwned::<Test>::get(2)[0];
		let set_price = 4;
		assert_ok!(SubstrateKitties::set_price(Origin::signed(2), id, Some(set_price)));

		// Only owner can set price
		assert_noop!(
			SubstrateKitties::set_price(Origin::signed(1), id, Some(set_price)),
			Error::<Test>::NotOwner
		);

	});
}

#[test]
fn not_owner_cant_set_price() {
	new_test_ext(vec![
		(1, *b"1234567890123456", Gender::Female),
		(2, *b"123456789012345a", Gender::Male),
	])
	.execute_with(|| {
		// Create kitty
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(10)));
		let id = KittiesOwned::<Test>::get(10)[0];

		// Check set_price fails when not owner
		let new_price = 4;

		assert_noop!(
			SubstrateKitties::set_price(Origin::signed(1), id, Some(new_price)),
			Error::<Test>::NotOwner
		);
	});
}
