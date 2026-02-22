mod common;
mod gantt_chart;

use crate::common::{AlgResult, create_result};
use crate::gantt_chart::draw_gantt;
use std::ffi::{CString, c_char};

#[unsafe(no_mangle)]
pub extern "C" fn name() -> *const c_char {
    CString::new("Метод приоритетов").unwrap().into_raw()
}

fn exec_alg(matrix: &Vec<Vec<i32>>) -> Result<(AlgResult, i32), String> {
    if matrix[0].len() != 2 {
        return Err("Нужно 2 станка".into());
    }
    let max_val = matrix.iter().flatten().copied().max().unwrap_or(1);
    let mut jobs: Vec<(usize, i32)> = matrix
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let (a, b) = (r[0], r[1]);
            let sign = if a < b { 1 } else { -1 };
            let p = sign * (max_val - a.min(b));
            (i, p)
        })
        .collect();
    jobs.sort_by_key(|k| std::cmp::Reverse(k.1));
    let sequence = jobs.into_iter().map(|(i, _)| i).collect();

    let orig_seq: Vec<usize> = (0..matrix.len()).collect();
    let orig_result = create_result(matrix, orig_seq, "Метод приоритетов (исходный)");

    let final_result = create_result(matrix, sequence, "Метод приоритетов (финальный)");

    draw_gantt(&orig_result.clone()?, &matrix.clone(), "orig.svg");
    draw_gantt(&final_result.clone()?, &matrix.clone(), "final.svg");

    Ok((final_result.unwrap(), orig_result.unwrap().makespan))
}

#[unsafe(no_mangle)]
pub extern "C" fn exec(matrix: &Vec<Vec<i32>>) -> *const c_char {
    let (result, initial_makespan) = exec_alg(matrix).expect("Ошибка выполнения алгоритма");
    let mut output = String::new();

    if matrix[0].len() == 2 {
        output.push_str("Оптимальная последовательность:\n");
    } else {
        output.push_str("Последовательность:\n");
    }

    output.push_str(&format!(
        "  {}\n",
        result
            .sequence
            .iter()
            .map(|&idx| format!("J{}", idx + 1))
            .collect::<Vec<_>>()
            .join(" → ")
    ));

    output.push_str("\nРасписание (вход → выход):\n");
    output.push_str("Работа| ");
    for machine in 0..matrix[0].len() {
        output.push_str(&format!("   M{}  | ", machine + 1));
    }
    output.push_str("\n");
    output.push_str(&"-------+".repeat(matrix[0].len()));
    output.push_str("-\n");

    for (seq_idx, &job_idx) in result.sequence.iter().enumerate() {
        output.push_str(&format!("   J{}    | ", job_idx + 1));
        for machine in 0..matrix[0].len() {
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
        output.push_str(&format!("M{}: {}\n", machine + 1, idle));
    }

    CString::new(output).unwrap().into_raw()
}
