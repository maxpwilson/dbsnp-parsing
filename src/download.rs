use std::path::Path;
use std::fs::{File, remove_file};
use std::io::{BufReader, Read, Write};
use reqwest::blocking::Client;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Response;

pub struct Download{
    pub filename: String,
    pub server: String,
    pub localpath: String
}
impl Download {
    pub fn new(filename: String, server:String, localpath:String) -> Download {
        Download { filename: filename, server: server, localpath: localpath }
    }
    pub fn localfile(&self) -> Option<String> {
        Some(self.localpath.clone() + &self.filename)
    }
    pub fn serverfile(&self) -> Option<String> {
        Some(self.server.clone() + &self.filename)
    }
    pub fn is_local(&self) -> Option<bool> {
        Some(Path::new(&self.localfile().unwrap()).exists())
    }
    pub fn remove_local(&self) -> Result<()> {
        if self.is_local().unwrap() {
            remove_file(self.localfile().unwrap())?;
        }
        Ok(())
    }
    fn get_response(&self) -> Result<Response> {
        let client = Client::new();
        let response = client.get(self.serverfile().unwrap()).send().context("Failed to get a response")?;
        Ok(response)
    }
    pub fn get_text(&self) -> Result<String> {
        let response = self.get_response()?;
        let text = response.text()?;
        Ok(text)
    }
    pub fn download(&self) -> Result<()> {
        let response = self.get_response()?;
        let mut file = File::create(self.localfile().unwrap())?;
        let content = response.bytes()?;
        file.write_all(&content)?;
        Ok(())
    }
    pub fn verbose_download(&self) -> Result<()> {
        let response = self.get_response()?;
        let download_size= response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(download_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        let mut file = File::create(self.localfile().unwrap())?;
        let mut source = BufReader::new(response);
        let mut buffer = [0u8; 8192];
        let mut download_counter = 0;

        while download_counter < download_size {
            let bytes_read = source.read(&mut buffer)?;
            file.write_all(&buffer[..bytes_read])?;
            download_counter += bytes_read as u64;
            pb.set_position(download_counter);
        }
        pb.finish_with_message("Download Complete");
        Ok(())
    }
}
