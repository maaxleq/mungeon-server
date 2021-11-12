use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct Seeder {
    store: Vec<u8>,
    cursor: usize,
}

impl Seeder {
    pub fn try_from_seed(seed: String) -> Result<Seeder, ParseIntError> {
        let mut store: Vec<u8> = Vec::new();
        let mut strings: Vec<String> = Vec::new();

        let mut new_string = true;
        let mut current_string = String::new();

        for (i, c) in seed.chars().enumerate() {
            if new_string {
                current_string = String::new();
                current_string.push(c);

                if i == seed.len() - 1 {
                    strings.push(current_string.clone());
                }
            } else {
                current_string.push(c);
                strings.push(current_string.clone());
            }

            new_string = !new_string;
        }

        for s in strings.iter() {
            store.push(u8::from_str_radix(s.as_str(), 16)?);
        }

        Ok(Seeder {
            store: store,
            cursor: 0,
        })
    }

    pub fn seed(&mut self) -> u8 {
        let seed = self.store[self.cursor];
        self.cursor = (self.cursor + 1) % self.store.len();

        seed
    }

    pub fn seed_u32(&mut self) -> u32 {
        ((self.seed() as u32) << 24)
            + ((self.seed() as u32) << 16)
            + ((self.seed() as u32) << 8)
            + ((self.seed() as u32) << 0)
    }

    pub fn seed_u32_bounded(&mut self, lower_bound: u32, upper_bound: u32) -> u32 {
        self.seed_u32() % (upper_bound - lower_bound + 1) + lower_bound
    }
}
