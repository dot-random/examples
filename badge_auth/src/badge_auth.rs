use scrypto::prelude::*;
use random::Random;

#[blueprint]
#[types(u16, u32)]
mod example {
    extern_blueprint!(
        "package_sim1p5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycnnzj0hj",
        MyRandom as RandomComponent {
            fn request_random2(&self, address: ComponentAddress, method_name: String, on_error: String, key: u32) -> u32;
        }
    );
    const RNG: Global<RandomComponent> = global_component!(
        RandomComponent,
        "component_sim1cqqqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycnf7v0gx"
    );
    const BADGE_RESOURCE: ResourceManager = resource_manager!("resource_sim1t5qqqqqqqyqszqgqqqqqqqgpqyqsqqqqxumnwqgqqqqqqycn38dnjs");

    enable_method_auth! {
        roles {
            random_provider => updatable_by: [];
        },
        methods {
            request_mint => PUBLIC;
            do_mint => restrict_to: [random_provider];
            abort_mint => restrict_to: [random_provider];
        }
    }
    struct ExampleCallerBadgeAuth {
        // nft id, e.g. 1-1000
        next_id: u16,
        // all traits (in this demo - just a raw random number) by id
        nfts: KeyValueStore<u16, u32>,
    }

    impl ExampleCallerBadgeAuth {
        pub fn instantiate() -> Global<ExampleCallerBadgeAuth> {
            debug!("EXEC:ExampleCallerBadgeAuth::instantiate()\n");

            let badge_address: ResourceAddress = BADGE_RESOURCE.address();
            return Self {
                next_id: 1,
                nfts: KeyValueStore::new_with_registered_type(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .roles(roles!(
                    random_provider => rule!(require(badge_address));
                ))
                .globalize();
        }

        /// Request random mint. Called by the User.
        pub fn request_mint(&mut self) -> u32 {
            debug!("EXEC:ExampleCallerBadgeAuth::request_mint()\n");
            /* 1. consume payment for mint here */
            /* ... */

            // 2. Request mint
            let nft_id = self.next_id;
            self.next_id += 1;
            // The address of your Component
            let address = Runtime::global_component().address();
            // The method on your component to call back
            let method_name = "do_mint".into();
            // The method on yor component that will be called if do_mint() panics
            let on_error = "abort_mint".into();
            // A key that will be sent back to you with the callback
            let key = nft_id.into();
            return RNG.request_random2(address, method_name, on_error, key);
        }

        /// Executed by our RandomWatcher off-ledger service (through [RandomComponent]).
        /// "nft_id" here is whatever was sent to RNG.request_random() above.
        pub fn do_mint(&mut self, nft_id: u32, random_seed: Vec<u8>) {
            debug!("EXEC:ExampleCallerBadgeAuth::do_mint({:?}, {:?})\n", nft_id, random_seed);
            // 2. seed the random
            let mut random: Random = Random::new(random_seed.as_slice());
            let random_traits = random.next_int::<u32>();

            self.nfts.insert(nft_id as u16, random_traits);
        }

        pub fn abort_mint(&mut self, nft_id: u32) {
            // revert what you did in `request_mint()` here
        }
    }
}