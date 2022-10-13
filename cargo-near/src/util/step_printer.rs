use colored::Colorize;

pub struct StepPrinter {
    cur_step: u32,
    max_steps: u32,
}

impl StepPrinter {
    pub fn new(max_steps: u32) -> Self {
        Self {
            cur_step: 0,
            max_steps,
        }
    }

    pub fn print_step(&mut self, msg: &str) {
        self.cur_step += 1;
        if self.cur_step > self.max_steps {
            panic!(
                "Expected process to have exactly {} steps, but encountered an extra step #{}",
                &self.max_steps, &self.cur_step
            )
        }
        eprintln!(
            " {} {}",
            format!("[{}/{}]", &self.cur_step, &self.max_steps).bold(),
            msg.green().bold()
        )
    }
}

impl Drop for StepPrinter {
    fn drop(&mut self) {
        if self.cur_step != self.max_steps {
            panic!(
                "Expected process to have exactly {} steps, but only saw {} steps",
                &self.max_steps, &self.cur_step
            )
        }
    }
}
