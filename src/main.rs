use std::usize;

use clap::{Arg, Command};
use futures::StreamExt;
use reqwest::Error;
use serde::Deserialize;

mod utils;
use crate::utils::read_lines;

const SAMPLE: i8 = 20;
const BUFFER_SIZE: usize = 15;
const Z_VALUE: f64 = 1.96_f64; // z-value for 95% confidence level.
const EMPTY_AUDIT: Audit = Audit {
    numeric_value: 0_f64,
};
const EMPTY_LH_RESULT: LHResult = LHResult {
    audits: Audits {
        cumulative_layout_shift: EMPTY_AUDIT,
        first_contentful_paint: EMPTY_AUDIT,
        js_execution_time: EMPTY_AUDIT,
        largest_contentful_paint: EMPTY_AUDIT,
        speed_index: EMPTY_AUDIT,
        time_to_interactive: EMPTY_AUDIT,
        total_blocking_time: EMPTY_AUDIT,
    },
    categories: Categories {
        performance: Category { score: 0_f64 },
    },
};

#[derive(Debug, Deserialize)]
enum Strategy {
    MOBILE,
    DESKTOP,
}

impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Strategy::MOBILE => write!(f, "mobile"),
            Strategy::DESKTOP => write!(f, "desktop"),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Audit {
    #[serde(rename = "numericValue")]
    numeric_value: f64,
}

#[derive(Deserialize, Debug)]
struct Audits {
    #[serde(rename = "cumulative-layout-shift")]
    cumulative_layout_shift: Audit,

    #[serde(rename = "first-contentful-paint")]
    first_contentful_paint: Audit,

    #[serde(rename = "bootup-time")]
    js_execution_time: Audit,

    #[serde(rename = "largest-contentful-paint")]
    largest_contentful_paint: Audit,

    #[serde(rename = "speed-index")]
    speed_index: Audit,

    #[serde(rename = "interactive")]
    time_to_interactive: Audit,

    #[serde(rename = "total-blocking-time")]
    total_blocking_time: Audit,
}

#[derive(Deserialize, Debug)]
struct Category {
    score: f64,
}

#[derive(Deserialize, Debug)]
struct Categories {
    performance: Category,
}

#[derive(Deserialize, Debug)]
struct LHResult {
    audits: Audits,
    categories: Categories,
}

#[derive(Deserialize, Debug)]
struct PSIResult {
    #[serde(rename = "lighthouseResult")]
    lighthouse_result: LHResult,
}

struct PSIResultValues {
    cumulative_layout_shift: Vec<f64>,
    first_contentful_paint: Vec<f64>,
    js_execution_time: Vec<f64>,
    largest_contentful_paint: Vec<f64>,
    speed_index: Vec<f64>,
    time_to_interactive: Vec<f64>,
    total_blocking_time: Vec<f64>,
    score: Vec<f64>,
}

#[derive(Debug)]
struct PSIStatisticResult<T> {
    cumulative_layout_shift: T,
    first_contentful_paint: T,
    js_execution_time: T,
    largest_contentful_paint: T,
    speed_index: T,
    time_to_interactive: T,
    total_blocking_time: T,
    score: T,
}

async fn get_page_audits(
    url: &str,
    token: &str,
    number_of_runs: i8,
    strategy: Strategy,
) -> Result<Vec<LHResult>, reqwest::Error> {
    let list_urls = (0..number_of_runs).map(|_| {
        format!("https://www.googleapis.com/pagespeedonline/v5/runPagespeed?key={api_key}&url={url}&strategy={strategy}&category=performance", url = url, api_key = token, strategy = strategy)
    }).collect::<Vec<String>>();
    let client = reqwest::Client::new();

    let list_responses = futures::stream::iter(
        list_urls
            .iter()
            .map(|url| client.get(url).send())
            .into_iter(),
    )
    .buffer_unordered(BUFFER_SIZE)
    .collect::<Vec<_>>()
    .await;

    let mut list_audits = Vec::new();
    for res in list_responses {
        let audit = match res {
            Ok(result) => match result.json::<PSIResult>().await {
                Ok(json) => LHResult {
                    audits: json.lighthouse_result.audits,
                    categories: json.lighthouse_result.categories,
                },
                Err(_) => EMPTY_LH_RESULT,
            },
            Err(error) => panic!(
                "Problem mounting audits {site}. \n {error}",
                site = url,
                error = error
            ),
        };
        list_audits.push(audit);
    }

    return Ok(list_audits);
}

