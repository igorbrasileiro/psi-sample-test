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

async fn batch_tests(url: &str, token: &str, number_of_runs: i8) -> bool {
    let mobile_page_result =
        tester::get_page_audits(&url as &str, token, number_of_runs, Strategy::MOBILE)
            .await
            .unwrap();
    // Handle if some test failed
    for mobile_result in &mobile_page_result.score {
        if *mobile_result == 0_f64 {
            return false;
        }
    }

    let desktop_page_result =
        tester::get_page_audits(&url as &str, token, number_of_runs, Strategy::DESKTOP)
            .await
            .unwrap();
    // Handle if some test failed
    for desktop_result in &desktop_page_result.score {
        if *desktop_result == 0_f64 {
            return false;
        }
    }

    let mobile_page_mean = statistics::calculate_mean(&mobile_page_result, number_of_runs);
    let mobile_page_median = statistics::median(&mobile_page_result.score);

    let desktop_page_mean = statistics::calculate_mean(&desktop_page_result, number_of_runs);
    let desktop_page_median = statistics::median(&desktop_page_result.score);

    println!(
        "{url},{d_mean:.3},{d_median:.3},{m_mean:.3},{m_median:.3}",
        url = url,
        d_mean = desktop_page_mean.score,
        d_median = desktop_page_median,
        m_mean = mobile_page_mean.score,
        m_median = mobile_page_median,
    );

    return true;
}

async fn run_batch_tests(filename: &str, token: &str, number_of_runs: i8) {
    let urls = utils::read_lines(filename);
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
    for _ in 0..2 {
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

    for url in failed_urls {
        println!(
            "{url},{d_mean:.3},{d_median:.3},{m_mean:.3},{m_median:.3}",
            url = url,
            d_mean = 0_f64,
            d_median = 0_f64,
            m_mean = 0_f64,
            m_median = 0_f64,
        );
    }
}

async fn run_single_tests(page_url: &str, token: &str, number_of_runs: i8) {
    let page_result = &tester::get_page_audits(page_url, token, number_of_runs, Strategy::MOBILE)
        .await
        .unwrap();

    let page_mean = statistics::calculate_mean(&page_result, number_of_runs);

    let page_deviation = statistics::calculate_deviation(&page_result, &page_mean, number_of_runs);

    let page_confidence_interval =
        statistics::calculate_confidence_interval(&page_mean, &page_deviation, number_of_runs);

    printer::print_result(
        page_url,
        &page_mean,
        &page_deviation,
        &page_confidence_interval,
    );
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

    // TODO: Filter get_page_audits results that's empty when failed.
    run_single_tests(page_url, token, number_of_runs).await;

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    psi_test().await?;

    return Ok(());
}
