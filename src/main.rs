#[allow(non_snake_case)]
use reqwest::blocking::Client;
use scraper::{Selector,Html};
use select::document::Document;
use select::predicate::Name;
use regex::Regex;
use std::time::Duration;
use csv::{WriterBuilder,Writer};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::collections::HashSet;

mod connect_duck_db;
mod fetch_latest_info;
mod clean_up;

fn batting_scorecard(html: &str, file_path: &str) {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse(".ci-scorecard-table").expect("Failed to find the table");

    let csv_file_directory = format!("{}/scorecard", file_path);

    if let Err(e) = fs::create_dir_all(&csv_file_directory) {
        eprintln!("Error creating the directory: {:?}", e);
        return;
    }

    let mut table_count = 1;

    for table in document.select(&table_selector) {
        // Define a selector for table rows
        let row_selector = Selector::parse("tr").unwrap();

        // Create a CSV file for writing the scorecard
        let csv_filename = format!("{}/batting_scorecard{}.csv", csv_file_directory, table_count);

        //println!("{:?}", csv_filename);
        let mut writer = Writer::from_path(csv_filename.clone()).expect("Failed to create the CSV file");

        // Write the CSV header with the new 'caption_flag' column
        writer.write_record(&["Player", "Dismissal", "R", "B", "M", "4s", "6s", "SR"])
            .expect("Failed to write the header");

        // Extract player information and write to the CSV
        for row in table.select(&row_selector) {
            // Create a selector for table columns within each row
            let column_selector = Selector::parse("td").expect("Failed to create column selector");
            let columns = row.select(&column_selector);

            let data: Vec<String> = columns.map(|column| column.text().collect()).collect();

            // Ensure that the row contains data
            if data.len() == 8 {
                let player_name = &data[0];
                let dismissal = &data[1];
                

                writer.write_record(&[player_name, dismissal, &data[2], &data[3], &data[4], &data[5], &data[6], &data[7]])
                    .expect("Failed to write content");
            }
        }

        writer.flush().expect("Failed to flush");
        table_count += 1;
    }
}

fn bowling_scraper(html: &str, file_path: &str) {
    // Parse the HTML document using the scraper crate
    let document = Html::parse_document(html);

    // Define a CSS selector for all tables
    let table_selector = Selector::parse("table.ds-table.ds-table-md.ds-table-auto").expect("Failed to find the table");

    let csv_file_directory = format!("{}/scorecard", file_path);

    if let Err(e) = fs::create_dir_all(&csv_file_directory) {
        eprintln!("Error creating the directory: {:?}", e);
        return;
    }

    let mut table_count = 1;

    for table in document.select(&table_selector) {
        // Define a selector for table rows within the table
        let row_selector = Selector::parse("tbody tr").expect("Failed to create row selector");

        // Create a CSV file for writing the bowling scorecard
        let csv_filename = format!("{}/bowling_scorecard{}.csv", csv_file_directory, table_count);

        let mut writer = Writer::from_path(csv_filename.clone()).expect("Failed to create the CSV file");

        // Write the CSV header
        writer.write_record(&[
            "Bowler_name", "O", "M", "R", "W", "ECON", "0s", "4s", "6s", "WD", "NB",
        ])
        .expect("Failed to write the header");

        // Extract bowler information and write to the CSV
        let mut has_records = false; // Track if there are records in the CSV

        for row in table.select(&row_selector) {
            // Create a selector for table columns within each row
            let column_selector = Selector::parse("td").expect("Failed to create column selector");
            let columns = row.select(&column_selector);

            let data: Vec<String> = columns.map(|column| column.text().collect()).collect();

            // Ensure that the row contains data
            if data.len() == 11 {
                writer.write_record(&data).expect("Failed to write content");
                has_records = true; // Records found in this CSV
            }
        }

        writer.flush().expect("Failed to flush");
        table_count += 1;

        // Remove the CSV file if it doesn't have records
        if !has_records {
            fs::remove_file(csv_filename).expect("Failed to remove empty CSV file");
        }
    }
}

