use async_nats::jetstream::{self, stream::StorageType};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use dotenvy::dotenv;
use extractous::Extractor;
use serde::Serialize;
use signature_extractor::*;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Debug)]
struct NatsPayload<'a> {
    client_id: Uuid,
    events: &'a [SignatureEvent],
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://192.168.2.107:4222".to_string());
    let nats_subject =
        env::var("NATS_SUBJECT").unwrap_or_else(|_| "pdf.events.started".to_string());
    let stream_name = env::var("JS_STREAM_NAME").unwrap_or_else(|_| "PDF_PROCESSING".to_string());

    println!("Conectando ao servidor NATS em {nats_url}...");
    let nats_client = async_nats::connect(&nats_url).await?;
    println!("Conexão estabelecida com sucesso!");
    let jetstream = jetstream::new(nats_client);

    println!("Criando o stream JetStream '{stream_name}'...");
    let _stream = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: stream_name.clone(),
            subjects: vec![nats_subject.clone()],
            storage: StorageType::File,
            ..Default::default()
        })
        .await?;
    println!(
        "O stream '{stream_name}' está configurado para capturar mensagens do tópico '{nats_subject}'"
    );
    println!("Lendo arquivo PDF...");
    let mut pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/DOC1.pdf");
    let pdf_bytes = fs::read(&pdf_path).expect("Não foi possível ler o arquivo PDF");
    let pdf_b64 = STANDARD.encode(pdf_bytes);
    println!("Arquivo PDF lido e convertido para Base64.");

    println!("Processando PDF...");
    let extractor = Extractor::new();

    match process_pdf_from_base64(&pdf_b64, &extractor) {
        Ok(events) => {
            println!(
                "PDF processado com sucesso! Encontrados {} eventos.",
                events.len()
            );
            let client_id = Uuid::new_v4();
            println!("Gerado client_id: {client_id}");
            println!("Criando payload para o NATS...");
            let nats_payload = NatsPayload {
                client_id,
                events: &events,
            };
            println!("Serializando payload para JSON...");
            let payload_bytes = serde_json::to_vec(&nats_payload)?;
            println!("Payload serializado com sucesso!");
            println!(
                "Publicando no stream JetStream (tópico: '{nats_subject}') com client_id: {client_id}"
            );
            let ack = jetstream
                .publish(nats_subject, payload_bytes.into())
                .await?
                .await?;
            println!("Mensagem publicada com sucesso! Ack: {ack:?}");
            println!("Payload: {nats_payload:?}");
        }
        Err(e) => {
            eprintln!("Erro ao processar o PDF: '{e}'. Nenhuma mensagem foi publicada.");
        }
    }

    Ok(())
}
