use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
};

use csv::Writer;
use image::{ImageFormat, Rgba};
use nanorand::{Rng, WyRand};
use palette::{Hsl, IntoColor, Pixel, Srgb};
use slugify::slugify;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use wcloud::{Tokenizer, Word, WordCloud, WordCloudSize, DEFAULT_EXCLUDE_WORDS_TEXT};

pub async fn run_export(app: tauri::AppHandle, data: HashMap<String, Vec<String>>) {
    // Get user save file location and folder name
    // app.app_handle()
    //     .dialog()
    //     .file()
    //     .pick_folder(move |file_path| {
    //         if let Some(tauri_path) = file_path {
    //             let system_path = tauri_path.as_path().unwrap();
    //             // Spawn the async logic here
    //             let system_path_buf = system_path.to_path_buf();
    //             tauri::async_runtime::spawn(handle_folder_selection(data.clone(), system_path_buf));
    //         }
    //     });
    // Use the async folder picker
    if let Some(file_path) = pick_folder_async(app).await {
        // Call the async logic for exporting
        handle_folder_selection(data, file_path).await;
    } else {
        println!("No folder was selected. Export canceled.");
    }
}

async fn handle_folder_selection(data: HashMap<String, Vec<String>>, system_path: PathBuf) {
    export_cluster_csv(&data, &system_path).await;
    for (key, values) in data {
        export_word_cloud(&key, &values, &system_path).await;
    }
}

async fn export_cluster_csv(data: &HashMap<String, Vec<String>>, parent_folder: &Path) {
    // Sort keys for consistent column order
    let mut headers: Vec<_> = data.keys().collect();
    headers.sort();

    // Find the maximum number of rows among all columns
    let max_rows = headers.iter().map(|h| data[*h].len()).max().unwrap_or(0);

    let output_path = parent_folder.join("cluster.csv");
    let file = File::create(&output_path).unwrap();
    let mut wtr = Writer::from_writer(file);

    // Write headers
    wtr.write_record(&headers).unwrap();

    // Write all rows up to max_rows
    // If a column doesn't have a value for a given row, write empty string
    for i in 0..max_rows {
        let row: Vec<&str> = headers
            .iter()
            .map(|h| {
                let col = &data[*h];
                if i < col.len() {
                    &col[i]
                } else {
                    ""
                }
            })
            .collect();
        wtr.write_record(row).unwrap();
    }

    wtr.flush().unwrap();
}

