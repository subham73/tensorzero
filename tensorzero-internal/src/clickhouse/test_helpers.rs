#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]
#[cfg(feature = "e2e_tests")]
use super::escape_string_for_clickhouse_comparison;
use super::ClickHouseConnectionInfo;
use serde::Deserialize;
use serde_json::Value;
#[cfg(feature = "e2e_tests")]
use std::collections::HashMap;
use uuid::Uuid;

lazy_static::lazy_static! {
    pub static ref CLICKHOUSE_URL: String = std::env::var("TENSORZERO_CLICKHOUSE_URL").expect("Environment variable TENSORZERO_CLICKHOUSE_URL must be set");
}

pub async fn get_clickhouse() -> ClickHouseConnectionInfo {
    let clickhouse_url = url::Url::parse(&CLICKHOUSE_URL).unwrap();
    let start = std::time::Instant::now();
    println!("Connecting to ClickHouse");
    let res = ClickHouseConnectionInfo::new(clickhouse_url.as_ref())
        .await
        .expect("Failed to connect to ClickHouse");
    println!("Connected to ClickHouse in {:?}", start.elapsed());
    res
}

#[cfg(feature = "e2e_tests")]
pub async fn clickhouse_flush_async_insert(clickhouse: &ClickHouseConnectionInfo) {
    if let Err(e) = clickhouse
        .run_query_synchronous("SYSTEM FLUSH ASYNC INSERT QUEUE".to_string(), None)
        .await
    {
        tracing::warn!("Failed to run `SYSTEM FLUSH ASYNC INSERT QUEUE`: {}", e);
    }
}

