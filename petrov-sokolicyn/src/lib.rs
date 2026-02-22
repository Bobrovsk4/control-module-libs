mod common;
mod gantt_chart;

use crate::common::{AlgResult, build_schedule, create_result};
use crate::gantt_chart::draw_gantt;
use std::ffi::{CString, c_char};

#[unsafe(no_mangle)]
pub extern "C" fn name() -> *const c_char {
    CString::new("Метод Петрова-Соколицына").unwrap().into_raw()
}

fn exec_alg(matrix: &Vec<Vec<i32>>) -> Result<(AlgResult, i32), String> {
    if matrix.is_empty() {
        return Err("Матрица пуста".to_string());
    }

    let num_jobs = matrix.len();
    let num_machines = matrix[0].len();

    if num_machines < 2 {
        return Err("Требуется минимум 2 станка для метода Петрова-Соколицына".to_string());
    }

    for (i, row) in matrix.iter().enumerate() {
        if row.len() != num_machines {
            return Err(format!(
                "Неравномерная матрица: строка {} имеет {} элементов, ожидалось {}",
                i,
                row.len(),
                num_machines
            ));
        }
        for (j, &time) in row.iter().enumerate() {
            if time < 0 {
                return Err(format!(
                    "Отрицательное время обработки в работе {} на станке {}",
                    i, j
                ));
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct JobMetric {
        job_idx: usize,
        s1: i32,
        s2: i32,
        d: i32,
    }

    let jobs: Vec<JobMetric> = matrix
        .iter()
        .enumerate()
        .map(|(idx, times)| {
            let s1: i32 = times.iter().skip(1).sum();
            let s2: i32 = times.iter().take(num_machines - 1).sum();
            let d = s1 - s2;

            JobMetric {
                job_idx: idx,
                s1,
                s2,
                d,
            }
        })
        .collect();

    let mut seq_s1 = jobs.clone();
    seq_s1.sort_by(|a, b| b.s1.cmp(&a.s1));
    let sequence_s1: Vec<usize> = seq_s1.iter().map(|j| j.job_idx).collect();

    let mut seq_s2 = jobs.clone();
    seq_s2.sort_by(|a, b| a.s2.cmp(&b.s2));
    let sequence_s2: Vec<usize> = seq_s2.iter().map(|j| j.job_idx).collect();

    let mut seq_d = jobs.clone();
    seq_d.sort_by(|a, b| b.d.cmp(&a.d));
    let sequence_d: Vec<usize> = seq_d.iter().map(|j| j.job_idx).collect();

    let (schedule_s1, makespan_s1, _) = build_schedule(matrix, &sequence_s1)?;
    let (schedule_s2, makespan_s2, _) = build_schedule(matrix, &sequence_s2)?;
    let (schedule_d, makespan_d, _) = build_schedule(matrix, &sequence_d)?;

    let (best_sequence, best_schedule, best_makespan) =
        if makespan_s1 <= makespan_s2 && makespan_s1 <= makespan_d {
            (sequence_s1, schedule_s1, makespan_s1)
        } else if makespan_s2 <= makespan_d {
            (sequence_s2, schedule_s2, makespan_s2)
        } else {
            (sequence_d, schedule_d, makespan_d)
        };

    let mut idle_times = vec![0; num_machines];
    for machine in 0..num_machines {
        let mut total_idle = best_schedule[0][machine].0;
        for seq_idx in 1..num_jobs {
            let gap = best_schedule[seq_idx][machine].0 - best_schedule[seq_idx - 1][machine].1;
            if gap > 0 {
                total_idle += gap;
            }
        }
        idle_times[machine] = total_idle;
    }

    let orig_seq: Vec<usize> = (0..matrix.len()).collect();
    let orig_result = create_result(matrix, orig_seq, "Петров-Соколицын (исходный)");

    let final_result = create_result(
        matrix,
        best_sequence.clone(),
        "Петров-Соколицын (финальный)",
    );

    draw_gantt(&orig_result.clone()?, &matrix.clone(), "orig.svg");
    draw_gantt(&final_result.clone()?, &matrix.clone(), "final.svg");

    Ok((
        AlgResult {
            sequence: best_sequence,
            schedule: best_schedule,
            makespan: best_makespan,
            idle_times,
            method_name: "Petrov_Sokolicyn".to_string(),
        },
        orig_result.unwrap().makespan,
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn exec(matrix: &Vec<Vec<i32>>) -> *const c_char {
    let (result, initial_makespan) = exec_alg(matrix).expect("Ошибка выполнения алгоритма");
    let mut output = String::new();

    let num_machines = matrix[0].len();

    #[derive(Debug)]
    struct JobMetrics {
        job_idx: usize,
        job_name: String,
        times: Vec<i32>,
        s1: i32,
        s2: i32,
        d: i32,
    }

    let metrics: Vec<JobMetrics> = matrix
        .iter()
        .enumerate()
        .map(|(idx, times)| {
            let s1: i32 = times.iter().skip(1).sum();
            let s2: i32 = times.iter().take(num_machines - 1).sum();
            let d = s1 - s2;
            JobMetrics {
                job_idx: idx,
                job_name: format!("J{}", idx + 1),
                times: times.clone(),
                s1,
                s2,
                d,
            }
        })
        .collect();

    output.push_str("Метрики для каждой работы:\n");
    output.push_str(&format!(
        "{:<6} | {:<7} | {:<6} | {:<6} | {:<8}\n",
        "Работа", "Времена", "S1", "S2", "D=S1-S2"
    ));
    output.push_str(&"-".repeat(60));
    output.push_str("\n");

    for m in &metrics {
        let times_str = m
            .times
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",");
        output.push_str(&format!(
            "{:<10} | {:<13} | {:<6} | {:<8} | {:<8}\n",
            m.job_name, times_str, m.s1, m.s2, m.d
        ));
    }
    output.push_str("\n");

    output.push_str("Кандидатные последовательности:\n");

    let mut seq_s1: Vec<&JobMetrics> = metrics.iter().collect();
    seq_s1.sort_by(|a, b| b.s1.cmp(&a.s1));
    let seq_s1_indices: Vec<usize> = seq_s1.iter().map(|m| m.job_idx).collect();
    let (_, makespan_s1, _) = build_schedule(matrix, &seq_s1_indices).unwrap();
    let seq_s1_str: String = seq_s1
        .iter()
        .map(|m| m.job_name.as_str())
        .collect::<Vec<&str>>()
        .join(" → ");
    output.push_str(&format!(
        "  1) По убыванию S1: {} → makespan = {}\n",
        seq_s1_str, makespan_s1
    ));

    let mut seq_s2: Vec<&JobMetrics> = metrics.iter().collect();
    seq_s2.sort_by(|a, b| a.s2.cmp(&b.s2));
    let seq_s2_indices: Vec<usize> = seq_s2.iter().map(|m| m.job_idx).collect();
    let (_, makespan_s2, _) = build_schedule(matrix, &seq_s2_indices).unwrap();
    let seq_s2_str: String = seq_s2
        .iter()
        .map(|m| m.job_name.as_str())
        .collect::<Vec<&str>>()
        .join(" → ");
    output.push_str(&format!(
        "  2) По возрастанию S2: {} → makespan = {}\n",
        seq_s2_str, makespan_s2
    ));

    let mut seq_d: Vec<&JobMetrics> = metrics.iter().collect();
    seq_d.sort_by(|a, b| b.d.cmp(&a.d));
    let seq_d_indices: Vec<usize> = seq_d.iter().map(|m| m.job_idx).collect();
    let (_, makespan_d, _) = build_schedule(matrix, &seq_d_indices).unwrap();
    let seq_d_str: String = seq_d
        .iter()
        .map(|m| m.job_name.as_str())
        .collect::<Vec<&str>>()
        .join(" → ");
    output.push_str(&format!(
        "  3) По убыванию D: {} → makespan = {}\n",
        seq_d_str, makespan_d
    ));

    output.push_str("\n");

    output.push_str(&format!(
        "Выбрана последовательность с минимальным makespan = {}\n\n",
        result.makespan
    ));

    output.push_str("Последовательность:\n");
    let optimal_seq_str: String = result
        .sequence
        .iter()
        .map(|&idx| format!("J{}", idx + 1))
        .collect::<Vec<String>>()
        .join(" → ");
    output.push_str(&format!("  {}\n", optimal_seq_str));
    output.push_str("\n");

    output.push_str("Расписание (вход → выход):\n");
    output.push_str("Работа | ");
    for machine in 0..num_machines {
        output.push_str(&format!("M{} | ", machine + 1));
    }
    output.push_str("\n");
    output.push_str(&"-------+".repeat(num_machines));
    output.push_str("-\n");

    for (seq_idx, &job_idx) in result.sequence.iter().enumerate() {
        output.push_str(&format!("  J{}   | ", job_idx + 1));
        for machine in 0..num_machines {
            let (in_time, out_time) = result.schedule[seq_idx][machine];
            output.push_str(&format!(" {:2}→{:2} |", in_time, out_time));
        }
        output.push_str("\n");
    }

    output.push_str(&format!(
        "\nДлительность производственного цикла: {} -> {}\n",
        initial_makespan, result.makespan
    ));
    output.push_str("Простои станков:\n");
    for (machine, &idle) in result.idle_times.iter().enumerate() {
        output.push_str(&format!("  M{}: {}\n", machine + 1, idle));
    }

    CString::new(output).unwrap().into_raw()
}
