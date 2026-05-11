use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{watch, Mutex, RwLock};
use uuid::Uuid;

// ── Public types ──────────────────────────────────────────────

/// A pipeline is a DAG of tasks submitted by the user.
#[derive(Debug, Clone, Deserialize)]
pub struct PipelineRequest {
    pub name: String,
    pub tasks: Vec<TaskDef>,
}

/// One task inside the pipeline definition.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskDef {
    /// Unique name within this pipeline (e.g. "fetch_weather").
    pub id: String,
    /// IDs of tasks that must complete before this one starts.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Simulated work type – determines how the task behaves.
    /// Supported: "http_fetch", "transform", "aggregate", "llm_generate", "validate"
    pub task_type: String,
    /// Arbitrary config passed to the task executor.
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineResult {
    pub pipeline_id: String,
    pub name: String,
    pub status: PipelineStatus,
    pub total_duration_ms: u64,
    pub tasks: Vec<TaskResult>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskResult {
    pub id: String,
    pub status: TaskStatus,
    pub duration_ms: u64,
    pub output: serde_json::Value,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

// ── Pipeline store ────────────────────────────────────────────

/// In-memory store of pipeline executions.
#[derive(Clone, Default)]
pub struct PipelineStore {
    inner: Arc<RwLock<HashMap<String, PipelineResult>>>,
}

impl PipelineStore {
    pub async fn get(&self, id: &str) -> Option<PipelineResult> {
        self.inner.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<PipelineResult> {
        self.inner.read().await.values().cloned().collect()
    }

    async fn upsert(&self, result: PipelineResult) {
        self.inner
            .write()
            .await
            .insert(result.pipeline_id.clone(), result);
    }
}

// ── DAG validation ────────────────────────────────────────────

fn validate_dag(tasks: &[TaskDef]) -> Result<Vec<String>, String> {
    let ids: HashSet<&str> = tasks.iter().map(|t| t.id.as_str()).collect();

    // Check all dependencies reference existing tasks
    for task in tasks {
        for dep in &task.depends_on {
            if !ids.contains(dep.as_str()) {
                return Err(format!(
                    "Task '{}' depends on '{}' which does not exist",
                    task.id, dep
                ));
            }
        }
    }

    // Topological sort (Kahn's algorithm) to detect cycles
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for task in tasks {
        in_degree.entry(task.id.as_str()).or_insert(0);
        for dep in &task.depends_on {
            adjacency
                .entry(dep.as_str())
                .or_default()
                .push(task.id.as_str());
            *in_degree.entry(task.id.as_str()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node.to_string());
        if let Some(neighbors) = adjacency.get(node) {
            for &neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    if order.len() != tasks.len() {
        return Err("Cycle detected in task dependencies".into());
    }

    Ok(order)
}

// ── Task executor (simulated workloads) ───────────────────────

async fn execute_task(task: &TaskDef) -> Result<serde_json::Value, String> {
    match task.task_type.as_str() {
        "http_fetch" => {
            let url = task.config.get("url").and_then(|v| v.as_str()).unwrap_or("https://httpbin.org/get");
            let delay_ms = task.config.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(0);

            if delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            let client = reqwest::Client::new();
            match client.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let body_len = resp.text().await.unwrap_or_default().len();
                    Ok(serde_json::json!({
                        "source": url,
                        "status_code": status,
                        "body_bytes": body_len
                    }))
                }
                Err(e) => Err(format!("HTTP fetch failed: {e}")),
            }
        }

        "transform" => {
            let operation = task.config.get("operation").and_then(|v| v.as_str()).unwrap_or("passthrough");
            let delay_ms = task.config.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(100);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            Ok(serde_json::json!({
                "operation": operation,
                "result": format!("Transformed data with '{}'", operation)
            }))
        }

        "aggregate" => {
            let strategy = task.config.get("strategy").and_then(|v| v.as_str()).unwrap_or("merge");
            let sources: Vec<&str> = task.depends_on.iter().map(|s| s.as_str()).collect();
            let delay_ms = task.config.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(50);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            Ok(serde_json::json!({
                "strategy": strategy,
                "aggregated_from": sources,
                "record_count": sources.len() * 42
            }))
        }

        "validate" => {
            let schema = task.config.get("schema").and_then(|v| v.as_str()).unwrap_or("default");
            let delay_ms = task.config.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(80);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            Ok(serde_json::json!({
                "schema": schema,
                "valid": true,
                "errors": []
            }))
        }

        "llm_generate" => {
            let prompt = task.config.get("prompt").and_then(|v| v.as_str()).unwrap_or("generate something");
            // Simulate LLM processing time
            let delay_ms = task.config.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(500);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            Ok(serde_json::json!({
                "prompt": prompt,
                "generated": format!("LLM output for: {}", prompt),
                "tokens": 150
            }))
        }

        other => {
            // Unknown task type – still runs but marks as generic
            let delay_ms = task.config.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(200);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            Ok(serde_json::json!({
                "task_type": other,
                "result": "executed"
            }))
        }
    }
}

// ── Pipeline executor (concurrent DAG) ────────────────────────

pub async fn run_pipeline(req: PipelineRequest, store: PipelineStore) -> Result<PipelineResult, String> {
    // Validate the DAG first
    let _topo_order = validate_dag(&req.tasks)?;

    let pipeline_id = Uuid::new_v4().to_string();
    let start = Instant::now();

    // Build task map and dependency tracking
    let task_map: HashMap<String, TaskDef> = req
        .tasks
        .iter()
        .map(|t| (t.id.clone(), t.clone()))
        .collect();

    let results: Arc<Mutex<HashMap<String, TaskResult>>> = Arc::new(Mutex::new(HashMap::new()));

    // Initialize all tasks as pending
    {
        let mut res = results.lock().await;
        for task in &req.tasks {
            res.insert(
                task.id.clone(),
                TaskResult {
                    id: task.id.clone(),
                    status: TaskStatus::Pending,
                    duration_ms: 0,
                    output: serde_json::Value::Null,
                    started_at: None,
                    finished_at: None,
                },
            );
        }
    }

    // Store initial state
    store
        .upsert(PipelineResult {
            pipeline_id: pipeline_id.clone(),
            name: req.name.clone(),
            status: PipelineStatus::Running,
            total_duration_ms: 0,
            tasks: results.lock().await.values().cloned().collect(),
        })
        .await;

    // Completion channels: each task gets a watch channel so dependents can await it
    let mut completion_txs: HashMap<String, watch::Sender<bool>> = HashMap::new();
    let mut completion_rxs: HashMap<String, watch::Receiver<bool>> = HashMap::new();

    for task in &req.tasks {
        let (tx, rx) = watch::channel(false);
        completion_txs.insert(task.id.clone(), tx);
        completion_rxs.insert(task.id.clone(), rx);
    }

    let completion_rxs = Arc::new(completion_rxs);

    // Spawn all tasks concurrently – each waits for its own dependencies
    let mut handles = Vec::new();

    for task in &req.tasks {
        let task_def = task_map[&task.id].clone();
        let results = Arc::clone(&results);
        let completion_rxs = Arc::clone(&completion_rxs);
        let completion_tx = completion_txs.remove(&task.id).unwrap();
        let dep_ids: Vec<String> = task.depends_on.clone();

        let handle = tokio::spawn(async move {
            // Wait for all dependencies to complete
            for dep_id in &dep_ids {
                let mut rx = completion_rxs[dep_id].clone();
                // Wait until the dependency signals completion
                while !*rx.borrow() {
                    if rx.changed().await.is_err() {
                        break;
                    }
                }

                // Check if dependency failed – if so, skip this task
                let dep_status = {
                    let res = results.lock().await;
                    res.get(dep_id).map(|r| r.status.clone())
                };
                if dep_status == Some(TaskStatus::Failed) {
                    let mut res = results.lock().await;
                    if let Some(r) = res.get_mut(&task_def.id) {
                        r.status = TaskStatus::Skipped;
                        r.output = serde_json::json!({"reason": format!("dependency '{}' failed", dep_id)});
                    }
                    let _ = completion_tx.send(true);
                    return;
                }
            }

            // Mark as running
            let task_start = Instant::now();
            let started_at = chrono::Utc::now().to_rfc3339();
            {
                let mut res = results.lock().await;
                if let Some(r) = res.get_mut(&task_def.id) {
                    r.status = TaskStatus::Running;
                    r.started_at = Some(started_at.clone());
                }
            }

            // Execute
            let outcome = execute_task(&task_def).await;
            let duration = task_start.elapsed().as_millis() as u64;
            let finished_at = chrono::Utc::now().to_rfc3339();

            // Record result
            {
                let mut res = results.lock().await;
                if let Some(r) = res.get_mut(&task_def.id) {
                    r.duration_ms = duration;
                    r.finished_at = Some(finished_at);
                    match outcome {
                        Ok(output) => {
                            r.status = TaskStatus::Completed;
                            r.output = output;
                        }
                        Err(e) => {
                            r.status = TaskStatus::Failed;
                            r.output = serde_json::json!({"error": e});
                        }
                    }
                }
            }

            // Signal completion
            let _ = completion_tx.send(true);
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }

    let total_duration = start.elapsed().as_millis() as u64;
    let final_results: Vec<TaskResult> = results.lock().await.values().cloned().collect();

    let any_failed = final_results.iter().any(|r| r.status == TaskStatus::Failed);
    let status = if any_failed {
        PipelineStatus::Failed
    } else {
        PipelineStatus::Completed
    };

    let pipeline_result = PipelineResult {
        pipeline_id,
        name: req.name,
        status,
        total_duration_ms: total_duration,
        tasks: final_results,
    };

    store.upsert(pipeline_result.clone()).await;

    Ok(pipeline_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_linear_pipeline() {
        let req = PipelineRequest {
            name: "test_linear".into(),
            tasks: vec![
                TaskDef {
                    id: "a".into(),
                    depends_on: vec![],
                    task_type: "transform".into(),
                    config: serde_json::json!({"delay_ms": 10}),
                },
                TaskDef {
                    id: "b".into(),
                    depends_on: vec!["a".into()],
                    task_type: "transform".into(),
                    config: serde_json::json!({"delay_ms": 10}),
                },
            ],
        };

        let store = PipelineStore::default();
        let result = run_pipeline(req, store).await.unwrap();
        assert_eq!(result.status, PipelineStatus::Completed);
        assert_eq!(result.tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let req = PipelineRequest {
            name: "test_cycle".into(),
            tasks: vec![
                TaskDef {
                    id: "a".into(),
                    depends_on: vec!["b".into()],
                    task_type: "transform".into(),
                    config: serde_json::json!({}),
                },
                TaskDef {
                    id: "b".into(),
                    depends_on: vec!["a".into()],
                    task_type: "transform".into(),
                    config: serde_json::json!({}),
                },
            ],
        };

        let store = PipelineStore::default();
        let result = run_pipeline(req, store).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cycle detected"));
    }

    #[tokio::test]
    async fn test_concurrent_fanout() {
        let req = PipelineRequest {
            name: "test_fanout".into(),
            tasks: vec![
                TaskDef {
                    id: "source".into(),
                    depends_on: vec![],
                    task_type: "transform".into(),
                    config: serde_json::json!({"delay_ms": 10}),
                },
                TaskDef {
                    id: "branch_a".into(),
                    depends_on: vec!["source".into()],
                    task_type: "transform".into(),
                    config: serde_json::json!({"delay_ms": 50}),
                },
                TaskDef {
                    id: "branch_b".into(),
                    depends_on: vec!["source".into()],
                    task_type: "transform".into(),
                    config: serde_json::json!({"delay_ms": 50}),
                },
                TaskDef {
                    id: "merge".into(),
                    depends_on: vec!["branch_a".into(), "branch_b".into()],
                    task_type: "aggregate".into(),
                    config: serde_json::json!({"delay_ms": 10}),
                },
            ],
        };

        let store = PipelineStore::default();
        let start = Instant::now();
        let result = run_pipeline(req, store).await.unwrap();
        let elapsed = start.elapsed().as_millis();

        assert_eq!(result.status, PipelineStatus::Completed);
        // If branches ran concurrently, total should be ~70ms, not ~120ms
        assert!(elapsed < 200, "Took {}ms – branches should run concurrently", elapsed);
    }
}
