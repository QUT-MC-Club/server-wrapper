use std::io;

use tokio::process;

pub struct Executor {
    tasks: Vec<String>,
}

impl Executor {
    pub fn new(tasks: Vec<String>) -> Executor {
        Executor { tasks }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        for task in &self.tasks {
            println!("executing: '{}'", task);

            let task: Vec<&str> = task.split_ascii_whitespace().collect();

            let mut command = process::Command::new(task[0]);
            command.args(&task[1..]);

            command.spawn()?.wait().await?;
        }

        Ok(())
    }
}
