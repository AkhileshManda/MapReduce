//! If you wish to encapsulate logic related to
//! task and job management, you can do so here.
//!

use std::collections::HashMap;

use crate::rpc::coordinator::JobRequestReply;
// You are not required to modify this file.
#[derive(PartialEq, Clone)]
pub enum JobType {
    MapJob,
    ReduceJob,
}

#[derive(PartialEq, Clone)]
pub struct SubJob {
    /** SUB JOB ID */
    pub id: u32,
    /** ID OF PARENT JOB */
    pub parent_job_id: u32,
    /** INPUT FILE FOR MAP */
    pub file: String,
    /** WHO WORKED ON THIS JOB */
    pub worker_id: u32,
    /** WHETHER MAP OR REDUCE TYPE JOB */
    pub type_job: JobType,
    /** WHAT IS THE MAP APP */
    pub app: String,
    /** NUM OF REDUCE BUCKETS */
    pub n_reduce: u32,
    /** INPUT ARGS FOR MAP */
    pub args: Vec<u8>,
    /** STATE WHETHER JOB COMPLETED */
    pub done: bool,
    /** STATE WHETHER JOB FAILED */
    pub failed: bool,
    /** OUTPUT DIR */
    pub output_dir: String,
}

pub struct Job {
    /** JOB ID */
    pub job_id: u32,
    /** MAP JOB ID AND THE MAP JOBS */
    pub map_jobs: HashMap<u32, SubJob>,
    /** REDUCE JOB ID AND THE REDUCE JOB */
    pub reduce_jobs: HashMap<u32, SubJob>,
    /** WHETHER THE JOB HAS COMPLETED */
    pub done: bool,
    /** WHETHER THE JOB HAS FAILED */
    pub failed: bool,
    /** THE ERROR ASSOCIATED WITH A ERRORED JOB */
    pub error_arr: Vec<String>,
}

impl Job {
    pub fn new(j_id: u32) -> Self {
        Job {
            job_id: j_id,
            map_jobs: HashMap::new(),
            reduce_jobs: HashMap::new(),
            done: false,
            failed: false,
            error_arr: Vec::new(),
        }
    }

    /** ADD A JOB */
    pub fn add_job(&mut self, j: SubJob, t: JobType) {
        if t == JobType::MapJob {
            self.map_jobs.insert(j.id, j);
        } else {
            self.reduce_jobs.insert(j.id, j);
        }
    }

    /** RETURNS WHETHER ALL MAP JOBS OF A JOB ARE DONE */
    pub fn all_map_done(&self) -> bool {
        for j in &self.map_jobs {
            let s = j.1;
            if !s.done {
                return false;
            }
        }
        return true;
    }

    /** CHECK WHETHER ALL REDUCE JOBS ARE DONE */
    pub fn all_reduce_done(&self) -> bool {
        for j in &self.reduce_jobs {
            let s = j.1;
            if !s.done {
                return false;
            }
        }
        return true;
    }

    /** MARK A MAP JOB AS DONE */
    pub fn map_job_done(&mut self, sub_job_id: u32) {
        let sub_job = self.map_jobs.get_mut(&sub_job_id);
        match sub_job {
            Some(sj) => {
                sj.done = true;
            }
            None => {}
        }
    }

    /** MARK A REDUCE JOB AS DONE */
    pub fn reduce_job_done(&mut self, sub_job_id: u32) {
        let sub_job = self.reduce_jobs.get_mut(&sub_job_id);
        match sub_job {
            Some(sj) => {
                sj.done = true;
            }
            None => {}
        }
    }

    /** RETURN IF ALL JOBS ARE DONE */
    pub fn all_done(&mut self) -> bool {
        self.done = self.all_map_done() && self.all_reduce_done();
        return self.done;
    }

    /** SET THE STATUS OF DONE */
    pub fn set_done(&mut self, s: bool) {
        self.done = s;
    }

    /** SET THE STATUS OF FAILED */
    pub fn set_failed(&mut self, s: bool) {
        self.failed = s;
    }

    /** RETURN A JOB REQUEST OF NEXT AVAILABLE MAP JOB */
    pub fn assign_and_ret_map_job(
        &mut self,
        worker_id: u32,
    ) -> Option<(JobRequestReply, u32, SubJob)> {
        // todo:
        for sj in self.map_jobs.iter_mut() {
            let sub_job = sj.1;
            if !sub_job.done && sub_job.worker_id == 0 {
                let job_req_req = JobRequestReply {
                    file: sub_job.file.to_string(),
                    args: sub_job.args.clone(),
                    // 0 is map job
                    job_type: 0,
                    app: sub_job.app.to_string(),
                    worker_ids: vec![],
                    parent_job_id: sub_job.parent_job_id,
                    job_id: sub_job.id,
                    reduce_bucket: 0,
                    valid: 1,
                    n_reduce: sub_job.n_reduce,
                    output_dir: sub_job.output_dir.clone(),
                };
                sub_job.worker_id = worker_id;
                println!("SENDING MAP JOB {} TO WORKER {}", sub_job.id, worker_id);
                return Some((job_req_req, sub_job.parent_job_id, sub_job.clone()));
            }
        }
        return None;
    }

    /* RETURN A JOB REQUEST OF NEXT AVAILABLE REDUCE JOB */
    pub fn assign_and_ret_reduce_job(
        &mut self,
        worker_id: u32,
    ) -> Option<(JobRequestReply, u32, SubJob)> {
        //todo:
        let mut workers_on_job: Vec<u32> = Vec::new();

        for sj in self.map_jobs.iter_mut() {
            let sub_map_job = sj.1;
            assert!(sub_map_job.done);
            /* PUSH THE WORKER WHO WORKED ON THIS JOB */
            workers_on_job.push(sub_map_job.worker_id);
        }

        for sj in self.reduce_jobs.iter_mut() {
            let sub_reduce_job = sj.1;
            if !sub_reduce_job.done && sub_reduce_job.worker_id == 0 {
                let job_req_req = JobRequestReply {
                    file: "".to_string(), // not needed in reduce
                    args: sub_reduce_job.args.clone(),
                    job_type: 1, // 1  is reduce job
                    app: sub_reduce_job.app.to_string(),
                    worker_ids: workers_on_job.clone(),
                    parent_job_id: sub_reduce_job.parent_job_id,
                    job_id: sub_reduce_job.id,
                    reduce_bucket: sub_reduce_job.id,
                    valid: 1,
                    n_reduce: sub_reduce_job.n_reduce,
                    output_dir: sub_reduce_job.output_dir.clone(),
                };
                sub_reduce_job.worker_id = worker_id;
                println!(
                    "SENDING REDUCE JOB {} TO WORKER {}",
                    sub_reduce_job.id, worker_id
                );
                return Some((
                    job_req_req,
                    sub_reduce_job.parent_job_id,
                    sub_reduce_job.clone(),
                ));
            }
        }
        return None;
    }
}

pub struct WC {
    pub worker_id: u32,
    pub hb_t: u128,
    pub jobs_done: Vec<SubJob>,
}

impl WC {
    pub fn update(&mut self, v: u128) {
        self.hb_t = v;
    }
}