fn map_audits(lh_results: &Vec<LHResult>) -> PSIResultValues {
    return PSIResultValues {
        cumulative_layout_shift: lh_results
            .iter()
            .map(|result| result.audits.cumulative_layout_shift.numeric_value)
            .collect(),
        first_contentful_paint: lh_results
            .iter()
            .map(|result| result.audits.first_contentful_paint.numeric_value)
            .collect(),
        js_execution_time: lh_results
            .iter()
            .map(|result| result.audits.js_execution_time.numeric_value)
            .collect(),
        largest_contentful_paint: lh_results
            .iter()
            .map(|result| result.audits.largest_contentful_paint.numeric_value)
            .collect(),
        speed_index: lh_results
            .iter()
            .map(|result| result.audits.speed_index.numeric_value)
            .collect(),
        time_to_interactive: lh_results
            .iter()
            .map(|result| result.audits.time_to_interactive.numeric_value)
            .collect(),
        total_blocking_time: lh_results
            .iter()
            .map(|result| result.audits.total_blocking_time.numeric_value)
            .collect(),
        score: lh_results
            .iter()
            .map(|result| result.categories.performance.score)
            .collect(),
    };
}

fn mean(results: &Vec<f64>, number_of_runs: i8) -> f64 {
    return results.iter().sum::<f64>() / number_of_runs as f64;
}

fn calculate_mean(page_results: &PSIResultValues, number_of_runs: i8) -> PSIStatisticResult<f64> {
    return PSIStatisticResult {
        cumulative_layout_shift: mean(&page_results.cumulative_layout_shift, number_of_runs),
        first_contentful_paint: mean(&page_results.first_contentful_paint, number_of_runs),
        js_execution_time: mean(&page_results.js_execution_time, number_of_runs),
        largest_contentful_paint: mean(&page_results.largest_contentful_paint, number_of_runs),
        speed_index: mean(&page_results.speed_index, number_of_runs),
        time_to_interactive: mean(&page_results.time_to_interactive, number_of_runs),
        total_blocking_time: mean(&page_results.total_blocking_time, number_of_runs),
        score: mean(&page_results.score, number_of_runs),
    };
}

fn std_deviation(data: &Vec<f64>, mean: f64, number_of_runs: i8) -> f64 {
    return data
        .iter()
        .map(|value| {
            let diff = mean - value;

            diff * diff
        })
        .sum::<f64>()
        / number_of_runs as f64;
}

fn calculate_deviation(
    page_results: &PSIResultValues,
    page_mean: &PSIStatisticResult<f64>,
    number_of_runs: i8,
) -> PSIStatisticResult<f64> {
    return PSIStatisticResult {
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
    };
}

// Reference: https://www.dummies.com/article/academics-the-arts/math/statistics/how-to-calculate-a-confidence-interval-for-a-population-mean-when-you-know-its-standard-deviation-169722/
fn confidence_interval(mean: f64, std_deviation: f64, number_of_runs: i8) -> (f64, f64) {
    // margin error =  z value * std_deviation / sqrt (number_of_runs)
    let margin_error = Z_VALUE * (std_deviation / (number_of_runs as f64).sqrt());

    return (mean - margin_error, mean + margin_error);
}

fn calculate_confidence_interval(
    mean: &PSIStatisticResult<f64>,
    std_deviation: &PSIStatisticResult<f64>,
    number_of_runs: i8,
) -> PSIStatisticResult<(f64, f64)> {
    return PSIStatisticResult::<(f64, f64)> {
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
    };
}

fn median(list: &Vec<f64>) -> f64 {
    let number_of_runs: usize = list.len();
    let index = number_of_runs / 2;

    // Sort list to get the middle value
    let mut sorted_list = list.clone();
    sorted_list.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if number_of_runs % 2 == 1 {
        // odd
        return sorted_list.get(index).unwrap().clone();
    } else {
        // even
        let first_median = sorted_list.get(index).unwrap();
        let second_median = sorted_list.get(index + 1).unwrap();

        return (first_median + second_median) / 2_f64;
    }
}

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

fn print_result(
    page_url: &str,
    page_mean: &PSIStatisticResult<f64>,
    page_std_deviation: &PSIStatisticResult<f64>,
    page_confidence_interval: &PSIStatisticResult<(f64, f64)>,
) {
    println!("Page result - {url}", url = page_url);
    print_table_result(page_mean, page_std_deviation, page_confidence_interval);
}

