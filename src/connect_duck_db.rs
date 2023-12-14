use duckdb::{params, Connection};

pub fn update_latest_info() {
    let conn_result = Connection::open_in_memory();
    let conn = conn_result.expect("Failed to connect to DuckDB");

    let _ = conn.execute(
        "
        CREATE TABLE MASTER_DATA (
            ID STRING,
            First_name STRING,
            Last_name STRING,
            DOB STRING,
            Playing_type STRING,
            Country_code STRING
        )
        ",
        params![],
    ).expect("failed in creating the master data table");

    let _ = conn.execute(
        "
        COPY MASTER_DATA FROM './Master_data/master_data.csv' WITH (FORMAT 'csv', DELIMITER ',');
        ",
        params![],
    ).expect("insert failed");
    

    let __ = conn.execute(
        "
        CREATE TABLE STAGING_DATA (
            ID STRING
        );",
        params![],
    ).expect("failed in creating the staging_data table");

    let _ = conn.execute(
        "
        COPY STAGING_DATA FROM './staging_area/player_info.csv' WITH (FORMAT 'csv', DELIMITER ',');
        ",
        params![],
    ).expect("insert failed");
    
    let _ = conn.execute("COPY ( SELECT STAGING_DATA.ID
    FROM STAGING_DATA
    LEFT JOIN MASTER_DATA ON STAGING_DATA.ID = MASTER_DATA.ID
    WHERE MASTER_DATA.ID IS NULL) TO './intrim/latest.csv' (DELIMITER ',');",params![]).expect("msg");
            
    }