fn venue_name(html: &str) -> String {
    // Parse the HTML content
    let fragment = Html::parse_fragment(html);

    // Define a variable to store the stadium name
    let mut stadium = "".to_string();

    // Iterate through the <a> elements in the HTML
    for a in fragment.select(&Selector::parse("a").unwrap()) {
        if let Some(href) = a.value().attr("href") {
            if href.contains("/cricket-grounds/") {
                // If the <a> element's href contains "/cricket-grounds/", extract the stadium name
                stadium = href.to_string();
                break; // Exit the loop since we found the stadium name
            }
        }
    }

    // Return the stadium name
    stadium
}


fn write_player_info(player_info: &HashSet<String>, file_path: &str) {
    let csv_file_path = format!("{}/player_info.csv", file_path);
    let headers_written = Path::new(&csv_file_path).exists();

    if let Err(e) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&csv_file_path)
    {
        eprintln!("Error creating/opening CSV file: {:?}", e);
        return;
    }

    let mut writer = WriterBuilder::new()
        .has_headers(!headers_written)  // Write headers only if they haven't been written
        .from_path(&csv_file_path)
        .unwrap();

    // Write headers to the CSV file if it's empty
    if !headers_written {
        writer.write_record(&["player_id"]).unwrap();
    }

    for info in player_info {
        let info_parts: &str = info; // Convert the String to &str if needed
        writer.write_record(&[info_parts]).unwrap();
    }

    writer.flush().expect("Failed to flush");
}


