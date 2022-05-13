use csv::Writer;
use serde::Serialize;
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

#[derive(Serialize)]
pub struct Row<'a> {
    #[serde(rename = "Store")]
    pub url: &'a str,

    #[serde(rename = "Desktop - Mean")]
    pub d_mean: f64,

    #[serde(rename = "Desktop - Median")]
    pub d_median: f64,

    #[serde(rename = "Mobile - Mean")]
    pub m_mean: f64,

    #[serde(rename = "Mobile - Median")]
    pub m_median: f64,
}

pub struct CSVPrinter {
    csv_writer: Writer<File>,
}

impl CSVPrinter {
    pub fn new() -> CSVPrinter {
        CSVPrinter {
            csv_writer: Writer::from_path(check_file_availability("./output.csv")).unwrap(),
        }
    }

    pub fn write_line(&mut self, row: Row) -> Result<(), Box<dyn Error>> {
        self.csv_writer.serialize(row)?;

        return Ok(());
    }

    pub fn flush(&mut self) {
        self.csv_writer.flush();
    }
}
