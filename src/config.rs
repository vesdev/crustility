#[derive(Default)]
pub struct Config {
    pub keys: Vec<Key>,
}

pub enum Key {
    HallEffect {
        rt: bool, //rapid trigger
        rt_down_sensitivity: usize,
        rt_up_sensitivity: usize,
        lower_hysterisis: usize,
        upper_hysterisis: usize,
    },
}