fn venue_scapper(html: &str, file_path: &str,file_name:&str){
    let document = Html::parse_document(html);

    let info_selector = Selector::parse(".ds-table tbody tr").expect("Failed to find the venue information");

    let mut toss = String::new();
    let mut series = String::new();
    let mut season = String::new();
    let mut player_of_the_match_href = String::new();
    let mut match_number = String::new();
    let mut match_days = String::new();
    let mut points = String::new();

    for row in document.select(&info_selector) {
        let columns: Vec<String> = row
            .text()
            .collect::<Vec<&str>>()
            .iter()
            .map(|s| s.trim())
            .filter(|&s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if columns.len() >= 2 {
            let key = columns[0].to_lowercase();
            let value = columns[1].to_string();

            match key.as_str() {
                "toss" => toss = value,
                "series" => series = value,
                "season" => season = value,
                "player of the match" => { if let Some(href) = row.select(&Selector::parse("a").unwrap()).next() {
                    player_of_the_match_href = href.value().attr("href").unwrap_or_default().to_string();
                    player_of_the_match_href=player_of_the_match_href.replace("/cricketers/","")
                }
            },
                "match number" => match_number = value,
                "match days" => match_days = value,
                "points" => points = value,
                "series result" => points = value,
                _ => {} // Handle other keys or ignore them
            }
        }
    }

    let csv_file_path = format!("{}/{}.csv", file_path,file_name);
    let mut writer = WriterBuilder::new()
        .has_headers(false)  // Do not write headers since there's only one row
        .from_path(csv_file_path.clone())
        .expect("Failed to create or open CSV file");

    writer.write_record(&[ "Toss", "Series", "Season", "Player_Of_The_Match",
        "Match_number", "Match_date", "Points",
    ]).expect("Failed to write headers");

    writer.write_record(&[
         toss, series, season, player_of_the_match_href,
        match_number, match_days, points,
    ]).expect("Failed to write venue data");

    writer.flush().expect("Failed to flush");

   // println!("Venue information has been written to CSV: {}", csv_file_path);
}


fn main() {
    let config_file_path ="match_url_config.txt";

    //clean_up::clean_folders();
    
    let urls = match fs::read_to_string(config_file_path) {
        Ok(contents) => contents
            .lines()
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>(),
        Err(e) => {
            eprintln!("Error reading match config file: {:?}", e);
            return;
        }
    };

    // Create a reqwest client with a timeout of 10 seconds
    let client = Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
    let csv_file_path = "./Match_details";

    if let Err(e) = fs::create_dir_all(&csv_file_path) {
        eprintln!("Error creating the directory: {:?}", e);
        return;
    }

    // Create a HashSet to keep track of already processed players
    
    let mut interim_player_info: HashSet<String> = HashSet::new();
    
    for url in urls {
        match client.get(&url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.text().unwrap();
                    let document = Document::from(content.as_str());

                   // Regular expression for extracting name and ID
                    let re = Regex::new(r#"/cricketers/([^/-]+(?:-[^/-]+)*)-(\d+)"#).unwrap();

                    let mut processed_players: HashSet<String> = HashSet::new();

                    let parts: Vec<&str> = url.split('/').collect();
                    let csv_file_directory = format!("{}/{}", csv_file_path, &parts[parts.len() - 2]);
                    let csv_file_path_temp = format!("{}/players.csv", csv_file_directory);

                    // Create the directory if it doesn't exist
                    if !Path::new(&csv_file_directory).exists() {
                        fs::create_dir_all(&csv_file_directory).unwrap();
                    }

                    // Check if headers have been written
                    let headers_written = Path::new(&csv_file_path_temp).exists();

                    if let Err(e) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&csv_file_path_temp)
                    {
                        eprintln!("Error creating/opening CSV file: {:?}", e);
                        continue;
                    }

                    // Use WriterBuilder to set headers for the CSV file
                    let mut writer = WriterBuilder::new()
                        .has_headers(!headers_written)  // Write headers only if they haven't been written
                        .from_path(&csv_file_path_temp)
                        .unwrap();

                    // Write headers to the CSV file if it's empty
                    if !headers_written {
                        writer.write_record(&["player name", "player_id", "captain_flag"]).unwrap();
                    }

                    // Iterate over <a> tags with the specified class
                    for node in document.find(Name("a")) {
                        if let Some(href) = node.attr("href") {
                            // Check if the URL contains "/cricketers/"
                            if href.contains("/cricketers/") {
                                // Extract cricketer's name and ID using the regex
                                if let Some(captures) = re.captures(href) {
                                    if let Some(name) = captures.get(1).map(|m| m.as_str()) {
                                        if let Some(id) = captures.get(2).map(|m| m.as_str()) {
                                            // Replace hyphens with an empty string
                                            let stripped_name = name.replace('-', " ");
                                            
                                            // Check if this player is already processed
                                            if processed_players.contains(id) {
                                                continue;
                                            }

                                            // Check if the player is the captain
                                            let is_captain = node.text().contains("(c)");

                                            // Write to CSV
                                            writer.write_record(&[stripped_name, id.to_string(), is_captain.to_string()]).unwrap();

                                            // Add the player ID to the processed set
                                            processed_players.insert(id.to_string());
                                            interim_player_info.insert(id.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    batting_scorecard(&content, &csv_file_directory);
                    bowling_scraper(&content, &csv_file_directory);
                    let stadium_name = venue_name(&content);
                    let cleaned_stadium_name = stadium_name.replace("/cricket-grounds/", "");
                    venue_scapper(&content,&csv_file_directory,&cleaned_stadium_name)
              
                } else {
                    eprintln!(
                        "Error: Failed to fetch the webpage. Status code: {}",
                        response.status()
                    );
                }
            }

            Err(e) => {
                eprintln!("Error fetching data: {:?}", e);
            }
        }
    }
    println!("Match Details Fetching Process completed successfully");
    let interim_folder = "./staging_area";
    if let Err(e) = fs::create_dir_all(&interim_folder) {
        eprintln!("Error creating the interim directory: {:?}", e);
        return;
    }
    println!("stagging File Generated successfully");
    // Call the function to write player information to a CSV file
    write_player_info(&interim_player_info, &interim_folder);

    let _ =connect_duck_db::update_latest_info();
    // println!("Filtering the Latest information based on master data");
    fetch_latest_info::update_master();

}