#[allow(dead_code)]
pub async fn select_chat_datapoint_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Value> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let query = format!(
        "SELECT * FROM ChatInferenceDatapoint WHERE id = '{}' LIMIT 1 FORMAT JSONEachRow",
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

#[allow(dead_code)]
pub async fn select_json_datapoint_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Value> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let query = format!(
        "SELECT * FROM JsonInferenceDatapoint WHERE id = '{}' LIMIT 1 FORMAT JSONEachRow",
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

pub async fn select_chat_inference_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Value> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let query = format!(
        "SELECT * FROM ChatInference WHERE id = '{}' LIMIT 1 FORMAT JSONEachRow",
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

pub async fn select_json_inference_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Value> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    // We limit to 1 in case there are duplicate entries (can be caused by a race condition in polling batch inferences)
    let query = format!(
        "SELECT * FROM JsonInference WHERE id = '{}' LIMIT 1 FORMAT JSONEachRow",
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

pub async fn select_model_inference_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Value> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    // We limit to 1 in case there are duplicate entries (can be caused by a race condition in polling batch inferences)
    let query = format!(
        "SELECT * FROM ModelInference WHERE inference_id = '{}' LIMIT 1 FORMAT JSONEachRow",
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

pub async fn select_model_inferences_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Vec<Value>> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    // We limit to 1 in case there are duplicate entries (can be caused by a race condition in polling batch inferences)
    let query = format!(
        "SELECT * FROM ModelInference WHERE inference_id = '{}' FORMAT JSONEachRow",
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json_rows: Vec<Value> = text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    if json_rows.is_empty() {
        None
    } else {
        Some(json_rows)
    }
}

pub async fn select_inference_tags_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    function_name: &str,
    tag_key: &str,
    tag_value: &str,
    inference_id: Uuid,
) -> Option<Value> {
    #[cfg(feature = "e2e_tests")]
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let query = format!(
        "SELECT * FROM InferenceTag WHERE function_name = '{}' AND key = '{}' AND value = '{}' AND inference_id = '{}' FORMAT JSONEachRow",
        function_name, tag_key, tag_value, inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

pub async fn select_batch_model_inference_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    inference_id: Uuid,
) -> Option<Value> {
    let query = format!(
        r#"
        SELECT bmi.*
        FROM BatchModelInference bmi
        INNER JOIN BatchIdByInferenceId bid ON bmi.inference_id = bid.inference_id
        WHERE bid.inference_id = '{}'
        FORMAT JSONEachRow"#,
        inference_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    Some(serde_json::from_str(&text).unwrap())
}

pub async fn select_batch_model_inferences_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    batch_id: Uuid,
) -> Option<Vec<Value>> {
    let query = format!(
        r#"
        SELECT bmi.*
        FROM BatchModelInference bmi
        WHERE bmi.batch_id = '{}'
        FORMAT JSONEachRow"#,
        batch_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json_rows: Vec<Value> = text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    Some(json_rows)
}

pub async fn select_latest_batch_request_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    batch_id: Uuid,
) -> Option<Value> {
    let query = format!(
        "SELECT * FROM BatchRequest WHERE batch_id = '{}' ORDER BY timestamp DESC LIMIT 1 FORMAT JSONEachRow",
        batch_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

#[cfg(feature = "e2e_tests")]
pub async fn select_feedback_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    table_name: &str,
    feedback_id: Uuid,
) -> Option<Value> {
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let query = format!(
        "SELECT * FROM {} WHERE id = '{}' FORMAT JSONEachRow",
        table_name, feedback_id
    );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

#[cfg(feature = "e2e_tests")]
pub async fn select_feedback_by_target_id_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    table_name: &str,
    target_id: Uuid,
    metric_name: Option<&str>,
) -> Option<Value> {
    let query = match metric_name {
        Some(metric_name) => {
            format!(
                "SELECT * FROM {} WHERE target_id = '{}' AND metric_name = '{}' FORMAT JSONEachRow",
                table_name, target_id, metric_name
            )
        }
        None => format!(
            "SELECT * FROM {} WHERE target_id = '{}' FORMAT JSONEachRow",
            table_name, target_id
        ),
    };

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

#[cfg(feature = "e2e_tests")]
pub async fn stale_datapoint_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    datapoint_id: Uuid,
) {
    let query = format!(
        "INSERT INTO ChatInferenceDatapoint
        (
            dataset_name,
            function_name,
            id,
            episode_id,
            input,
            output,
            tool_params,
            tags,
            auxiliary,
            is_deleted,
            source_inference_id,
            staled_at,
            updated_at
        )
        SELECT
            dataset_name,
            function_name,
            id,
            episode_id,
            input,
            output,
            tool_params,
            tags,
            auxiliary,
            is_deleted,
            source_inference_id,
            now64() as staled_at,
            now64() as updated_at
        FROM ChatInferenceDatapoint FINAL
        WHERE id = '{}'",
        datapoint_id
    );

    // Execute the query and ignore errors (in case the datapoint doesn't exist in this table)
    let _ = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await;

    let query = format!(
        "INSERT INTO JsonInferenceDatapoint
        (
            dataset_name,
            function_name,
            id,
            episode_id,
            input,
            output,
            output_schema,
            tags,
            auxiliary,
            is_deleted,
            source_inference_id,
            staled_at,
            updated_at
        )
        SELECT
            dataset_name,
            function_name,
            id,
            episode_id,
            input,
            output,
            output_schema,
            tags,
            auxiliary,
            is_deleted,
            source_inference_id,
            now64() as staled_at,
            now64() as updated_at
        FROM JsonInferenceDatapoint FINAL
        WHERE id = '{}'",
        datapoint_id
    );

    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let _ = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await;
}

#[cfg(feature = "e2e_tests")]
pub async fn select_feedback_tags_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    metric_name: &str,
    tag_key: &str,
    tag_value: &str,
) -> Option<Value> {
    clickhouse_flush_async_insert(clickhouse_connection_info).await;

    let query = format!(
            "SELECT * FROM FeedbackTag WHERE metric_name = '{}' AND key = '{}' AND value = '{}' FORMAT JSONEachRow",
            metric_name, tag_key, tag_value
        );

    let text = clickhouse_connection_info
        .run_query_synchronous(query, None)
        .await
        .unwrap();
    let json: Value = serde_json::from_str(&text).ok()?;
    Some(json)
}

#[derive(Debug, Deserialize)]
pub struct StaticEvaluationHumanFeedback {
    pub metric_name: String,
    pub datapoint_id: Uuid,
    pub output: String,
    pub value: String,
    pub feedback_id: Uuid,
}

#[cfg(feature = "e2e_tests")]
pub async fn select_human_static_evaluation_feedback_clickhouse(
    clickhouse_connection_info: &ClickHouseConnectionInfo,
    metric_name: &str,
    datapoint_id: Uuid,
    output: &str,
) -> Option<StaticEvaluationHumanFeedback> {
    let datapoint_id_str = datapoint_id.to_string();
    let escaped_output = escape_string_for_clickhouse_comparison(output);
    let params = HashMap::from([
        ("metric_name", metric_name),
        ("datapoint_id", &datapoint_id_str),
        ("output", &escaped_output),
    ]);
    let query = r#"
        SELECT * FROM StaticEvaluationHumanFeedback
        WHERE
            metric_name = {metric_name:String}
            AND datapoint_id = {datapoint_id:UUID}
            AND output = {output:String}
        FORMAT JSONEachRow"#
        .to_string();
    let text = clickhouse_connection_info
        .run_query_synchronous(query, Some(&params))
        .await
        .unwrap();
    if text.is_empty() {
        // Return None if the query returns no rows
        None
    } else {
        // Panic if the query fails to parse or multiple rows are returned
        let json: StaticEvaluationHumanFeedback = serde_json::from_str(&text).unwrap();
        Some(json)
    }
}
