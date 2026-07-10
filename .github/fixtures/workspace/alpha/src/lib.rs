use near_sdk::near;

#[near(contract_state)]
#[derive(Default)]
pub struct Contract {
    value: u32,
}

#[near]
impl Contract {
    pub fn get(&self) -> u32 {
        self.value
    }

    pub fn set(&mut self, value: u32) {
        self.value = value;
    }
}
