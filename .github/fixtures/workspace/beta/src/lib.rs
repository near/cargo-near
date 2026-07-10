use near_sdk::near;

#[near(contract_state)]
#[derive(Default)]
pub struct Contract {
    value: u32,
}

#[near]
impl Contract {
    pub fn label(&self) -> &'static str {
        if cfg!(feature = "featured") {
            "featured"
        } else {
            "plain"
        }
    }

    pub fn get(&self) -> u32 {
        self.value
    }
}
