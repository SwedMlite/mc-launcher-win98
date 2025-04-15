use std::path::Path;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use crate::models::Profile;

pub fn read_profiles(path: &Path) -> Result<Vec<Profile>, Box<dyn std::error::Error>> {
    if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let profiles: Vec<Profile> = serde_json::from_reader(reader)?;
        Ok(profiles)
    } else {
        Ok(Vec::new())
    }
}

pub fn write_profiles(path: &Path, profiles: &[Profile]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, profiles)?;
    Ok(())
} 