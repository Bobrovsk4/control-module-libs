use crate::common::AlgResult;
use plotters::prelude::*;

pub fn draw_gantt(result: &AlgResult, matrix: &Vec<Vec<i32>>, filename: &str) {
    let machines = matrix[0].len();
    let jobs = result.schedule.len();
    let root = SVGBackend::new(filename, (1400, 400)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let (chart_area, legend_area) = root.split_horizontally(1200);

    let mut chart = ChartBuilder::on(&chart_area)
        .caption(result.method_name.as_str(), ("sans-serif", 20))
        .set_label_area_size(LabelAreaPosition::Left, 50)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0..result.makespan, 0.0..(machines as f64))
        .unwrap();

    chart
        .configure_mesh()
        .y_labels((machines as u32).try_into().unwrap())
        .y_label_formatter(&|v| format!("M{}", machines - *v as usize))
        .draw()
        .unwrap();

    let height = 0.6;
    let pad = (1.0 - height) / 2.0;
    let mut legend_items: Vec<(RGBAColor, String)> = Vec::new();

    for seq_idx in 0..jobs {
        let color: RGBAColor = Palette99::pick(seq_idx).to_rgba();
        legend_items.push((color, format!("Job {}", seq_idx)));

        let rects = (0..machines).filter_map(move |m_idx| {
            let (start, end) = result.schedule[seq_idx][m_idx];
            let y_base = (machines - 1 - m_idx) as f64;
            if end > start {
                Some(Rectangle::new(
                    [(start, y_base + pad), (end, y_base + 1.0 - pad)],
                    color.filled(),
                ))
            } else {
                None
            }
        });

        chart.draw_series(rects).unwrap();
    }

    let mut y_offset = 20;
    for (color, label) in legend_items {
        let y = y_offset;
        legend_area
            .draw(&Rectangle::new([(10, y - 5), (20, y + 5)], color.filled()))
            .unwrap();
        legend_area
            .draw(&Text::new(label, (25, y + 5), ("sans-serif", 18)))
            .unwrap();
        y_offset += 20;
    }

    root.present().unwrap();
}
