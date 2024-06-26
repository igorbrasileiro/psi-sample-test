use clap::{Arg, Command};
use reqwest::Error;
use serde::Deserialize;

mod printer;
mod statistics;
mod tester;
mod utils;

const SAMPLE: i8 = 20;

#[derive(Debug, Deserialize)]
pub enum Strategy {
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
pub struct LHResult {
    audits: Audits,
    categories: Categories,
}

#[derive(Deserialize, Debug)]
struct PSIResult {
    #[serde(rename = "lighthouseResult")]
    lighthouse_result: LHResult,
}

pub struct PSIResultValues {
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
pub struct PSIStatisticResult<T> {
    cumulative_layout_shift: T,
    first_contentful_paint: T,
    js_execution_time: T,
    largest_contentful_paint: T,
    speed_index: T,
    time_to_interactive: T,
    total_blocking_time: T,
    score: T,
}

async fn batch_tests(
    url: &str,
    token: &str,
    number_of_runs: i8,
    printer: &mut printer::CSVPrinter,
) -> bool {
    let mobile_page_result = tester::get_page_audits(url, token, number_of_runs, Strategy::MOBILE)
        .await
        .unwrap();
    // Handle if some test failed
    if mobile_page_result.score.len() != number_of_runs as usize {
        return false;
    }

    let desktop_page_result =
        tester::get_page_audits(url, token, number_of_runs, Strategy::DESKTOP)
            .await
            .unwrap();
    // Handle if some test failed
    if desktop_page_result.score.len() != number_of_runs as usize {
        return false;
    }

    let mobile_page_mean = statistics::calculate_mean(&mobile_page_result, number_of_runs);
    let mobile_page_median = statistics::median(&mobile_page_result.score);

    let desktop_page_mean = statistics::calculate_mean(&desktop_page_result, number_of_runs);
    let desktop_page_median = statistics::median(&desktop_page_result.score);

    let _x = printer.write_line(printer::Row {
        url,
        d_mean: desktop_page_mean.score,
        d_median: desktop_page_median,
        m_mean: mobile_page_mean.score,
        m_median: mobile_page_median,
    });

    let _x = printer.flush();

    true
}

async fn run_batch_tests(filename: &str, token: &str, number_of_runs: i8) -> bool {
    let urls = utils::read_lines(filename);
    let mut failed_urls: Vec<String> = Vec::new();

    let mut csv_printer = printer::CSVPrinter::new();

    for url in urls.map_while(Result::ok) {
        println!("Testing {url}", url = url);

        let test_finished = batch_tests(&url, token, number_of_runs, &mut csv_printer).await;

        if !test_finished {
            failed_urls.push(url.clone());
        }
    }

    // Handle failed urls until failed_urls list is empty
    for qtt in 0..2 {
        let urls_size = failed_urls.len();

        for url_idx in 0..urls_size {
            // from last to first
            let idx = (urls_size - 1) - url_idx;
            let url = failed_urls[idx].clone();

            println!("Retesting {url} {qtt}x", url = url, qtt = qtt);

            let test_finished = batch_tests(&url, token, number_of_runs, &mut csv_printer).await;

            if !test_finished {
                continue;
            }

            failed_urls.remove(idx);
        }
    }

    for url in failed_urls {
        println!("Test failed for {url} after two retries", url = url);
    }

    let _x = csv_printer.flush();

    true
}

struct TestResult {
    page_mean: PSIStatisticResult<f64>,
    page_deviation: PSIStatisticResult<f64>,
    page_confidence_interval: PSIStatisticResult<(f64, f64)>,
    success_runs: i8,
}
async fn run_single_tests(
    page_url: &str,
    token: &str,
    number_of_runs: i8,
    strategy: Strategy,
) -> TestResult {
    let page_result = &tester::get_page_audits(page_url, token, number_of_runs, strategy)
        .await
        .unwrap();

    let _nruns = page_result.score.len() as i8;

    let page_mean = statistics::calculate_mean(page_result, _nruns);

    let page_deviation = statistics::calculate_deviation(page_result, &page_mean, _nruns);

    let page_confidence_interval =
        statistics::calculate_confidence_interval(&page_mean, &page_deviation, _nruns);

    TestResult {
        page_mean,
        page_deviation,
        page_confidence_interval,
        success_runs: _nruns,
    }
}

async fn psi_test() -> Result<(), Error> {
    let matches = Command::new("psi-tests")
        .about("PSI Tests is a tool to run multiple page speed insight tests.")
        .long_about(
        "PSI Tests is a tool to run multiple page speed insight tests and get the mean and standard deviation from some metrics.
        Example: run 10 tests from a specific url
        psi-test --token=<TOKEN_VALUE> --number-of-runs=10 https://www.google.com

        Example: run 5 tests for multiples urls
        psi-test --token=<TOKEN_VALUE> --number_of_runs=5 -B ./input.txt",
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
        .arg(
            // https://developers.google.com/speed/docs/insights/v5/reference/pagespeedapi/runpagespeed#response
            Arg::new("strategy")
            .value_name("STRATEGY")
            .short('S')
            .long("strategy")
            .help("The analysis strategy (desktop or mobile) to use, and mobile is the default.

Acceptable values are:
    \"desktop\": Fetch and analyze the URL for desktop browsers
    \"mobile\": Fetch and analyze the URL for mobile devices

This value isn't used when batch_tests flag is present."
            )
        )
        .arg(
            Arg::new("output-format")
            .value_name("OUTPUT_FORMAT")
            .short('F')
            .long("output-format")
            .help("output-format can be: md for markdown, json for json. --output-format: md|json.")
        )
        .get_matches();

    // Required value
    let token = matches.value_of("token").expect("Token is required!");
    let number_of_runs = match matches.value_of("number-of-runs") {
        Some(value) => value.parse::<i8>().unwrap(),
        None => SAMPLE,
    };

    // Run batch tests
    if let Some(batch) = matches.value_of("batch") {
        run_batch_tests(batch, token, number_of_runs).await;

        return Ok(());
    }

    // Required value
    let page_url = matches
        .value_of("first-page")
        .expect("Page URL is required");

    let strategy = match matches.value_of("strategy") {
        Some(value) => {
            if value.parse::<String>().unwrap().eq("desktop") {
                Strategy::DESKTOP
            } else {
                Strategy::MOBILE
            }
        }
        None => Strategy::MOBILE,
    };

    let output_format = matches.value_of("output-format").unwrap_or("json");

    let test_result = run_single_tests(page_url, token, number_of_runs, strategy).await;

    if output_format == "md" {
        printer::print_md(
            page_url,
            test_result.success_runs,
            &test_result.page_mean,
            &test_result.page_deviation,
            &test_result.page_confidence_interval,
        );
    } else if output_format == "json" {
        printer::print_json(
            page_url,
            test_result.success_runs,
            &test_result.page_mean,
            &test_result.page_deviation,
            &test_result.page_confidence_interval,
        )
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    psi_test().await?;

    Ok(())
}
