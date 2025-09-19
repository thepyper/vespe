import ollama
import onnxruntime
import numpy as np
from transformers import AutoTokenizer
import argparse
import math

# --- CONFIGURATION ---
MODEL_CHECKPOINT = "distilbert-base-uncased"
ONNX_MODEL_PATH = "onnx/model.onnx"
WINDOW_SIZE = 512
STRIDE = 256 # Overlap will be WINDOW_SIZE - STRIDE

# Le etichette devono essere le stesse usate durante il training
LABELS = [
    "O", "B-THOUGHT", "I-THOUGHT", "B-TOOLCALL", "I-TOOLCALL",
    "B-TOOLRESPONSE", "I-TOOLRESPONSE", "B-TEXT", "I-TEXT",
]
id2label = {i: label for i, label in enumerate(LABELS)}

# --- SCRIPT ---

def query_ollama(prompt, model_name, system_prompt):
    """Invia un prompt a Ollama e ottiene una risposta."""
    print(f"\n>>> Querying Ollama model '{model_name}'...")
    
    messages = []
    if system_prompt:
        messages.append({'role': 'system', 'content': system_prompt})
    messages.append({'role': 'user', 'content': prompt})

    # Stampa il payload completo per debugging
    import json
    print(f"--- Ollama Request Payload ---\n{json.dumps(messages, indent=2)}\n----------------------------")

    try:
        response = ollama.chat(model=model_name, messages=messages)
        content = response['message']['content']
        print(f"<<< Ollama response:\n{content}")
        return content
    except Exception as e:
        print(f"\n--- ERROR ---")
        print(f"Could not connect to Ollama or model '{model_name}' not found. Make sure Ollama is running and the model is pulled.")
        print(f"Error details: {e}")
        return None

def run_inference(text):
    """Processa il testo con il modello ONNX, usando una sliding window se necessario."""
    print("\n>>> Loading tokenizer and ONNX session...")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_CHECKPOINT)
    session = onnxruntime.InferenceSession(ONNX_MODEL_PATH)

    print("\n>>> Running inference...")
    # Tokenizza l'intero testo, senza aggiungere token speciali [CLS] o [SEP]
    all_input_ids = tokenizer(text, add_special_tokens=False)["input_ids"]
    
    if len(all_input_ids) <= WINDOW_SIZE:
        print("Input is shorter than window size, running simple inference.")
        # Se il testo è corto, esegui l'inferenza normale (con token speciali)
        inputs = tokenizer(text, return_tensors="np")
        onnx_inputs = {
            "input_ids": inputs["input_ids"].astype(np.int64),
            "attention_mask": inputs["attention_mask"].astype(np.int64)
        }
        logits = session.run(None, onnx_inputs)[0]
        predictions = np.argmax(logits, axis=2)
        return predictions[0], inputs["input_ids"][0]

    print(f"Input is longer than {WINDOW_SIZE} tokens, running sliding window inference...")
    # Inizializza i tensori per accumulare i logits e contare le sovrapposizioni
    total_logits = np.zeros((len(all_input_ids), len(LABELS)), dtype=np.float32)
    overlap_counts = np.zeros(len(all_input_ids), dtype=np.float32)

    # Itera sul testo con la finestra mobile
    for start in range(0, len(all_input_ids), STRIDE):
        end = start + WINDOW_SIZE
        
        # Estrai la finestra di token, aggiungendo [CLS] e [SEP]
        chunk_input_ids = [tokenizer.cls_token_id] + all_input_ids[start:end] + [tokenizer.sep_token_id]
        # Crea l'attention mask (tutti 1 perché non c'è padding)
        chunk_attention_mask = [1] * len(chunk_input_ids)

        # Prepara l'input per il modello
        onnx_inputs = {
            "input_ids": np.array([chunk_input_ids], dtype=np.int64),
            "attention_mask": np.array([chunk_attention_mask], dtype=np.int64)
        }

        # Esegui l'inferenza sulla finestra
        chunk_logits = session.run(None, onnx_inputs)[0][0]
        
        # Aggiungi i logits al totale, ignorando [CLS] e [SEP]
        total_logits[start:end] += chunk_logits[1:-1]
        overlap_counts[start:end] += 1

    # Calcola la media dei logits nelle aree di sovrapposizione
    # Aggiungi una piccola costante per evitare divisioni per zero
    average_logits = total_logits / (overlap_counts[:, np.newaxis] + 1e-9)
    final_predictions = np.argmax(average_logits, axis=1)
    
    # Ricostruisci gli input_ids finali per la visualizzazione
    final_input_ids = [tokenizer.cls_token_id] + all_input_ids + [tokenizer.sep_token_id]
    # Aggiungi predizioni fittizie per [CLS] e [SEP] (verranno ignorate)
    final_predictions_with_special = np.concatenate(([0], final_predictions, [0]))

    return final_predictions_with_special, final_input_ids

def post_process_and_display(predictions, input_ids):
    """Raggruppa i token e le etichette e stampa il risultato."""
    print("\n>>> Parsed Segments:")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_CHECKPOINT)
    tokens = tokenizer.convert_ids_to_tokens(input_ids)
    
    current_segment_type = None
    current_words = []

    for token, pred_id in zip(tokens, predictions):
        if token in tokenizer.all_special_tokens:
            continue

        label = id2label[pred_id]
        word = token.replace("##", "")
        
        # Determina il tipo di segmento (es. THOUGHT, TOOLCALL)
        segment_type = label.split('-')[-1] if label != 'O' else 'O'

        if current_segment_type is None:
            current_segment_type = segment_type

        # Se il tipo di segmento cambia, stampa quello precedente
        if segment_type != current_segment_type and current_segment_type != 'O':
            print(f"  - [{current_segment_type}]: {' '.join(current_words)}")
            current_words = []
        
        current_segment_type = segment_type
        if segment_type != 'O':
            current_words.append(word)

    # Stampa l'ultimo segmento se esiste
    if current_words and current_segment_type != 'O':
        print(f"  - [{current_segment_type}]: {' '.join(current_words)}")

if __name__ == "__main__":
    import os

    parser = argparse.ArgumentParser(description="Test the buzz parser model with an Ollama backend.")
    parser.add_argument(
        "-p", "--prompt",
        type=str,
        required=True,
        help="The user prompt to send to the Ollama model."
    )
    parser.add_argument(
        "-s", "--system-prompt",
        type=str,
        help="An optional system prompt string, or a path to a file containing the system prompt."
    )
    parser.add_argument(
        "-m", "--model",
        type=str,
        default="gemma3:1b",
        help="The name of the Ollama model to use (e.g., 'mistral', 'llama3')."
    )
    args = parser.parse_args()

    system_prompt_content = None
    if args.system_prompt:
        # Controlla se l'argomento è un percorso a un file esistente
        if os.path.isfile(args.system_prompt):
            print(f"Reading system prompt from file: {args.system_prompt}")
            with open(args.system_prompt, 'r', encoding='utf-8') as f:
                system_prompt_content = f.read()
        else:
            # Altrimenti, usalo come stringa letterale
            system_prompt_content = args.system_prompt
    
    # 1. Ottieni la risposta dall'LLM
    ollama_response = query_ollama(args.prompt, model_name=args.model, system_prompt=system_prompt_content)
    
    if ollama_response:
        # 2. Esegui l'inferenza con il nostro modello ONNX
        predictions, input_ids = run_inference(ollama_response)
        
        # 3. Processa e visualizza i risultati
        post_process_and_display(predictions, input_ids)
