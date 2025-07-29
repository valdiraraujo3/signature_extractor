use extractous::Extractor;
use signature_extractor::error::AppError;
use signature_extractor::*;

fn main() -> Result<(), AppError> {
    let pdf_b64: &'static str = "";

    let extractor = Extractor::new();
    match process_pdf_from_base64(pdf_b64, &extractor) {
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
