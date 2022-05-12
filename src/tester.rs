use futures::StreamExt;

use crate::{Audit, Audits, Categories, Category, LHResult, PSIResult, PSIResultValues, Strategy};

const BUFFER_SIZE: usize = 15;
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

pub async fn get_page_audits(
    url: &str,
    token: &str,
    number_of_runs: i8,
    strategy: Strategy,
) -> Result<PSIResultValues, reqwest::Error> {
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

    return Ok(map_audits(&list_audits));
}

pub fn map_audits(lh_results: &Vec<LHResult>) -> PSIResultValues {
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
