mod common;
mod gantt_chart;

use crate::common::{create_result, AlgResult};
use crate::gantt_chart::draw_gantt;

#[unsafe(no_mangle)]
pub extern "C" fn exec(matrix: &Vec<Vec<i32>>) -> Result<(AlgResult, i32), String> {
    let mut jobs: Vec<(usize, usize)> = matrix
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let (max_idx, _) = r.iter().enumerate().max_by_key(|&(_, v)| v).unwrap();
            (i, max_idx)
        })
        .collect();
    jobs.sort_by_key(|k| std::cmp::Reverse(k.1));
    let sequence = jobs.into_iter().map(|(i, _)| i).collect();

    let orig_seq: Vec<usize> = (0..matrix.len()).collect();
    let orig_result = create_result(
        matrix,
        orig_seq,
        "Джонсон приоритет «узкого места» (исходный)",
    );

    let final_result = create_result(
        matrix,
        sequence,
        "Джонсон приоритет «узкого места» (финальный)",
    );

    draw_gantt(&orig_result.clone()?, &matrix.clone(), "orig.svg");
    draw_gantt(&final_result.clone()?, &matrix.clone(), "final.svg");

    Ok((final_result.unwrap(), orig_result.unwrap().makespan))
}

#[unsafe(no_mangle)]
pub extern "C" fn format_result(
    result: &AlgResult,
    initial_makespan: i32,
    matrix: &Vec<Vec<i32>>,
) -> String {
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

    output
}
