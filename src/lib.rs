use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use chrono::NaiveDateTime;
use extractous::Extractor;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub mod error;
use error::AppError;

/// Representa os eventos de assinatura.
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct SignatureEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geolocation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    pub signed_at: NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

/// Analisa o texto completo de um PDF para extrair os eventos de assinatura.
///
/// Procura pela seção "2. Log’s e eventos do processo de assinatura" para processar os dados.
pub fn parse_signature_events(full_text: &str) -> Result<Vec<SignatureEvent>, AppError> {
    const START_MARKER: &str = "2. Log’s e eventos do processo de assinatura:";
    const HEADERS: &str = "Evento: Dados do Dispositivo: Data e hora (UTC -3):";

    let log_section = if let Some((_, section)) = full_text.split_once(START_MARKER) {
        section
    } else {
        return Err(AppError::EventsLogs);
    };

    let clean_section = log_section.replace(HEADERS, "").trim().to_string();
    let date_regex = Regex::new(r"(\d{2}/\d{2}/\d{4}\s\d{2}:\d{2}:\d{2})")
        .map_err(|e| AppError::ParseSignature(format!("Erro ao compilar regex de data: {e}")))?;
    let mut events: Vec<SignatureEvent> = Vec::new();
    let mut current_record_lines: Vec<String> = Vec::new();

    for line in clean_section.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Página") {
            continue;
        }

        if date_regex.is_match(line) && !current_record_lines.is_empty() {
            let record_text = current_record_lines.join("\n");
            if let Some(event) = parse_record_to_details(&record_text) {
                events.push(event);
            }
            current_record_lines.clear();
        }
        current_record_lines.push(line.to_string());
    }

    if !current_record_lines.is_empty() {
        let record_text = current_record_lines.join("\n");
        if let Some(event) = parse_record_to_details(&record_text) {
            events.push(event);
        }
    }

    if events.is_empty() {
        return Err(AppError::DataNotFound);
    }

    Ok(events)
}

/// Função auxiliar para analisar um registro individual de evento.
fn parse_record_to_details(record_text: &str) -> Option<SignatureEvent> {
    let date_regex = Regex::new(r"(\d{2}/\d{2}/\d{4}\s\d{2}:\d{2}:\d{2})").unwrap();
    let date_str = date_regex.captures(record_text)?.get(1)?.as_str();
    let signed_at = NaiveDateTime::parse_from_str(date_str, "%d/%m/%Y %H:%M:%S").ok()?;

    let mut event_description: Vec<String> = Vec::new();
    let mut geolocation = None;
    let mut ip_address = None;
    let mut user_agent_parts: Vec<String> = Vec::new();
    let mut is_parsing_ua = false;

    for line in record_text.lines() {
        let clean_line = date_regex.replace(line, "").to_string();
        let trimmed_line = clean_line.trim();

        if trimmed_line.is_empty() {
            continue;
        }

        if let Some(val) = trimmed_line.strip_prefix("Dispositivo:") {
            user_agent_parts.push(val.trim().to_string());
            is_parsing_ua = true;
        } else if let Some(val) = trimmed_line.strip_prefix("Geolocalização (DD):") {
            geolocation = Some(val.trim().to_string());
            is_parsing_ua = false;
        } else if let Some(val) = trimmed_line.strip_prefix("IP de acesso:") {
            ip_address = Some(val.trim().to_string());
            is_parsing_ua = false;
        } else if let Some(val) = trimmed_line.strip_prefix("Sistema Unico") {
            user_agent_parts.push(val.trim().to_string());
            is_parsing_ua = true;
        } else if !trimmed_line.starts_with("Porta lógica:") {
            if is_parsing_ua {
                user_agent_parts.push(trimmed_line.to_string());
            } else {
                event_description.push(trimmed_line.to_string());
            }
        }
    }

    let final_user_agent = if user_agent_parts.is_empty() {
        None
    } else {
        Some(user_agent_parts.join("\n"))
    };

    if geolocation.is_none() || ip_address.is_none() || final_user_agent.is_none() {
        return None;
    }

    Some(SignatureEvent {
        geolocation,
        ip_address,
        signed_at,
        user_agent: final_user_agent,
    })
}

