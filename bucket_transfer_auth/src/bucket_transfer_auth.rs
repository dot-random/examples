use scrypto::prelude::*;
use random::Random;

#[blueprint]
#[types(u16, u32)]
mod example {
    extern_blueprint!(
        "package_sim1p5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycnnzj0hj",
        MyRandom as RandomComponent {
            fn request_random(&self, address: ComponentAddress, method_name: String, on_error: String,
                key: u32, badge_opt: Option<FungibleBucket>, expected_fee: u8) -> u32;

        }
    );
    const RNG: Global<RandomComponent> = global_component!(
        RandomComponent,
        "component_sim1cqqqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycnf7v0gx"
    );

    struct ExampleCaller {
        // nft id, e.g. 1-1000
        next_id: u16,
        // all traits (in this demo - just a raw random number) by id
        nfts: KeyValueStore<u16, u32>,
        badge_vault: Vault,
    }

    impl ExampleCaller {
        pub fn instantiate() -> Global<ExampleCaller> {
            // controls actual minting. should be recallable, non-transferable, etc, but omitted for simplicity
            let nft_minter_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata!(
                    init {
                        "name" => "ExampleCaller NFT Minter", locked;
                    }
                ))
                .mint_initial_supply(1000)
                .into();
            return Self {
                next_id: 1,
                nfts: KeyValueStore::new_with_registered_type(),
                badge_vault: Vault::with_bucket(nft_minter_badge),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
        }

        /// Request random mint. Called by the User.
        pub fn request_mint(&mut self) -> u32 {
            debug!("EXEC:ExampleCaller::request_mint()\n");
            /* 1. consume payment for mint here */
            /* ... */

            // 2. Request mint
            let nft_id = self.next_id;
            self.next_id += 1;
            // The address of your Component
            let address = Runtime::global_component().address();
            // A key that will be sent back to you with the callback
            let key = nft_id.into();
            // The method on your component to call back
            let method_name: String = "do_mint".into();
            // The method on yor component that will be called if do_mint() panics
            let on_error: String = "abort_mint".into();
            // The token that you are going to send in `request_random()`. None if you send None there.
            // A token that will be sent back to you with the callback
            // You should check that the token is present before minting
            let badge = self.badge_vault.take(Decimal::ONE);
            // How much you would expect the callback to cost, cents (e.g. test on Stokenet).
            // It helps to avoid a sharp increase in royalties during the first few invocations of `request_random()`
            // but is completely optional.
            let expected_fee = 6u8; // 6 cents = 1 XRD
            return RNG.request_random(address, method_name, on_error, key, Some(badge.as_fungible()), expected_fee);
        }

        /// Executed by our RandomWatcher off-ledger service (through [RandomComponent]).
        /// "nft_id" here is whatever was sent to RNG.request_random() above.
        pub fn do_mint(&mut self, nft_id: u32, badge: FungibleBucket, random_seed: Vec<u8>) {
            debug!("EXEC:ExampleCaller::do_mint({:?}, {:?}, {:?})\n", nft_id, badge, random_seed);
            if badge.amount() == Decimal::ONE {
                let bucket = badge.into();
                self.badge_vault.put(bucket); // fails if a different token was sent.

                // 2. seed the random
                let mut random: Random = Random::new(&random_seed);
                let random_traits = random.next_int::<u32>();

                self.nfts.insert(nft_id as u16, random_traits);
            }
        }

        pub fn abort_mint(&mut self, _nft_id: u32, _badge: FungibleBucket) {
            // revert what you did in `request_mint()` here
        }
    }
}