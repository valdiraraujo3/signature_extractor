PDF Event Publisher 🚀

Este projeto é um utilitário que lê um arquivo PDF localmente, extrai eventos de assinatura e publica essas informações em um stream JetStream do NATS. 
Ele serve como um publicador de eventos, ideal para ser usado em um pipeline de processamento de documentos.

Como Funciona ⚙️
O programa se conecta a um servidor NATS.

Cria ou obtém um stream JetStream com um tópico (pdf.events.started).

Lê um arquivo PDF, converte seu conteúdo para Base64.

Processa o conteúdo do PDF para extrair eventos de assinatura.

Gera um client_id único para a solicitação.

Publica um payload JSON contendo o client_id e os eventos extraídos no stream JetStream.

Tecnologias Utilizadas 🛠️
- Linguagem: Rust
- Streaming de Mensagens: async-nats (JetStream)
- Processamento de PDF: extractous (biblioteca para extração de dados)
- Serialização: serde e serde_json
- Outras Dependências: tokio, dotenvy, uuid, base64, anyhow

Instalação e Execução ⚡

Pré-requisitos
- Rust e Cargo
- Um servidor NATS em execução (com JetStream habilitado).

Configuração
Clone o repositório:

Bash
```bash
git clone https://github.com/valdiraraujo3/signature_extractor.git

cd signature_extractor
```
Configure o ambiente:
Crie um arquivo .env na raiz do projeto com as seguintes variáveis, se necessário:

- NATS_URL=nats://localhost:4222
- NATS_SUBJECT=pdf.events.started
- JS_STREAM_NAME=PDF_PROCESSING

O programa usará valores padrão se essas variáveis não forem definidas.

Execute o projeto:

Bash
```bash
cargo run
```
Ao rodar o comando, você verá o output no terminal indicando a conexão com o NATS, a criação do stream e a publicação da mensagem, incluindo a confirmação do JetStream.
