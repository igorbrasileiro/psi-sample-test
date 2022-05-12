use csv::Writer;
use futures::io::Write;
use std::error::Error;
use std::fs::File;

use crate::utils::check_file_availability;
use crate::PSIStatisticResult;

fn print_table_result(
    page_mean: &PSIStatisticResult<f64>,
    page_std_deviation: &PSIStatisticResult<f64>,
    page_confidence_interval: &PSIStatisticResult<(f64, f64)>,
) {
    println!("| Metric | Mean | Standard deviation | Confidence Interval (95%) |");
    println!("|--------|--------|--------|--------|");

    println!(
        "| Cumulative Layout shift (CLS) | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.cumulative_layout_shift,
        std_deviation = page_std_deviation.cumulative_layout_shift,
        ci_min = page_confidence_interval.cumulative_layout_shift.0,
        ci_max = page_confidence_interval.cumulative_layout_shift.1,
    );
    println!(
        "| First Contentful Paint (FCP) | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.first_contentful_paint,
        std_deviation = page_std_deviation.first_contentful_paint,
        ci_min = page_confidence_interval.first_contentful_paint.0,
        ci_max = page_confidence_interval.first_contentful_paint.1,
    );
    println!(
        "| Largest Contentful Paint (LCP) | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.largest_contentful_paint,
        std_deviation = page_std_deviation.largest_contentful_paint,

        ci_min = page_confidence_interval.largest_contentful_paint.0,
        ci_max = page_confidence_interval.largest_contentful_paint.1,
    );
    println!(
        "| Time to Interactive (TTI) | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.time_to_interactive,
        std_deviation = page_std_deviation.time_to_interactive,

        ci_min = page_confidence_interval.time_to_interactive.0,
        ci_max = page_confidence_interval.time_to_interactive.1,
    );
    println!(
        "| Total Blocking Time (TBT) | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.total_blocking_time,
        std_deviation = page_std_deviation.total_blocking_time,

        ci_min = page_confidence_interval.total_blocking_time.0,
        ci_max = page_confidence_interval.total_blocking_time.1,
    );
    println!(
        "| Performance score | {mean:.3} | {std_deviation:.6} | [{ci_min:.6}, {ci_max:.6}] |",
        mean = page_mean.score,
        std_deviation = page_std_deviation.score,
        ci_min = page_confidence_interval.score.0,
        ci_max = page_confidence_interval.score.1,
    );
    println!(
        "| JavaScript Execution Time | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.js_execution_time,
        std_deviation = page_std_deviation.js_execution_time,

        ci_min = page_confidence_interval.js_execution_time.0,
        ci_max = page_confidence_interval.js_execution_time.1,
    );
    println!(
        "| Speed Index | {mean:.2} | {std_deviation:.2} | [{ci_min:.2}, {ci_max:.2}] |",
        mean = page_mean.speed_index,
        std_deviation = page_std_deviation.speed_index,
        ci_min = page_confidence_interval.speed_index.0,
        ci_max = page_confidence_interval.speed_index.1,
    );
}

pub fn print_result(
    page_url: &str,
    page_mean: &PSIStatisticResult<f64>,
    page_std_deviation: &PSIStatisticResult<f64>,
    page_confidence_interval: &PSIStatisticResult<(f64, f64)>,
) {
    println!("Page result - {url}", url = page_url);
    print_table_result(page_mean, page_std_deviation, page_confidence_interval);
}

// pub fn write_csv_file_line(
// csv_writer: &Writer<File>,
// url: &str,
// d_mean: f64,
// d_median: f64,
// m_mean: f64,
// m_median: f64,
// ) -> Result<(), Box<dyn Error>> {
// println!("Store,Desktop - Media,Desktop - Mediana,Mobile - Media,Mobile - Mediana");

// csv_writer.write_record(&[url, d_mean, d_median, m_mean, m_median]);

// return Ok(());
// }

// pub fn create_csv_file() -> Write<File> {
// let mut wtr = Writer::from_path(check_file_availability("./output.csv"))?;
// wtr.write_record([
// "Store",
// "Desktop - Mean",
// "Desktop - Median",
// "Mobile - Mean",
// "Mobile - Median",
// ]);
// return wtr;
// }
