use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use hnsw_rs::prelude::*;
use hnsw_rs::dist::DistL2;
use std::path::PathBuf;
use dirs::home_dir;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io;
use thiserror::Error;

const EMBEDDINGS_FILE: &str = "embeddings.json";
const EF_CONSTRUCTION: usize = 200;  // Higher values give better accuracy but slower construction

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Model error: {0}")]
    Model(#[from] rust_bert::RustBertError),
}

#[derive(Serialize, Deserialize)]
struct NoteEmbedding {
    text: String,
    embedding: Vec<f32>,
}

pub struct SemanticSearch {
    model: rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel,
    index: Hnsw<f32, DistL2>,
    notes: Vec<NoteEmbedding>,
}

impl SemanticSearch {
    pub fn new() -> Result<Self, SearchError> {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
            .create_model()
            .map_err(SearchError::Model)?;

        let index = Hnsw::<f32, DistL2>::new(
            16,   // max number of connections per layer
            200,  // max number of elements
            16,   // max layer
            EF_CONSTRUCTION,  // ef construction
            DistL2{},
        );

        let notes = Self::load_embeddings()?;
        
        // Add existing embeddings to the index
        let mut index = index;
        for (i, note) in notes.iter().enumerate() {
            index.insert((&note.embedding, i));
        }

        Ok(Self { model, index, notes })
    }

    fn get_data_path() -> PathBuf {
        let mut path = home_dir().expect("Could not find home directory");
        path.push("notes");
        fs::create_dir_all(&path).expect("Failed to create notes directory");
        path
    }

    fn load_embeddings() -> Result<Vec<NoteEmbedding>, SearchError> {
        let mut path = Self::get_data_path();
        path.push(EMBEDDINGS_FILE);
        
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(path)?;
        let notes: Vec<NoteEmbedding> = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(notes)
    }

    fn save_embeddings(&self) -> Result<(), SearchError> {
        let mut path = Self::get_data_path();
        path.push(EMBEDDINGS_FILE);
        
        let content = serde_json::to_string(&self.notes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_note(&mut self, text: &str) -> Result<(), SearchError> {
        let embedding = self.model.encode(&[text])
            .map_err(SearchError::Model)?[0].to_vec();
        
        let note = NoteEmbedding {
            text: text.to_string(),
            embedding: embedding.clone(),
        };
        
        // Add to index
        let index = self.notes.len();
        self.index.insert((&embedding, index));
        
        // Add to notes
        self.notes.push(note);
        
        // Save embeddings
        self.save_embeddings()?;
        Ok(())
    }

    pub fn search(&self, query: &str, k: usize) -> Result<Vec<(String, f32)>, SearchError> {
        let query_embedding = self.model.encode(&[query])
            .map_err(SearchError::Model)?[0].to_vec();
        
        let neighbors = self.index.search(&query_embedding, k, EF_CONSTRUCTION);
        
        let results: Vec<(String, f32)> = neighbors
            .into_iter()
            .map(|n| (self.notes[n.d_id].text.clone(), n.distance))
            .collect();
            
        Ok(results)
    }
}