use std::{
    ffi::OsStr,
    io::Read,
    process::{Child, Command, Stdio},
    thread,
};

use crate::commands::{
    command::Statement,
    outcome::Outcome::{self},
    stream::handle,
};

#[derive(Debug)]
pub enum JobStatus {
    Running,
    Done,
}

pub struct Job {
    pub number: u16,
    pub proc_id: u32,
    pub cmd_string: String,
    pub status: JobStatus,
    pub child: Child,
}

#[derive(Default)]
pub struct Jobs {
    jobs: Vec<Job>,
}

impl Jobs {
    pub fn get_jobs(&self) -> &Vec<Job> {
        &self.jobs
    }

    pub fn add_job(&mut self, proc_id: u32, cmd: &str, child: Child) -> u16 {
        let job_number = self.jobs.len() + 1;
        self.jobs.push(Job {
            number: u16::try_from(job_number).unwrap(),
            proc_id,
            cmd_string: cmd.to_string(),
            status: JobStatus::Running,
            child,
        });
        u16::try_from(job_number).unwrap()
    }

    pub fn reap_jobs(&mut self) -> String {
        let mut ret = String::new();
        self.jobs.retain_mut(|job| match job.child.try_wait() {
            Ok(Some(status)) => {
                let cmd_string: Vec<&str> = job.cmd_string.split("&").collect();
                ret = format!(
                    "[{}]{}  Done{:space$}{}",
                    job.number,
                    "+",
                    " ",
                    cmd_string.first().unwrap(),
                    space = (24 - 4)
                );
                false
            }
            Ok(None) => true,
            Err(err) => {
                println!("");
                true
            }
        });
        ret
    }

    pub fn run_job(&mut self, cmd: &OsStr, stmt: &Statement) -> Outcome {
        let mut child = Command::new(cmd)
            .args(&stmt.args[0..stmt.args.len() - 1])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute process");

        let proc_id = child.id();
        let mut input = stmt.program.clone() + " ";
        input.push_str(&stmt.args.join(" "));

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

        let job_id = self.add_job(child.id(), &input.clone(), child);

        Outcome::Output {
            stdout: format!("[{job_id}] {}", proc_id),
            stderr: String::new(),
        }
    }
}
