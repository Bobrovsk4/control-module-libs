mod common;
mod gantt_chart;

use crate::common::{build_schedule, create_result, AlgResult};
use crate::gantt_chart::draw_gantt;

#[unsafe(no_mangle)]
pub extern "C" fn name() -> String {
    String::from("Метод полного перебора")
}

#[unsafe(no_mangle)]
pub extern "C" fn exec(matrix: &Vec<Vec<i32>>) -> Result<(AlgResult, i32), String> {
    let n = matrix.len();
    if n > 10 {
        return Err("Слишком много задач (>10)".into());
    }

    let mut best_seq = (0..n).collect();
    let mut best_makespan = i32::MAX;

    let mut perms = Vec::new();
    generate_perms(n, &mut Vec::new(), &mut perms);

    for seq in perms {
        let (_, makespan, _) = build_schedule(matrix, &seq)?;
        if makespan < best_makespan {
            best_makespan = makespan;
            best_seq = seq;
        }
    }

    let orig_seq: Vec<usize> = (0..matrix.len()).collect();
    let orig_result = create_result(matrix, orig_seq, "Метод полного перебора (исходный)");

    let final_result = create_result(matrix, best_seq, "Метод полного перебора (финальный)");

    draw_gantt(&orig_result.clone()?, &matrix.clone(), "orig.svg");
    draw_gantt(&final_result.clone()?, &matrix.clone(), "final.svg");

    Ok((final_result.unwrap(), orig_result.unwrap().makespan))
}

fn generate_perms(n: usize, cur: &mut Vec<usize>, res: &mut Vec<Vec<usize>>) {
    if cur.len() == n {
        res.push(cur.clone());
        return;
    }
    for i in 0..n {
        if !cur.contains(&i) {
            cur.push(i);
            generate_perms(n, cur, res);
            cur.pop();
        }
    }
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
