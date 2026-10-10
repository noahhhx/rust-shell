use std::{
    ffi::OsStr,
    io::Read,
    process::{Command, Stdio},
    thread,
};

use crate::commands::{command::Statement, outcome::Outcome, stream::handle};

#[derive(Debug)]
pub enum JobStatus {
    Running,
}

pub struct Job {
    pub job_number: u16,
    pub proc_id: u32,
    pub cmd_string: String,
    pub status: JobStatus,
}

#[derive(Default)]
pub struct Jobs {
    jobs: Vec<Job>,
}

impl Jobs {
    pub fn get_jobs(&self) -> &Vec<Job> {
        &self.jobs
    }

    pub fn add_job(&mut self, proc_id: u32, cmd: &str) {
        let job_number = self.jobs.len() + 1;
        self.jobs.push(Job {
            job_number: job_number as u16,
            proc_id,
            cmd_string: cmd.to_string(),
            status: JobStatus::Running,
        });
    }

    pub fn run_job(&mut self, cmd: &OsStr, stmt: &Statement) -> Outcome {
        let mut child = Command::new(cmd)
            .args(&stmt.args[0..stmt.args.len() - 1])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute process");

        let mut test = stmt.program.clone() + " ";
        test.push_str(&stmt.args.join(" "));

        self.add_job(child.id(), &test.to_string());

        let mut stdout = child.stdout.take().unwrap();
        thread::spawn(move || {
            let mut string: String = String::new();
            let _test = stdout.read_to_string(&mut string).unwrap();
            handle(
                Outcome::Output {
                    stdout: string.trim().to_string(),
                    stderr: String::new(),
                },
                &Vec::new(),
                true,
            );
        });

        Outcome::Output {
            stdout: format!("[1] {}", child.id()),
            stderr: String::new(),
        }
    }
}
