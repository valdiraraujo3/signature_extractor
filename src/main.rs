use base64::{Engine as _, engine::general_purpose::STANDARD};
use extractous::Extractor;
//use signature_extractor::error::AppError;
use signature_extractor::*;
use std::fs;
use std::path::PathBuf;

fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn main() {
    let mut pdf_path = get_project_root();
    pdf_path.push("tests/DOC1.pdf");

    let pdf_bytes = fs::read(pdf_path).expect("Não foi possível ler o arquivo DOC1.pdf");
    let pdf_b64 = STANDARD.encode(pdf_bytes);

    let extractor = Extractor::new();

    let data = process_pdf_from_base64(&pdf_b64, &extractor);
    println!("{data:?}");
}

/*
fn main() -> Result<(), AppError> {
    let mut pdf_path = get_project_root();
    pdf_path.push("tests/DOC1.pdf");

    let pdf_bytes = fs::read(pdf_path).expect("Não foi possível ler o arquivo sample.pdf");
    let pdf_b64 = STANDARD.encode(pdf_bytes);

    let extractor = Extractor::new();

    match process_pdf_from_base64(&pdf_b64, &extractor) {
        Ok(events) => {
            let json_output =
                serde_json::to_string_pretty(&events).map_err(AppError::JsonSerialization)?;
            println!("{json_output}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Erro ao processar o PDF base64 '{e}'");
            Ok(())
        }
    }
}
*/
