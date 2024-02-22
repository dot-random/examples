use random::Random;

pub fn get_random(seed: &Vec<u8>) -> Random {
    Random::new(&seed)
}