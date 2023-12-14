# Web scrapper
The web scraper was created entirely using the RUST programming language. This tool is designed to extract data from one of the most popular sports websites, ESPNCricinfo.com. With this project, you can scrape cricket data from any format, like IPL, T20, ODI, and Test available on ESPNCricinfo. All you have to do is provide the scorecard URL in the match_config_url.txt file. If you want, you can provide more than one scorecard URL in the config file.

# Software Requriments and Installation URL's
1. Rust https://www.rust-lang.org/tools/install
3. Duckdb https://duckdb.org/

# Process Flow
![test](https://github.com/Gopi-Krishna-Patapanchala/web_scrapper/assets/135157984/3a173686-d3d4-4f66-93ac-a07e574daa58)

# Execution Porcess

clone the project 
```
git clone https://github.com/Gopi-Krishna-Patapanchala/web_scrapper.git
```
Navigate into the clone project directory
```
cd web_scrapper
```
Now build the Cargo project using the below command
```
cargo build
```
Now everything was set to go. you need to place the scorecard URL's in the config file and execute the project using the below command
```
cargo run
```
you will see the Match_details folder with match wise subfolders in it and Master_data folder with all players information.
