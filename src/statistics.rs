use crate::{PSIResultValues, PSIStatisticResult};

const Z_VALUE: f64 = 1.96_f64; // z-value for 95% confidence level.

pub fn mean(results: &[f64], number_of_runs: i8) -> f64 {
    return results.iter().sum::<f64>() / number_of_runs as f64;
}

pub fn calculate_mean(
    page_results: &PSIResultValues,
    number_of_runs: i8,
) -> PSIStatisticResult<f64> {
    PSIStatisticResult {
        cumulative_layout_shift: mean(&page_results.cumulative_layout_shift, number_of_runs),
        first_contentful_paint: mean(&page_results.first_contentful_paint, number_of_runs),
        js_execution_time: mean(&page_results.js_execution_time, number_of_runs),
        largest_contentful_paint: mean(&page_results.largest_contentful_paint, number_of_runs),
        speed_index: mean(&page_results.speed_index, number_of_runs),
        time_to_interactive: mean(&page_results.time_to_interactive, number_of_runs),
        total_blocking_time: mean(&page_results.total_blocking_time, number_of_runs),
        score: mean(&page_results.score, number_of_runs),
    }
}

pub fn std_deviation(data: &[f64], mean: f64, number_of_runs: i8) -> f64 {
    return data
        .iter()
        .map(|value| {
            let diff = mean - value;

            diff * diff
        })
        .sum::<f64>()
        / number_of_runs as f64;
}

pub fn calculate_deviation(
    page_results: &PSIResultValues,
    page_mean: &PSIStatisticResult<f64>,
    number_of_runs: i8,
) -> PSIStatisticResult<f64> {
    PSIStatisticResult {
        cumulative_layout_shift: std_deviation(
            &page_results.cumulative_layout_shift,
            page_mean.cumulative_layout_shift,
            number_of_runs,
        ),
        first_contentful_paint: std_deviation(
            &page_results.first_contentful_paint,
            page_mean.first_contentful_paint,
            number_of_runs,
        ),
        js_execution_time: std_deviation(
            &page_results.js_execution_time,
            page_mean.js_execution_time,
            number_of_runs,
        ),
        largest_contentful_paint: std_deviation(
            &page_results.largest_contentful_paint,
            page_mean.largest_contentful_paint,
            number_of_runs,
        ),
        speed_index: std_deviation(
            &page_results.speed_index,
            page_mean.speed_index,
            number_of_runs,
        ),
        time_to_interactive: std_deviation(
            &page_results.time_to_interactive,
            page_mean.time_to_interactive,
            number_of_runs,
        ),
        total_blocking_time: std_deviation(
            &page_results.total_blocking_time,
            page_mean.total_blocking_time,
            number_of_runs,
        ),
        score: std_deviation(&page_results.score, page_mean.score, number_of_runs),
    }
}

// Reference: https://www.dummies.com/article/academics-the-arts/math/statistics/how-to-calculate-a-confidence-interval-for-a-population-mean-when-you-know-its-standard-deviation-169722/
pub fn confidence_interval(mean: f64, std_deviation: f64, number_of_runs: i8) -> (f64, f64) {
    // margin error =  z value * std_deviation / sqrt (number_of_runs)
    let margin_error = Z_VALUE * (std_deviation / (number_of_runs as f64).sqrt());

    (mean - margin_error, mean + margin_error)
}

pub fn calculate_confidence_interval(
    mean: &PSIStatisticResult<f64>,
    std_deviation: &PSIStatisticResult<f64>,
    number_of_runs: i8,
) -> PSIStatisticResult<(f64, f64)> {
    PSIStatisticResult::<(f64, f64)> {
        cumulative_layout_shift: confidence_interval(
            mean.cumulative_layout_shift,
            std_deviation.cumulative_layout_shift,
            number_of_runs,
        ),
        first_contentful_paint: confidence_interval(
            mean.first_contentful_paint,
            std_deviation.first_contentful_paint,
            number_of_runs,
        ),
        js_execution_time: confidence_interval(
            mean.js_execution_time,
            std_deviation.js_execution_time,
            number_of_runs,
        ),
        largest_contentful_paint: confidence_interval(
            mean.largest_contentful_paint,
            std_deviation.largest_contentful_paint,
            number_of_runs,
        ),
        speed_index: confidence_interval(
            mean.speed_index,
            std_deviation.speed_index,
            number_of_runs,
        ),
        time_to_interactive: confidence_interval(
            mean.time_to_interactive,
            std_deviation.time_to_interactive,
            number_of_runs,
        ),
        total_blocking_time: confidence_interval(
            mean.total_blocking_time,
            std_deviation.total_blocking_time,
            number_of_runs,
        ),
        score: confidence_interval(mean.score, std_deviation.score, number_of_runs),
    }
}

pub fn median(list: &[f64]) -> f64 {
    let number_of_runs: usize = list.len();
    let index = number_of_runs / 2;

    // Sort list to get the middle value
    let mut sorted_list = list.to_owned();
    sorted_list.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if number_of_runs % 2 == 1 {
        // odd
        *sorted_list.get(index).unwrap()
    } else {
        // even
        let first_median = sorted_list.get(index).unwrap();
        let second_median = sorted_list.get(index + 1).unwrap();

        (first_median + second_median) / 2_f64
    }
}

// pub fn calculate_median(page_results: &PSIResultValues) -> PSIStatisticResult<f64> {
// return PSIStatisticResult {
// cumulative_layout_shift: median(&page_results.cumulative_layout_shift),
// first_contentful_paint: median(&page_results.first_contentful_paint),
// js_execution_time: median(&page_results.js_execution_time),
// largest_contentful_paint: median(&page_results.largest_contentful_paint),
// speed_index: median(&page_results.speed_index),
// time_to_interactive: median(&page_results.time_to_interactive),
// total_blocking_time: median(&page_results.total_blocking_time),
// score: median(&page_results.score),
// };
// }