async fn batch_tests(url: &str, token: &str, number_of_runs: i8) -> bool {
    let mobile_page_result = map_audits(
        &get_page_audits(&url as &str, token, number_of_runs, Strategy::MOBILE)
            .await
            .unwrap(),
    );
    // Handle if some test failed
    for mobile_result in &mobile_page_result.score {
        if *mobile_result == 0_f64 {
            return false;
        }
    }

    let desktop_page_result = map_audits(
        &get_page_audits(&url as &str, token, number_of_runs, Strategy::DESKTOP)
            .await
            .unwrap(),
    );
    // Handle if some test failed
    for desktop_result in &desktop_page_result.score {
        if *desktop_result == 0_f64 {
            return false;
        }
    }

    let mobile_page_mean = calculate_mean(&mobile_page_result, number_of_runs);
    let mobile_page_median = median(&mobile_page_result.score);

    let desktop_page_mean = calculate_mean(&desktop_page_result, number_of_runs);
    let desktop_page_median = median(&desktop_page_result.score);

    println!(
        "{url},{d_mean:.3},{d_median:.3},{m_mean:.3},{m_median:.3}",
        url = url,
        d_mean = desktop_page_mean.score,
        d_median = desktop_page_median,
        m_mean = mobile_page_mean.score,
        m_median = mobile_page_median
    );

    return true;
}

async fn run_batch_tests(filename: &str, token: &str, number_of_runs: i8) {
    let urls = read_lines(filename);
    let mut failed_urls: Vec<String> = Vec::new();

    println!("Store,Desktop - Media,Desktop - Mediana,Mobile - Media,Mobile - Mediana");
    for _url in urls {
        if let Ok(url) = _url {
            let url = url;
            let test_finished = batch_tests(&url, token, number_of_runs).await;

            if !test_finished {
                failed_urls.push(url.clone());
            }
        }
    }

    // Handle failed urls until failed_urls list is empty
    while failed_urls.len() > 0 {
        let urls_size = failed_urls.len();

        for url_idx in 0..urls_size {
            // from last to first
            let idx = (urls_size - 1) - url_idx;
            let url = failed_urls[idx].clone();
            let test_finished = batch_tests(&url, token, number_of_runs).await;

            if !test_finished {
                continue;
            }

            failed_urls.remove(idx);
        }
    }
}

async fn psi_test() -> Result<(), Error> {
    let matches = Command::new("psi-tests")
        .about("PSI Tests is a tool to run multiple page speed insight tests.")
        .long_about(
        "PSI Tests is a tool to run multiple page speed insight tests and get the mean and standard deviation from some metrics.
        Example: you wanna run 10 tests from a specific site
        psi-test --token=<TOKEN_VALUE> --number-of-runs=10 https://www.google.com",
        )
        // Change if crate_version start work again
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("token")
            .value_name("TOKEN_VALUE")
            .required(true)
            .short('T')
            .long("token")
            .help("Google cloud token to access Page Speed Insights API. For more informartion: https://developers.google.com/speed/docs/insights/v5/get-started#APIKey"),
        )
        .arg(
            Arg::new("number-of-runs")
            .value_name("NUMBER")
            .short('N')
            .long("number-of-runs")
            .help("Number of PSI tests for each page."),
        )
        .arg(
            Arg::new("first-page")
            .help("Page URL.")
            .index(1)
        )
        .arg(
            Arg::new("batch")
            .value_name("INPUT")
            .short('B')
            .long("batch-file")
            .help("Batch file allow pass a TXT input file with URLs, line by line, to be tested.")
        )
        .get_matches();

    // Required value
    let token = matches.value_of("token").expect("Token is required!");
    let number_of_runs = match matches.value_of("number-of-runs") {
        Some(value) => value.parse::<i8>().unwrap(),
        None => SAMPLE,
    };

    if let Some(batch) = matches.value_of("batch") {
        run_batch_tests(batch, token, number_of_runs).await;

        return Ok(());
    }

    // Required value
    let page_url = matches
        .value_of("first-page")
        .expect("Page URL is required");

    // TODO: Filter get_page_audits results that's empty when failed.
    let page_result =
        map_audits(&get_page_audits(page_url, token, number_of_runs, Strategy::MOBILE).await?);

    let page_mean = calculate_mean(&page_result, number_of_runs);

    let page_deviation = calculate_deviation(&page_result, &page_mean, number_of_runs);

    let page_confidence_interval =
        calculate_confidence_interval(&page_mean, &page_deviation, number_of_runs);

    print_result(
        page_url,
        &page_mean,
        &page_deviation,
        &page_confidence_interval,
    );

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    psi_test().await?;

    return Ok(());
}
