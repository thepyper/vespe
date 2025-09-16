use llm::{
    builder::{FunctionBuilder, LLMBackend, LLMBuilder, ParamBuilder},
    chat::ChatMessage, // Chat-related structures
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use tokio;

// Definizione del parametro per il tool read_file
#[derive(Debug, Serialize, Deserialize)]
struct ReadFileParams {
    file_path: String,
}

// Definizione della risposta del tool
#[derive(Debug, Serialize, Deserialize)]
struct ReadFileResult {
    success: bool,
    content: Option<String>,
    error: Option<String>,
}

// Implementazione del tool read_file
async fn execute_read_file_tool(params: ReadFileParams) -> ReadFileResult {
    match fs::read_to_string(&params.file_path) {
        Ok(content) => ReadFileResult {
            success: true,
            content: Some(content),
            error: None,
        },
        Err(e) => ReadFileResult {
            success: false,
            content: None,
            error: Some(format!("Errore nella lettura del file: {}", e)),
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurazione del provider LLM (usando OpenAI come esempio)
    let client = ChatBuilder::new()
          .backend(LLMBackend::Ollama) // Use OpenAI as the LLM provider
        .base_url(base_url) // Set the Ollama server URL
        .model("llama3.1:8b")
      .temperature(0.7)
        .max_tokens(1000)
        .build()?;

    // Definizione dei tools disponibili
    let mut tools = HashMap::new();
    
    // Tool read_file
    let read_file_tool = Tool {
        name: "read_file".to_string(),
        description: "Legge il contenuto di un file dal filesystem".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Il percorso del file da leggere"
                }
            },
            "required": ["file_path"]
        }),
    };
    
    tools.insert("read_file".to_string(), read_file_tool);

    // Messaggio iniziale dell'utente
    let initial_prompt = "Puoi leggere il file 'example.txt' e dirmi cosa contiene?";
    
    let mut messages = vec![
        Message::user(initial_prompt),
    ];

    loop {
        // Invio della richiesta al modello con i tools
        let response = client
            .chat()
            .messages(&messages)
            .tools(tools.values().cloned().collect())
            .send()
            .await?;

        println!("Risposta del modello: {:?}", response.content);

        // Verifica se il modello ha richiesto l'uso di tools
        if let Some(tool_calls) = &response.tool_calls {
            // Aggiungi la risposta del modello ai messaggi
            messages.push(Message::assistant(&response.content)
                .tool_calls(tool_calls.clone()));

            // Elabora ogni tool call
            for tool_call in tool_calls {
                match tool_call.function.name.as_str() {
                    "read_file" => {
                        println!("Eseguendo tool: read_file con parametri: {}", 
                               tool_call.function.arguments);

                        // Parse dei parametri
                        let params: ReadFileParams = 
                            serde_json::from_str(&tool_call.function.arguments)?;
                        
                        // Esecuzione del tool
                        let result = execute_read_file_tool(params).await;
                        
                        // Preparazione del messaggio di feedback per il modello
                        let tool_result = if result.success {
                            format!("File letto con successo:\n{}", 
                                  result.content.unwrap_or_default())
                        } else {
                            format!("Errore: {}", result.error.unwrap_or_default())
                        };

                        println!("Risultato del tool: {}", tool_result);

                        // Aggiunta del risultato del tool ai messaggi per il feedback
                        messages.push(Message::tool(
                            &tool_call.id,
                            &tool_result
                        ));
                    }
                    _ => {
                        // Tool non riconosciuto
                        let error_msg = format!("Tool '{}' non riconosciuto", 
                                               tool_call.function.name);
                        println!("Errore: {}", error_msg);
                        
                        messages.push(Message::tool(
                            &tool_call.id,
                            &error_msg
                        ));
                    }
                }
            }

            // Continua il loop per ottenere la risposta finale dal modello
            // dopo aver fornito i risultati dei tools
            continue;
        } else {
            // Nessun tool call, la conversazione è terminata
            println!("Conversazione terminata. Risposta finale: {}", response.content);
            break;
        }
    }

    Ok(())
}

// Versione alternativa con gestione più avanzata del feedback
#[allow(dead_code)]
async fn advanced_tool_calling_example() -> Result<(), Box<dyn std::error::Error>> {
    let client = ChatBuilder::new()
        .provider(Provider::OpenAI)
        .model("gpt-4")
        .build()?;

    let mut conversation = Conversation::new();
    
    // Aggiungi messaggio iniziale
    conversation.add_message(Message::user(
        "Leggi il file 'config.json' e analizza la sua struttura"
    ));

    // Tool definitions
    let tools = vec![
        Tool {
            name: "read_file".to_string(),
            description: "Legge e restituisce il contenuto di un file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Percorso completo del file da leggere"
                    }
                },
                "required": ["file_path"]
            }),
        }
    ];

    let max_iterations = 5;
    let mut iteration = 0;

    while iteration < max_iterations {
        iteration += 1;
        
        let response = client
            .chat()
            .conversation(&conversation)
            .tools(tools.clone())
            .send()
            .await?;

        // Aggiungi la risposta dell'assistente
        if let Some(tool_calls) = &response.tool_calls {
            conversation.add_message(
                Message::assistant(&response.content)
                    .tool_calls(tool_calls.clone())
            );

            // Processa tool calls
            for tool_call in tool_calls {
                let result = match tool_call.function.name.as_str() {
                    "read_file" => {
                        let params: ReadFileParams = 
                            serde_json::from_str(&tool_call.function.arguments)?;
                        
                        let file_result = execute_read_file_tool(params).await;
                        
                        if file_result.success {
                            file_result.content.unwrap_or_default()
                        } else {
                            format!("ERRORE: {}", file_result.error.unwrap_or_default())
                        }
                    }
                    _ => format!("Tool '{}' non disponibile", tool_call.function.name)
                };

                // Feedback al modello
                conversation.add_message(Message::tool(&tool_call.id, &result));
            }
        } else {
            // Nessun tool call, conversazione completata
            println!("Risposta finale: {}", response.content);
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_read_file_tool() {
        // Crea un file temporaneo per il test
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Contenuto di test").unwrap();

        // Test del tool
        let params = ReadFileParams {
            file_path: file_path.to_string_lossy().to_string(),
        };

        let result = execute_read_file_tool(params).await;
        
        assert!(result.success);
        assert!(result.content.is_some());
        assert!(result.content.unwrap().contains("Contenuto di test"));
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let params = ReadFileParams {
            file_path: "/path/che/non/esiste.txt".to_string(),
        };

        let result = execute_read_file_tool(params).await;
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}