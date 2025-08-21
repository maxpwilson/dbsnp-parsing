#![allow(dead_code)]

use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use reqwest::Response;
use std::fs::{File, remove_file};
use std::io::Write;
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub struct Download {
    pub filename: String,
    pub server: String,
    pub localpath: String,
}
impl Download {
    pub fn new(filename: String, server: String, localpath: String) -> Download {
        Download {
            filename: filename,
            server: server,
            localpath: localpath,
        }
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
    async fn get_response(&self) -> Result<Response> {
        let client = Client::new();
        let response = client
            .get(self.serverfile().unwrap())
            .send()
            .await
            .context("Failed to get a response")?;
        Ok(response)
    }
    pub async fn get_text(&self) -> Result<String> {
        let response = self.get_response().await?;
        let text = response.text().await?;
        Ok(text)
    }
    pub async fn download(&self) -> Result<()> {
        let response = self.get_response().await?;
        let mut file = File::create(self.localfile().unwrap())?;
        let content = response.bytes().await?;
        file.write_all(&content)?;
        Ok(())
    }
    pub async fn verbose_download(&self, multipb: Option<MultiProgress>) -> Result<()> {
        let mut response = self.get_response().await?;
        let download_size = response.content_length().unwrap_or(0);
        let pb = match multipb {
            Some(p) => p.add(ProgressBar::new(download_size)),
            None => ProgressBar::new(download_size),
        };
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        let mut download_counter = 0;
        let file = tokio::fs::File::create(self.localfile().unwrap()).await?;
        let mut file = tokio::io::BufWriter::new(file);

        while let Some(chunk) = response.chunk().await? {
            file.write(&chunk).await?;
            download_counter += chunk.len();
            pb.inc(download_counter as u64);
        }
        pb.finish_with_message("Finished downloading");
        Ok(())
    }
}
