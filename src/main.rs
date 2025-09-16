use signature_extractor::run_processing;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <caminho_para_pasta_de_pdfs>", args[0]);
        std::process::exit(1);
    }

    let dir_path = &args[1];
    println!("Iniciando processamento no diretório: {}", dir_path);

    if let Err(e) = run_processing(dir_path) {
        eprintln!("Ocorreu um erro na aplicação: {}", e);
        std::process::exit(1);
    }

    println!("\nProcessamento concluído com sucesso.");
}