async fn export_word_cloud(name: &str, values: &Vec<String>, path: &Path) {
    let text = values.join(" ");

    // If you have slugify:
    // let key_slug = slugify!(name, separator = "_");
    // Otherwise just replace spaces/slashes:
    let key_slug = name.to_lowercase().replace(' ', "_").replace('/', "_");

    let out_path = path.join(format!("{}.png", key_slug));

    // Create a filter set for ignored words
    let mut filter = DEFAULT_EXCLUDE_WORDS_TEXT.lines().collect::<HashSet<_>>();
    // let ignore_words = ["Dell", "HPE", "Lenovo", "Tech"]; // add more as needed
    let ignore_words = [
        // Brand and corporate references (with variations)
        "Dell",
        "dell",
        "DELL",
        "DellEMC",
        "dellemc",
        "DELLEMC",
        "DellEMC2",
        "dellemc2",
        "DELLEMC2",
        "DellTechnologies",
        "delltechnologies",
        "DELLTECHNOLOGIES",
        "DellTech",
        "delltech",
        "DELLTECH",
        "Alienware",
        "alienware",
        "ALIENWARE",
        "HPE",
        "hpe",
        "Hpe",
        "HewlettPackardEnterprise",
        "hewlettpackardenterprise",
        "HEWLETTPACKARDENTERPRISE",
        "HP",
        "hp",
        "Hp",
        "Lenovo",
        "lenovo",
        "LENOVO",
        "IBM",
        "ibm",
        "Ibm",
        "Apple",
        "apple",
        "APPLE",
        "Acer",
        "acer",
        "ACER",
        "Asus",
        "asus",
        "ASUS",
        "Microsoft",
        "microsoft",
        "MICROSOFT",
        "Intel",
        "intel",
        "INTEL",
        "AMD",
        "amd",
        "Amd",
        "Nvidia",
        "nvidia",
        "NVIDIA",
        "VMware",
        "vmware",
        "VMWARE",
        "Cisco",
        "cisco",
        "CISCO",
        "Oracle",
        "oracle",
        "ORACLE",
        // Dell product lines and related terms (with variations)
        "EMC",
        "emc",
        "Emc",
        "Inspiron",
        "inspiron",
        "INSPIRON",
        "Latitude",
        "latitude",
        "LATITUDE",
        "Precision",
        "precision",
        "PRECISION",
        "OptiPlex",
        "optiplex",
        "OPTIPLEX",
        "XPS",
        "xps",
        "Xps",
        "Vostro",
        "vostro",
        "VOSTRO",
        "Wyse",
        "wyse",
        "WYSE",
        "PowerEdge",
        "poweredge",
        "POWEREDGE",
        "PowerVault",
        "powervault",
        "POWERVAULT",
        "EqualLogic",
        "equallogic",
        "EQUALLOGIC",
        "Compellent",
        "compellent",
        "COMPELLENT",
        // General tech references (with variations)
        "Tech",
        "tech",
        "TECH",
        "Technologies",
        "technologies",
        "TECHNOLOGIES",
        "Technology",
        "technology",
        "TECHNOLOGY",
        "InfoTech",
        "infotech",
        "INFOTECH",
        "IT",
        "it",
        "It",
        // Survey meta terms (with variations)
        "Survey",
        "survey",
        "SURVEY",
        "Questionnaire",
        "questionnaire",
        "QUESTIONNAIRE",
        "Respondent",
        "respondent",
        "RESPONDENT",
        "Response",
        "response",
        "RESPONSE",
        "Feedback",
        "feedback",
        "FEEDBACK",
        "N/A",
        "n/a",
        "N/A", // N/A doesn't really have case variants but included anyway
        "NA",
        "na",
        "Na",
        "None",
        "none",
        "NONE",
        "Nothing",
        "nothing",
        "NOTHING",
        "NotApplicable",
        "notapplicable",
        "NOTAPPLICABLE",
        "Comment",
        "comment",
        "COMMENT",
        "Form",
        "form",
        "FORM",
        "Please",
        "please",
        "PLEASE",
        "Thank",
        "thank",
        "THANK",
        "Thanks",
        "thanks",
        "THANKS",
        "Reviewer",
        "reviewer",
        "REVIEWER",
        "User",
        "user",
        "USER",
        // Numerical placeholders or irrelevant tokens (with variations)
        "Q1",
        "q1",
        "Q1",
        "Q2",
        "q2",
        "Q2",
        "Q3",
        "q3",
        "Q3",
        "Q4",
        "q4",
        "Q4",
        "Q5",
        "q5",
        "Q5",
        "ID",
        "id",
        "Id",
        "TicketNumber",
        "ticketnumber",
        "TICKETNUMBER",
        "CaseNumber",
        "casenumber",
        "CASENUMBER",
        "RefNumber",
        "refnumber",
        "REFNUMBER",
    ];

    for w in ignore_words {
        filter.insert(w);
    }

    let tokenizer = Tokenizer::default()
        .with_max_words(100000)
        .with_filter(filter)
        .with_repeat(true);

    let wordcloud = WordCloud::default()
        .with_tokenizer(tokenizer)
        .with_rng_seed(0);

    let size = WordCloudSize::FromDimensions {
        width: 1000,
        height: 500,
    };

    // Light bluish color function
    let color_func = |word: &Word, _rng: &mut WyRand| {
        let freq = (word.frequency * 100.0) as u8;
        let saturation = match freq {
            90..=100 => word.frequency,
            20..=89 => 1.0,
            10..=19 => 0.8,
            6..=9 => 0.6,
            3..=5 => 0.3,
            _ => 0.2,
        };
        let col = Hsl::new(200.0, saturation, 0.5);
        let rgb: Srgb = col.into_color();
        let raw: [u8; 3] = rgb.into_format().into_raw();
        Rgba([raw[0], raw[1], raw[2], 1])
    };

    let wordcloud_image =
        wordcloud.generate_from_text_with_color_func(&text, size, 1.0, color_func);

    wordcloud_image.save(&out_path).unwrap();
}

async fn pick_folder_async(app: tauri::AppHandle) -> Option<PathBuf> {
    use tokio::sync::oneshot;

    let (sender, receiver) = oneshot::channel();

    app.dialog().file().pick_folder(move |file_path| {
        // Safely unwrap Option<FilePath>
        if let Some(file_path) = file_path {
            // Convert FilePath into PathBuf
            if let Some(path) = file_path.as_path() {
                sender.send(Some(path.to_path_buf())).ok();
            } else {
                sender.send(None).ok();
            }
        } else {
            sender.send(None).ok();
        }
    });

    receiver.await.ok().flatten()
}
