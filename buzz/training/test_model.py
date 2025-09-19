import ollama
import onnxruntime
import numpy as np
from transformers import AutoTokenizer
import argparse

# --- CONFIGURATION ---
MODEL_CHECKPOINT = "distilbert-base-uncased"
ONNX_MODEL_PATH = "onnx/model.onnx"

# Le etichette devono essere le stesse usate durante il training
LABELS = [
    "O", "B-THOUGHT", "I-THOUGHT", "B-TOOLCALL", "I-TOOLCALL",
    "B-TOOLRESPONSE", "I-TOOLRESPONSE", "B-TEXT", "I-TEXT",
]
id2label = {i: label for i, label in enumerate(LABELS)}

# --- SCRIPT ---

def query_ollama(prompt, model_name):
    """Invia un prompt a Ollama e ottiene una risposta."""
    print(f"\n>>> Querying Ollama model '{model_name}' with prompt: '{prompt}'")
    try:
        response = ollama.chat(
            model=model_name,
            messages=[{'role': 'user', 'content': prompt}],
        )
        content = response['message']['content']
        print(f"<<< Ollama response:\n{content}")
        return content
    except Exception as e:
        print(f"\n--- ERROR ---")
        print(f"Could not connect to Ollama or model '{model_name}' not found. Make sure Ollama is running and the model is pulled.")
        print(f"Error details: {e}")
        return None

def run_inference(text):
    """Processa il testo con il modello ONNX e restituisce le predizioni."""
    print("\n>>> Loading tokenizer and ONNX session...")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_CHECKPOINT)
    session = onnxruntime.InferenceSession(ONNX_MODEL_PATH)

    print("\n>>> Running inference...")
    inputs = tokenizer(text, return_tensors="np")
    input_ids = inputs["input_ids"]
    attention_mask = inputs["attention_mask"]
    
    # L'input per ONNX deve essere un dizionario
    onnx_inputs = { 
        "input_ids": input_ids.astype(np.int64),
        "attention_mask": attention_mask.astype(np.int64)
    }
    
    # Esegui l'inferenza
    logits = session.run(None, onnx_inputs)[0]
    
    # Ottieni le predizioni
    predictions = np.argmax(logits, axis=2)
    
    return predictions[0], input_ids[0]

def post_process_and_display(predictions, input_ids):
    """Raggruppa i token e le etichette e stampa il risultato."""
    print("\n>>> Parsed Segments:")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_CHECKPOINT)
    tokens = tokenizer.convert_ids_to_tokens(input_ids)
    
    current_segment = None
    current_words = []

    for token, pred_id in zip(tokens, predictions):
        # Ignora i token speciali come [CLS] e [SEP]
        if token in tokenizer.all_special_tokens:
            continue

        label = id2label[pred_id]
        
        # Pulisci il token per la visualizzazione
        word = token.replace("##", "")

        segment_type = label.split('-')[-1] if label != 'O' else 'O'

        if current_segment is None:
            current_segment = segment_type
        
        if segment_type != current_segment and current_segment != 'O':
            # Stampa il segmento completato
            print(f"  - [{current_segment}]: {' '.join(current_words)}")
            current_words = []
            current_segment = segment_type
        
        current_words.append(word)

    # Stampa l'ultimo segmento
    if current_words and current_segment != 'O':
        print(f"  - [{current_segment}]: {' '.join(current_words)}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Test the buzz parser model with an Ollama backend.")
    parser.add_argument(
        "--model",
        type=str,
        default="gemma3:1b",
        help="The name of the Ollama model to use (e.g., 'mistral', 'llama3')."
    )
    args = parser.parse_args()

    # Prompt di esempio per testare il modello
    test_prompt = "Please tell me the contents of the file prepare_dataset.py"
    
    # 1. Ottieni la risposta dall'LLM
    ollama_response = query_ollama(test_prompt, model_name=args.model)
    
    if ollama_response:
        # 2. Esegui l'inferenza con il nostro modello ONNX
        predictions, input_ids = run_inference(ollama_response)
        
        # 3. Processa e visualizza i risultati
        post_process_and_display(predictions, input_ids)
