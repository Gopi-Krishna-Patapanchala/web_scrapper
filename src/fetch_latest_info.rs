#[allow(unused_imports, unused_must_use)]
use reqwest::blocking::Client;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::fs::{OpenOptions, remove_file};

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct PlayerInfo {
    id: String,
    firstName: String,
    lastName: String,
    dateOfBirth: String,
    style: Vec<Style>,
    country: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Style {
    description: String,
}

pub fn style_description(player_info: &PlayerInfo) -> String {
    player_info
        .style
        .iter()
        .map(|s| s.description.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn update_master() {
    let master_final_data_path = "./Master_data/master_data.csv";
    let appended_data_file_path = "./Master_data/latest_data.csv";
    let record_tracking_file_path = "./intrim/latest.csv";

    // Check if the latest.csv file is available
    if !std::path::Path::new(record_tracking_file_path).exists() {
        println!("latest.csv file is not available. Which we have all player information in Master data skipping the master data updation process.");
        return;
    }

    // Read the latest.csv file
    let mut new_player_info_csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(record_tracking_file_path)
        .unwrap();

    // Create or open the master file in append mode
    let master_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&master_final_data_path)
        .unwrap();

    let mut master_final_data_csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(master_file);

    // Create a new file for appended data
    let appended_data_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&appended_data_file_path)
        .unwrap();

    let mut appended_data_csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(appended_data_file);

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // Iterate over each row in latest.csv
    for result in new_player_info_csv_reader.records() {
        match result {
            Ok(record) => {
                if !record.is_empty() {
                    println!("{:?}",record);
                    let player_id = record.get(0).unwrap_or("default_value");

                    let url = format!(
                        "http://core.espnuk.org/v2/sports/cricket/athletes/{}",
                        player_id
                    );

                    match client.get(&url).send() {
                        Ok(response) => {
                            if response.status().is_success() {
                                let content = response.text().unwrap();
                                let player_info: PlayerInfo =
                                    serde_json::from_str(&content).unwrap();
                                let player_style = style_description(&player_info);

                                let record_for_final_data: Vec<String> = vec![
                                    player_id.to_string(),
                                    player_info.firstName.clone(),
                                    player_info.lastName.clone(),
                                    player_info.dateOfBirth.clone(),
                                    player_style,
                                    player_info.country.to_string(),
                                ];

                                if let Err(err) =
                                    master_final_data_csv_writer.write_record(&record_for_final_data)
                                {
                                    eprintln!(
                                        "Error writing record to master final data file: {}",
                                        err
                                    );
                                }

                                // Write to the appended data file as well
                                if let Err(err) =
                                    appended_data_csv_writer.write_record(&record_for_final_data)
                                {
                                    eprintln!(
                                        "Error writing record to appended data file: {}",
                                        err
                                    );
                                }
                            } else {
                                eprintln!(
                                    "Received a non-successful status code: {}",
                                    response.status()
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Error fetching player data for final report information: {:?}",
                                e
                            );
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading record from new player info file: {}", err);
            }
        }
    }

    if let Err(err) = master_final_data_csv_writer.flush() {
        eprintln!("Error flushing changes to master final data file: {}", err);
    }

    if let Err(err) = appended_data_csv_writer.flush() {
        eprintln!("Error flushing changes to appended data file: {}", err);
    }

    // Remove latest.csv after processing
    if let Err(err) = remove_file(record_tracking_file_path) {
        eprintln!("Error deleting latest.csv file: {}", err);
    }
    println!("Update master raw data completed successfully");
}