/// Processa um único arquivo PDF para extrair eventos de assinatura.
///
/// Retorna `Result<Vec<SignatureEvent>, AppError>` com os eventos encontrados
/// ou um erro se a extração ou análise falhar.
pub fn process_pdf_from_file(
    pdf_path: &Path,
    extractor: &Extractor,
) -> Result<Vec<SignatureEvent>, AppError> {
    let (content, _metadata) = extractor
        .extract_file_to_string(pdf_path.to_str().ok_or_else(|| {
            AppError::InvalidBasePath(format!("Caminho inválido: {}", pdf_path.display()))
        })?)
        .map_err(|e| AppError::PdfExtraction(format!("Erro do extrator: {e}")))?;

    let events = parse_signature_events(&content)?;
    Ok(events)
}

/// Processa um PDF a partir de uma string Base64 para extrair eventos de assinatura.
///
/// Retorna `Result<Vec<SignatureEvent>, AppError>` com os eventos encontrados
/// ou um erro se a decodificação, extração ou análise falhar.
pub fn process_pdf_from_base64(
    pdf_base64_string: &str,
    extractor: &Extractor,
) -> Result<Vec<SignatureEvent>, AppError> {
    // let pdf_bytes: Vec<u8> = base64::decode(pdf_base64_string)
    //     .map_err(|e| AppError::InvalidData(format!("Erro ao decodificar Base64: {e}")))?;
    let pdf_bytes = STANDARD
        .decode(pdf_base64_string) // <--- MUDANÇA AQUI
        .map_err(|e| AppError::InvalidData(format!("Erro ao decodificar Base64: {e}")))?;

    let mut temp_pdf_file = NamedTempFile::new().map_err(AppError::Io)?;
    temp_pdf_file.write_all(&pdf_bytes).map_err(AppError::Io)?;
    let temp_path = temp_pdf_file.path();

    let (content, _metadata) = extractor
        .extract_file_to_string(temp_path.to_str().ok_or_else(|| {
            AppError::InvalidBasePath(format!(
                "Caminho temporário inválido: {}",
                temp_path.display()
            ))
        })?)
        .map_err(|e| AppError::PdfExtraction(format!("Erro do extrator: {e}")))?;
    let events = parse_signature_events(&content)?;
    Ok(events)
}

/// Lista arquivos PDF em um diretório especificado.
///
/// Retorna um `Result` contendo um vetor de `PathBuf` para os arquivos PDF encontrados,
/// ou um `AppError::Io` se houver problemas de I/O.
pub fn list_pdf_files(caminho_da_pasta: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut arquivos_pdf = Vec::new();
    for entrada in fs::read_dir(caminho_da_pasta)? {
        let entrada = entrada?;
        let path = entrada.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "pdf") {
            arquivos_pdf.push(path);
        }
    }
    Ok(arquivos_pdf)
}

