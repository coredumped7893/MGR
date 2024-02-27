use std::fs::File;

use log::debug;
use osmpbfreader::OsmPbfReader;

#[derive(Debug, Clone, Copy)]
pub enum DataFetcher {
    File,
    // Http,
    // S3,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Json,
    // Csv,
    // Binary
}

#[derive(Debug, Clone, Copy)]
pub struct Loader {
    fetch_strategy: DataFetcher,
    data_output_format: OutputFormat,
    merge_files: bool,
}

const INPUT_FILE: &str = "data/map_m_slodowiec.pbf";
// const INPUT_FILE: &str = "data/map_waw_1.pbf";

/*
Load file and convert it to rust structures
 */
impl Loader {
    pub fn new(fetch_strategy: DataFetcher, data_output_format: OutputFormat, merge_files: bool) -> Self {
        Loader {
            fetch_strategy,
            data_output_format,
            merge_files,
        }
    }

    pub fn load(&self) -> OsmPbfReader<File> {
        debug!("Loading data from {:?}", self.fetch_strategy);

        let path = std::path::Path::new(&INPUT_FILE);
        match File::open(path){
            Ok(file_handle) => {
                osmpbfreader::OsmPbfReader::new(file_handle)       
            }
            Err(_) => {
                panic!("File not found");
            }
        }
    }
}

