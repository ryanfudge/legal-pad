use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use hnsw_rs::prelude::*;
use hnsw_rs::dist::DistCosine;
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
    index: std::cell::RefCell<Hnsw<f32, DistCosine>>,
    notes: Vec<NoteEmbedding>,
}

impl SemanticSearch {
    pub fn new() -> Result<Self, SearchError> {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
            .create_model()
            .map_err(SearchError::Model)?;

        let index = Hnsw::<f32, DistCosine>::new(
            16,   // max number of connections per layer
            200,  // max number of elements
            16,   // max layer
            EF_CONSTRUCTION,  // ef construction
            DistCosine{},
        );

        let notes = Self::load_embeddings()?;
        
        // Add existing embeddings to the index
        let index = std::cell::RefCell::new(index);
        for (i, note) in notes.iter().enumerate() {
            index.borrow_mut().insert((&note.embedding, i));
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
        let mut notes: Vec<NoteEmbedding> = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        for note in &mut notes {
            let embedding = std::mem::take(&mut note.embedding);
            note.embedding = normalize_embedding(embedding);
        }
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
        let embedding = normalize_embedding(embedding);
        
        let note = NoteEmbedding {
            text: text.to_string(),
            embedding: embedding.clone(),
        };
        
        // Add to index
        let index = self.notes.len();
        self.index.borrow_mut().insert((&embedding, index));
        
        // Add to notes
        self.notes.push(note);
        
        // Save embeddings
        self.save_embeddings()?;
        Ok(())
    }

    pub fn search(&self, query: &str, k: usize) -> Result<Vec<(String, f32)>, SearchError> {
        let query_embedding = self.model.encode(&[query])
            .map_err(SearchError::Model)?[0].to_vec();
        let query_embedding = normalize_embedding(query_embedding);
        
        let neighbors = self.index.borrow().search(&query_embedding, k, EF_CONSTRUCTION);
        
        let results: Vec<(String, f32)> = neighbors
            .into_iter()
            .map(|n| (self.notes[n.d_id].text.clone(), n.distance))
            .collect();
            
        Ok(results)
    }

    pub fn remove_note_text(&mut self, text: &str) -> Result<(), SearchError> {
        let original_len = self.notes.len();
        self.notes.retain(|note| note.text != text);
        if self.notes.len() == original_len {
            return Ok(());
        }

        self.rebuild_index();
        self.save_embeddings()?;
        Ok(())
    }

    fn rebuild_index(&mut self) {
        let max_elements = self.notes.len().max(200);
        let mut index = Hnsw::<f32, DistCosine>::new(
            16,
            max_elements,
            16,
            EF_CONSTRUCTION,
            DistCosine{},
        );
        for (i, note) in self.notes.iter().enumerate() {
            index.insert((&note.embedding, i));
        }
        self.index = std::cell::RefCell::new(index);
    }
}

fn normalize_embedding(mut embedding: Vec<f32>) -> Vec<f32> {
    let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut embedding {
            *value /= norm;
        }
    }
    embedding
}
