use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Erro de I/O: {0}")]
    Io(#[from] io::Error),
    #[error("Dados inválidos: {0}")]
    InvalidData(String),
    #[error("Erro desconhecido.")]
    Unknown,
    #[error("Seção '2. Log's e eventos do processo de assinatura', não encontrada.")]
    EventsLogs,
    #[error("Dados de evento de assinatura não encontrados ou incompletos.")]
    DataNotFound,
    #[error("Erro ao extrair conteúdo do PDF: {0}")]
    PdfExtraction(String),
    #[error("Erro ao analisar eventos de assinatura: {0}")]
    ParseSignature(String),
    #[error("Erro de serialização JSON: {0}")]
    JsonSerialization(#[from] serde_json::Error),
    #[error("Caminho base inválido ou inexistente: {0}")]
    InvalidBasePath(String),
}