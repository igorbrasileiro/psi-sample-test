use clap::{Arg, Command};
use reqwest::Error;
use serde::Deserialize;

const SAMPLE: i8 = 20;

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

    #[serde(rename = "first-contentful-paint-3g")]
    first_contentful_paint_3g: Audit,

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
    first_contentful_paint_3g: Vec<f64>,
    js_execution_time: Vec<f64>,
    largest_contentful_paint: Vec<f64>,
    speed_index: Vec<f64>,
    time_to_interactive: Vec<f64>,
    total_blocking_time: Vec<f64>,
    score: Vec<f64>,
}

#[derive(Debug)]
struct PSIStatisticResult {
    cumulative_layout_shift: f64,
    first_contentful_paint: f64,
    first_contentful_paint_3g: f64,
    js_execution_time: f64,
    largest_contentful_paint: f64,
    speed_index: f64,
    time_to_interactive: f64,
    total_blocking_time: f64,
    score: f64,
}

async fn get_page_audits(
    url: &str,
    token: &str,
    number_of_runs: i8,
) -> Result<Vec<LHResult>, reqwest::Error> {
    let list_urls = (0..number_of_runs).map(|_| {
        format!("https://www.googleapis.com/pagespeedonline/v5/runPagespeed?key={api_key}&url={url}&strategy=mobile&category=performance", url = url, api_key = token)
    }).collect::<Vec<String>>();
    let client = reqwest::Client::new();

    let list_responses =
        futures::future::join_all(list_urls.iter().map(|url| client.get(url).send())).await;

    let mut list_audits = Vec::new();
    for res in list_responses {
        let audit = match res {
            Ok(result) => match result.json::<PSIResult>().await {
                Ok(json) => LHResult {
                    audits: json.lighthouse_result.audits,
                    categories: json.lighthouse_result.categories,
                },
                Err(_) => panic!("Problem parsing response from  {site}", site = url),
            },
            Err(_) => panic!("Problem parsing response from  {site}", site = url),
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
        first_contentful_paint_3g: lh_results
            .iter()
            .map(|result| result.audits.first_contentful_paint_3g.numeric_value)
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

fn calculate_mean(page_results: &PSIResultValues, number_of_runs: i8) -> PSIStatisticResult {
    return PSIStatisticResult {
        cumulative_layout_shift: mean(&page_results.cumulative_layout_shift, number_of_runs),
        first_contentful_paint: mean(&page_results.first_contentful_paint, number_of_runs),
        first_contentful_paint_3g: mean(&page_results.first_contentful_paint_3g, number_of_runs),
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
    page_mean: &PSIStatisticResult,
    number_of_runs: i8,
) -> PSIStatisticResult {
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
        first_contentful_paint_3g: std_deviation(
            &page_results.first_contentful_paint_3g,
            page_mean.first_contentful_paint_3g,
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

fn print_table_result(page_mean: &PSIStatisticResult, page_std_deviation: &PSIStatisticResult) {
    println!("| Metric | Mean | Standard deviation |");
    println!("|--------|--------|--------|");
    println!(
        "| Cumulative Layout shift (CLS) | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.cumulative_layout_shift,
        std_deviation = page_std_deviation.cumulative_layout_shift
    );
    println!(
        "| First Contentful Paint (FCP) | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.first_contentful_paint,
        std_deviation = page_std_deviation.first_contentful_paint,
    );
    println!(
        "| First Contentful Paint 3g (FCP) | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.first_contentful_paint_3g,
        std_deviation = page_std_deviation.first_contentful_paint_3g,
    );
    println!(
        "| Largest Contentful Paint (LCP) | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.largest_contentful_paint,
        std_deviation = page_std_deviation.largest_contentful_paint,
    );
    println!(
        "| Time to Interactive (TTI) | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.time_to_interactive,
        std_deviation = page_std_deviation.time_to_interactive,
    );
    println!(
        "| Total Blocking Time (TBT) | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.total_blocking_time,
        std_deviation = page_std_deviation.total_blocking_time,
    );
    println!(
        "| Performance score | {mean:.3} | {std_deviation:.6} |",
        mean = page_mean.score,
        std_deviation = page_std_deviation.score,
    );
    println!(
        "| JavaScript Execution Time | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.js_execution_time,
        std_deviation = page_std_deviation.js_execution_time,
    );
    println!(
        "| Speed Index | {mean:.2} | {std_deviation:.2} |",
        mean = page_mean.speed_index,
        std_deviation = page_std_deviation.speed_index,
    );
}

fn print_result(
    page_url: &str,
    page_mean: &PSIStatisticResult,
    page_std_deviation: &PSIStatisticResult,
) {
    println!("Page result - {url}", url = page_url);
    print_table_result(page_mean, page_std_deviation);
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
            .short('N')
            .long("number-of-runs")
            .help("Number of PSI tests for each page."),
        )
        .arg(
            Arg::new("first-page")
            .required(true)
            .help("Page URL.")
            .index(1)
        ).get_matches();

    // Required value
    let token = matches.value_of("token").expect("Token is required!");
    let number_of_runs = match matches.value_of("number_of_runs") {
        Some(value) => value.parse::<i8>().unwrap(),
        None => SAMPLE,
    };
    // Required value
    let first_page_url = matches
        .value_of("first-page")
        .expect("Page URL is required");

    let first_page_result =
        map_audits(&get_page_audits(first_page_url, token, number_of_runs).await?);

    let first_page_mean = calculate_mean(&first_page_result, number_of_runs);

    let first_page_deviation =
        calculate_deviation(&first_page_result, &first_page_mean, number_of_runs);

    print_result(first_page_url, &first_page_mean, &first_page_deviation);

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    psi_test().await?;

    return Ok(());
}
