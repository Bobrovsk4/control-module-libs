mod common;
mod gantt_chart;

use crate::common::{create_result, AlgResult};
use crate::gantt_chart::draw_gantt;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[unsafe(no_mangle)]
pub extern "C" fn exec(matrix: &Vec<Vec<i32>>) -> Result<(AlgResult, BranchAndBoundStats), String> {
    let time_limit_ms = 5000;
    let node_limit = 1_000_000;

    if matrix.is_empty() {
        return Err("Матрица пуста".to_string());
    }

    let num_jobs = matrix.len();
    let num_machines = matrix[0].len();

    if num_machines < 2 {
        return Err("Метод ветвей и границ требует минимум 2 станка".to_string());
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

    if num_jobs > 15 && node_limit == 0 && time_limit_ms == 0 {
        return Err("Задача слишком велика для полного перебора".to_string());
    }

    use std::time::Instant;
    let start_time = Instant::now();

    let mut pq = BinaryHeap::new();
    let mut initial = PartialSchedule::new(num_jobs);
    initial.lower_bound = initial.calculate_lower_bound(matrix, num_jobs);

    pq.push(SearchNode {
        schedule: initial.clone(),
        priority: initial.lower_bound.clone(),
    });

    let mut best_makespan = i32::MAX;
    let mut best_sequence = Vec::new();
    let mut best_schedule = Vec::new();

    let mut nodes_explored = 0;
    let mut nodes_pruned = 0;
    let mut best_found_at_node = 0;

    while let Some(SearchNode {
        schedule: partial, ..
    }) = pq.pop()
    {
        nodes_explored += 1;

        if node_limit > 0 && nodes_explored > node_limit {
            return Err(format!(
                "Превышено ограничение на количество узлов: {} > {}",
                nodes_explored, node_limit
            ));
        }

        if time_limit_ms > 0 && start_time.elapsed().as_millis() > time_limit_ms as u128 {
            return Err(format!(
                "Превышено ограничение по времени: > {} мс (исследовано {} узлов)",
                time_limit_ms, nodes_explored
            ));
        }

        if partial.lower_bound >= best_makespan {
            nodes_pruned += 1;
            continue;
        }

        if partial.is_complete(num_jobs) {
            if partial.current_makespan < best_makespan {
                best_makespan = partial.current_makespan;
                best_sequence = partial.sequence.clone();
                best_schedule = partial.schedule.clone();
                best_found_at_node = nodes_explored;
            }
            continue;
        }

        let remaining = partial.remaining_jobs(num_jobs);
        for &job_idx in &remaining {
            let mut child = partial.clone();
            child.add_job(job_idx, matrix);
            child.lower_bound = child.calculate_lower_bound(matrix, num_jobs);

            if child.lower_bound < best_makespan {
                pq.push(SearchNode {
                    schedule: child.clone(),
                    priority: child.lower_bound.clone(),
                });
            } else {
                nodes_pruned += 1;
            }
        }
    }

    if best_sequence.is_empty() {
        return Err("Не удалось найти решение".to_string());
    }

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

    let result = AlgResult {
        sequence: best_sequence.clone(),
        schedule: best_schedule,
        makespan: best_makespan,
        idle_times,
        method_name: "Branch and Bound".to_string(),
    };

    let stats = BranchAndBoundStats {
        nodes_explored,
        nodes_pruned,
        best_found_at_node,
        total_permutations: (1..=num_jobs).fold(1u128, |acc, x| acc * x as u128),
        time_ms: start_time.elapsed().as_millis() as u64,
    };

    let orig_seq: Vec<usize> = (0..matrix.len()).collect();
    let orig_result = create_result(matrix, orig_seq, "Метод ветвей и границ (исходный)");

    let final_result = create_result(matrix, best_sequence, "Метод ветвей и границ (финальный)");

    draw_gantt(&orig_result?, &matrix.clone(), "orig.svg");
    draw_gantt(&final_result.clone()?, &matrix.clone(), "final.svg");

    Ok((result, stats))
}

#[derive(Debug, Clone)]
pub struct BranchAndBoundStats {
    pub nodes_explored: usize,
    pub nodes_pruned: usize,
    pub best_found_at_node: usize,
    pub total_permutations: u128,
    pub time_ms: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn format_result(
    result: &AlgResult,
    stats: &BranchAndBoundStats,
    matrix: &Vec<Vec<i32>>,
) -> String {
    let mut output = String::new();

    output.push_str("Статистика поиска:\n");
    output.push_str(&format!(
        "  Исследовано узлов:       {:>10}\n",
        stats.nodes_explored
    ));
    output.push_str(&format!(
        "  Отсечено узлов:          {:>10}\n",
        stats.nodes_pruned
    ));
    output.push_str(&format!(
        "  Лучшее решение найдено:  на узле #{}\n",
        stats.best_found_at_node
    ));
    output.push_str(&format!(
        "  Всего перестановок ({}!): {:>10}\n",
        matrix.len(),
        stats.total_permutations
    ));
    output.push_str(&format!(
        "  Эффективность отсечения: {:>9.2}%\n",
        if stats.total_permutations > 0 {
            (stats.nodes_pruned as f64 / (stats.nodes_explored + stats.nodes_pruned) as f64) * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!(
        "  Время выполнения:        {} мс\n",
        stats.time_ms
    ));
    output.push_str("\n");

    let optimal = if stats.nodes_explored < stats.total_permutations as usize {
        "Гарантированно оптимальное решение (все перспективные варианты исследованы)"
    } else {
        "Решение может быть неоптимальным (достигнуты лимиты поиска)"
    };
    output.push_str(&format!("{}\n\n", optimal));

    output.push_str("Оптимальная последовательность:\n");
    let seq_str: String = result
        .sequence
        .iter()
        .map(|&idx| format!("J{}", idx + 1))
        .collect::<Vec<String>>()
        .join(" → ");
    output.push_str(&format!("  {}\n", seq_str));
    output.push_str("\n");

    let num_machines = matrix[0].len();
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
        "\nДлительность производственного цикла: {}\n",
        result.makespan
    ));
    output.push_str("Простои станков:\n");
    for (machine, &idle) in result.idle_times.iter().enumerate() {
        output.push_str(&format!("  M{}: {}\n", machine + 1, idle));
    }
    output
}

#[derive(Debug, Clone)]
struct PartialSchedule {
    sequence: Vec<usize>,
    remaining_mask: u64,
    schedule: Vec<Vec<(i32, i32)>>,
    lower_bound: i32,
    current_makespan: i32,
}

impl PartialSchedule {
    fn new(num_jobs: usize) -> Self {
        PartialSchedule {
            sequence: Vec::new(),
            remaining_mask: (1u64 << num_jobs) - 1,
            schedule: Vec::new(),
            lower_bound: 0,
            current_makespan: 0,
        }
    }

    fn add_job(&mut self, job_idx: usize, matrix: &Vec<Vec<i32>>) {
        let num_machines = matrix[0].len();
        let seq_idx = self.sequence.len();

        self.sequence.push(job_idx);
        self.remaining_mask &= !(1u64 << job_idx);

        let mut new_row = vec![(0, 0); num_machines];

        let prev_out_m1 = if seq_idx > 0 {
            self.schedule[seq_idx - 1][0].1
        } else {
            0
        };
        let proc_time_m1 = matrix[job_idx][0];
        new_row[0] = (prev_out_m1, prev_out_m1 + proc_time_m1);

        for machine in 1..num_machines {
            let out_prev_machine = new_row[machine - 1].1;
            let out_prev_job = if seq_idx > 0 {
                self.schedule[seq_idx - 1][machine].1
            } else {
                0
            };
            let in_time = out_prev_machine.max(out_prev_job);
            let proc_time = matrix[job_idx][machine];
            new_row[machine] = (in_time, in_time + proc_time);
        }

        self.current_makespan = new_row[num_machines - 1].1;
        self.schedule.push(new_row);
    }

    fn remaining_jobs(&self, num_jobs: usize) -> Vec<usize> {
        (0..num_jobs)
            .filter(|&idx| (self.remaining_mask & (1u64 << idx)) != 0)
            .collect()
    }

    fn calculate_lower_bound(&self, matrix: &Vec<Vec<i32>>, num_jobs: usize) -> i32 {
        let num_machines = matrix[0].len();
        let remaining = self.remaining_jobs(num_jobs);

        if remaining.is_empty() {
            return self.current_makespan;
        }

        let mut max_lb = 0;

        for machine in 0..num_machines {
            let current_completion = if let Some(last_job) = self.schedule.last() {
                last_job[machine].1
            } else {
                0
            };

            let sum_remaining: i32 = remaining.iter().map(|&j| matrix[j][machine]).sum();

            let lb_for_machine = current_completion + sum_remaining;
            if lb_for_machine > max_lb {
                max_lb = lb_for_machine;
            }
        }

        max_lb
    }

    fn is_complete(&self, num_jobs: usize) -> bool {
        self.sequence.len() == num_jobs
    }
}

#[derive(Debug, Clone)]
struct SearchNode {
    schedule: PartialSchedule,
    priority: i32,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for SearchNode {}
