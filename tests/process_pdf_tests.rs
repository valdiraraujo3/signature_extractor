use base64::{Engine as _, engine::general_purpose::STANDARD};
use extractous::Extractor;
use rstest::{fixture, rstest};
use signature_extractor::*;
use std::fs;
use std::path::PathBuf;

fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[fixture]
fn sample_pdf_path() -> PathBuf {
    let mut pdf_path = get_project_root();
    pdf_path.push("tests/DOC1.pdf");

    if !pdf_path.exists() {
        panic!(
            "Arquivo de teste 'tests/DOC1.pdf' não encontrado. Por favor, adicione um PDF de exemplo para o teste de integração."
        );
    }
    pdf_path
}

#[fixture]
fn extractor() -> Extractor {
    Extractor::new()
}

#[rstest]
fn test_process_pdf_from_file_with_fixtures(extractor: Extractor, sample_pdf_path: PathBuf) {
    let result = process_pdf_from_file(&sample_pdf_path, &extractor);

    assert!(
        result.is_ok(),
        "Falha ao processar PDF do arquivo: {:?}",
        result.err()
    );

    let events = result.unwrap();
    assert!(
        !events.is_empty(),
        "Nenhum evento de assinatura foi encontrado no PDF de exemplo."
    );
}

#[rstest]
fn test_process_pdf_from_base64_with_fixtures(extractor: Extractor, sample_pdf_path: PathBuf) {
    let pdf_bytes = fs::read(sample_pdf_path).expect("Não foi possível ler o arquivo sample.pdf");
    let pdf_base64 = STANDARD.encode(pdf_bytes);

    let result = process_pdf_from_base64(&pdf_base64, &extractor);

    assert!(
        result.is_ok(),
        "Falha ao processar PDF a partir de Base64: {:?}",
        result.err()
    );

    let events = result.unwrap();
    assert!(
        !events.is_empty(),
        "Nenhum evento de assinatura foi encontrado no PDF de exemplo (Base64)."
    );
}