/// Função principal para processar múltiplos PDFs em um diretório.
///
/// Recebe o caminho base do diretório contendo PDFs.
/// Itera sobre os PDFs, extrai o conteúdo e analisa os eventos de assinatura.
/// Imprime os eventos JSON ou erros.
pub fn run_processing(base_dir_str: &str) -> Result<(), AppError> {
    let base_path = PathBuf::from(base_dir_str);

    if !base_path.exists() || !base_path.is_dir() {
        return Err(AppError::InvalidBasePath(format!(
            "O diretório base '{base_dir_str}' não existe ou não é um diretório válido."
        )));
    }
    let extractor = Extractor::new();

    let pdf_paths = list_pdf_files(&base_path)?;

    if !pdf_paths.is_empty() {
        println!("Arquivos PDF encontrados: {}", pdf_paths.len());
        for pdf_path in pdf_paths {
            println!("\nProcessando arquivo: {}", pdf_path.display());

            match process_pdf_from_file(&pdf_path, &extractor) {
                Ok(events) => {
                    let json_output = serde_json::to_string_pretty(&events)
                        .map_err(AppError::JsonSerialization)?;
                    println!("{json_output}");
                }
                Err(e) => {
                    eprintln!("Erro ao processar o PDF '{}': {}", pdf_path.display(), e);
                }
            }
        }
    } else {
        println!("Nenhum arquivo PDF encontrado no diretório: {base_dir_str}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rstest::{fixture, rstest};

    const FAKE_PDF_CONTENT_SUCCESS: &str = r#"
        2. Log's e eventos do processo de assinatura:

        Evento: Dados do Dispositivo: Data e hora (UTC -3):

        O documento 98edc2bc-2063-466f-
        855e-7f3f5202a046 foi criado e enviado

        IP de acesso: 186.251.21.12

        Porta lógica:
        16/07/2025 15:10:19

        O signatário QUEM ASSINOU
        abriu o envelope.

        Dispositivo:Mozilla/5.0 (X11; Linux
        x86_64) AppleWebKit/537.36
        (KHTML, like Gecko)
        Chrome/138.0.0.0 Safari/537.36

        Geolocalização (DD): 0,0

        IP de acesso: 186.251.21.12

        16/07/2025 15:11:39

        O signatário QUEM ASSINOU
        visualizou o documento.

        Dispositivo:Mozilla/5.0 (X11; Linux
        x86_64) AppleWebKit/537.36
        (KHTML, like Gecko)
        Chrome/138.0.0.0 Safari/537.36

        Geolocalização (DD): 0,0

        IP de acesso: 186.251.21.12

        16/07/2025 15:11:40

        O signatário QUEM ASSINOU leu
        e concordou com o documento.

        Dispositivo:Mozilla/5.0 (X11; Linux
        x86_64) AppleWebKit/537.36
        (KHTML, like Gecko)
        Chrome/138.0.0.0 Safari/537.36

        Geolocalização (DD): 0,0

        IP de acesso: 186.251.21.12

        16/07/2025 15:11:42

        O processo de assinatura do signatário
        QUEM ASSINOU foi finalizado

        Dispositivo:Mozilla/5.0 (X11; Linux
        x86_64) AppleWebKit/537.36
        (KHTML, like Gecko)
        Chrome/138.0.0.0 Safari/537.36

        Geolocalização (DD): 0,0

        16/07/2025 15:11:44
    "#;

    const FAKE_PDF_CONTENT_NO_MARKER: &str = "O processo de assinatura do signatário.";
    const FAKE_PDF_CONTENT_NO_DATA: &str = "2. Log's e eventos do processo de assinatura:";

    #[rstest]
    #[case("success", FAKE_PDF_CONTENT_SUCCESS)]
    #[case("no marker", FAKE_PDF_CONTENT_NO_MARKER)]
    #[case("no data", FAKE_PDF_CONTENT_NO_DATA)]
    fn test_parse_signature_events_parameterized(#[case] description: &str, #[case] input: &str) {
        let result = parse_signature_events(input);

        match description {
            "success" => {
                let events = result.expect("Deveria ter tido sucesso na extração");
                assert_eq!(events.len(), 3);
                assert_eq!(events[0].ip_address, Some("186.251.21.12".to_string()));
                assert_eq!(events[1].geolocation, Some("0,0".to_string()));
            }
            "no marker" => {
                assert!(matches!(result, Err(AppError::EventsLogs)));
            }
            "no data" => {
                assert!(matches!(result, Err(AppError::DataNotFound)));
            }
            _ => panic!("Caso de teste desconhecido: {description}"),
        }
    }

    #[fixture]
    fn valid_signature_event() -> SignatureEvent {
        SignatureEvent {
            geolocation: Some("0,0".to_string()),
            ip_address: Some("186.251.21.12".to_string()),
            signed_at: NaiveDate::from_ymd_opt(2024, 7, 19)
                .unwrap()
                .and_hms_opt(16, 50, 1)
                .unwrap(),
            user_agent: Some("Sistema Unico de Processo Eletronico em Santa Catarina".to_string()),
        }
    }

    #[rstest]
    fn test_parse_record_to_details_with_fixture(valid_signature_event: SignatureEvent) {
        let record_text = r#"
            Assinatura em lote realizada
            Dispositivo: Sistema Unico de Processo Eletronico em Santa Catarina
            Geolocalização (DD): 0,0
            IP de acesso: 186.251.21.12
            19/07/2024 16:50:01
    "#;

        let result = parse_record_to_details(record_text);

        assert_eq!(result, Some(valid_signature_event));
    }
}
