use std::collections::HashMap;

use ndarray::Array2;
use petal_decomposition::PcaBuilder;

pub fn reduce_with_pca(data: &Vec<Vec<f64>>, n_components: usize) -> Vec<Vec<f64>> {
    let n_samples = data.len();
    let n_features = data[0].len();

    // Convert input data to ndarray
    let flattened_data: Vec<f64> = data.iter().flatten().cloned().collect();
    let array_data = Array2::from_shape_vec((n_samples, n_features), flattened_data)
        .expect("Failed to create ndarray from input data");

    // Perform PCA
    let mut pca = PcaBuilder::new(n_components).build(); // Specify the desired number of components
    pca.fit(&array_data).expect("PCA fitting failed");

    // Transform the data to lower dimensions
    let reduced_array = pca.transform(&array_data).expect("PCA transform failed");

    // Convert reduced data back to Vec<Vec<f64>>
    reduced_array.outer_iter().map(|row| row.to_vec()).collect()
}

pub fn group_sentences_by_cluster(
    data: Vec<(String, Option<usize>)>,
) -> HashMap<usize, Vec<String>> {
    let mut grouped: HashMap<usize, Vec<String>> = HashMap::new();

    for (sentence, cluster) in data {
        if let Some(cluster_id) = cluster {
            grouped
                .entry(cluster_id)
                .or_insert_with(Vec::new)
                .push(sentence);
        }
    }

    grouped
}
