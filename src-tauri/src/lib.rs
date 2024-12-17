use csv_parser::extract_non_empty_first_column;
use export::run_export;
use nlp::{group_sentences_by_cluster, reduce_with_pca};
use std::{collections::HashMap, sync::Arc};
use tauri::{
    async_runtime::{self, Mutex},
    path::BaseDirectory,
    Manager,
};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

use hdbscan::{DistanceMetric, Hdbscan, HdbscanHyperParams, NnAlgorithm};

mod csv_parser;
mod export;
mod nlp;

#[allow(unused_imports)]
use burn::{
    backend::{Autodiff, Wgpu},
    prelude::Backend,
    prelude::*,
    tensor::{Device, Tensor},
};
#[allow(unused_imports)]
use fast_umap::{
    chart,
    model::{UMAPModel, UMAPModelConfigBuilder},
    train::{train, TrainingConfig},
    utils::*,
};
use fastembed::{
    read_file_to_bytes, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde_json::Value;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn process_strings(
    app: tauri::AppHandle,
    lines: Vec<String>,
    min_cluster_size: usize,
    min_samples: usize,
) -> Value {
    // Files from Qdrant/all-MiniLM-L6-v2-onnx
    // model.onnx
    let onnx_file = get_model_file("model/model.onnx", &app);
    // tokenizer.json
    let tokenizer_file = get_model_file("model/tokenizer.json", &app);
    // config.json
    let config_file = get_model_file("model/config.json", &app);
    // special_tokens_map.json
    let special_tokens_map_file = get_model_file("model/special_tokens_map.json", &app);
    // tokenizer_config.json
    let tokenizer_config_file = get_model_file("model/tokenizer_config.json", &app);

    let tokenizer_files = TokenizerFiles {
        tokenizer_file,
        config_file,
        special_tokens_map_file,
        tokenizer_config_file,
    };
    let base_model = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files)
        .with_quantization(QuantizationMode::Dynamic)
        .with_pooling(Pooling::Mean);

    let model = TextEmbedding::try_new_from_user_defined(base_model, Default::default()).unwrap();

    let embeddings = model.embed(lines.clone(), None).unwrap();

    let embeddings_f64 = embeddings
        .into_iter()
        .map(|embedding| {
            embedding
                .into_iter()
                .map(|value| value as f64)
                .collect::<Vec<f64>>()
        })
        .collect::<Vec<Vec<f64>>>();

    let reduced_embeddings_f64 = reduce_with_pca(&embeddings_f64, 5);

    let cluster_labels = cluster_embeddings(&reduced_embeddings_f64, min_cluster_size, min_samples);

    let line_cluster_pairs: Vec<_> = lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| (line, cluster_labels[index]))
        .collect();

    let clusters = group_sentences_by_cluster(line_cluster_pairs);

    println!("{:#?}", clusters);

    let json_response = serde_json::to_value(&clusters).expect("Failed to serialize JSON");

    println!("{:#?}", json_response);

    json_response
}

#[tauri::command]
async fn from_csv(app: tauri::AppHandle) -> Vec<String> {
    let result = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = oneshot::channel(); // Create a oneshot channel

    let result_clone = Arc::clone(&result);
    app.dialog()
        .file()
        .add_filter(String::from("CSV files"), &["csv"])
        .pick_file(move |file_path| {
            let result_clone = Arc::clone(&result_clone);
            async_runtime::spawn(async move {
                if let Some(tauri_file_path) = file_path {
                    if let Some(path) = tauri_file_path.as_path() {
                        if let Ok(data) = extract_non_empty_first_column(path) {
                            println!("Intermediate Result: {:?}", data);
                            let mut result = result_clone.lock().await;
                            *result = data; // Assign the data to result
                        }
                    }
                }
                // Signal that the task is done, regardless of the outcome
                let _ = tx.send(());
            });
        });

    // Wait for the dialog operation to complete
    let _ = rx.await;

    // Safely retrieve the final result
    let final_result = result.lock().await.clone();
    println!("Final Result: {:?}", final_result);
    final_result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            process_strings,
            from_csv,
            export_findings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn cluster_embeddings(
    data: &[Vec<f64>],
    min_cluster_size: usize,
    min_samples: usize,
) -> Vec<Option<usize>> {
    let hyper_params = HdbscanHyperParams::builder()
        .min_cluster_size(min_cluster_size) // Equivalent to BERTopic's default
        .min_samples(min_samples) // Adjustable based on dataset size
        .dist_metric(DistanceMetric::Euclidean) // Default metric
        .nn_algorithm(NnAlgorithm::Auto) // Efficient for dense data
        .build();

    let clusterer = Hdbscan::new(data, hyper_params);

    match clusterer.cluster() {
        Ok(labels) => labels
            .into_iter()
            .map(|label| {
                if label >= 0 {
                    Some(label as usize)
                } else {
                    None
                }
            })
            .collect(),
        Err(err) => {
            println!("Clustering failed: {:?}", err);
            vec![None; data.len()]
        }
    }
}

fn get_model_file(file_name: &str, app: &tauri::AppHandle) -> Vec<u8> {
    let onnx_file_path = app
        .app_handle()
        .path()
        .resolve(file_name, BaseDirectory::Resource)
        .unwrap();
    read_file_to_bytes(&onnx_file_path).unwrap()
}

#[tauri::command]
async fn export_findings(app: tauri::AppHandle, data: HashMap<String, Vec<String>>) {
    run_export(app, data).await;
}
