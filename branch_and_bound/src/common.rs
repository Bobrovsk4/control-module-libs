#[allow(dead_code)] // чтобы warning на method_name не мешал
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgResult {
    pub sequence: Vec<usize>,
    pub schedule: Vec<Vec<(i32, i32)>>,
    pub makespan: i32,
    pub idle_times: Vec<i32>,
    pub method_name: String,
}

pub fn build_schedule(
    matrix: &Vec<Vec<i32>>,
    sequence: &Vec<usize>,
) -> Result<(Vec<Vec<(i32, i32)>>, i32, Vec<i32>), String> {
    let num_jobs = sequence.len();
    let num_machines = matrix[0].len();

    let mut schedule = vec![vec![(0, 0); num_machines]; num_jobs];

    let first_job = sequence[0];
    let mut current_time = 0;
    for machine in 0..num_machines {
        let proc_time = matrix[first_job][machine];
        schedule[0][machine] = (current_time, current_time + proc_time);
        current_time += proc_time;
    }

    for (seq_idx, &job_idx) in sequence.iter().enumerate().skip(1) {
        let prev_out_m1 = schedule[seq_idx - 1][0].1;
        let proc_time_m1 = matrix[job_idx][0];
        schedule[seq_idx][0] = (prev_out_m1, prev_out_m1 + proc_time_m1);

        for machine in 1..num_machines {
            let out_prev_machine = schedule[seq_idx][machine - 1].1;
            let out_prev_job = schedule[seq_idx - 1][machine].1;
            let in_time = out_prev_machine.max(out_prev_job);
            let proc_time = matrix[job_idx][machine];
            schedule[seq_idx][machine] = (in_time, in_time + proc_time);
        }
    }

    let makespan = schedule[num_jobs - 1][num_machines - 1].1;

    let mut idle_times = vec![0; num_machines];
    for machine in 0..num_machines {
        let mut total_idle = schedule[0][machine].0;

        for seq_idx in 1..num_jobs {
            let gap = schedule[seq_idx][machine].0 - schedule[seq_idx - 1][machine].1;
            if gap > 0 {
                total_idle += gap;
            }
        }

        idle_times[machine] = total_idle;
    }

    Ok((schedule, makespan, idle_times))
}

pub fn create_result(
    matrix: &Vec<Vec<i32>>,
    sequence: Vec<usize>,
    name: &str,
) -> Result<AlgResult, String> {
    let (schedule, makespan, idle_times) = build_schedule(matrix, &sequence)?;
    Ok(AlgResult {
        sequence,
        schedule,
        makespan,
        idle_times,
        method_name: name.into(),
    })
}
