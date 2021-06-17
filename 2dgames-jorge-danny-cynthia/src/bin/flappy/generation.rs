use rand::Rng;

pub struct Obstacles {
    // tuple vec of objects one for the top and one for the bottom
    pub obstacles: Vec<(i32, i32)>,

    // how frequently we want each to be created
    pub frequency_values: Vec<usize>,
}

// randomly picks a pair of obstacles to generate
impl Obstacles {
    pub fn generate_obstacles(&self) -> (i32, i32) {
        let freq_total: usize = self.frequency_values.iter().sum();
        let mut rng = rand::thread_rng();
        let mut x: i32 = rng.gen_range((0 as i32)..(freq_total as i32));

        for (f_vals, obs) in self.frequency_values.iter().zip(self.obstacles.iter()) {
            x -= *f_vals as i32;
            if x < 0 {
                return *obs;
            }
        }
        self.obstacles[self.obstacles.len() - 1]
    }
}
