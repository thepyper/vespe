import os
import json
import re

# Definiamo le directory di input e il file di output
SOURCE_DIR = "2025_09_19_gemini"
OUTPUT_FILE = "dataset.jsonl"

def create_dataset():
    """
    Legge tutti i file .txt, li processa per creare un testo completo
    e una lista di "spans" (intervalli di caratteri) per ogni etichetta.
    Scrive il risultato nel file di output JSONL.
    """
    print(f"Starting dataset creation with span-based approach...")
    print(f"Source directory: {SOURCE_DIR}")

    # Regex per estrarre LABEL e contenuto da una riga
    line_regex = re.compile(r"^([A-Z_]+):\s?(.*)$")

    processed_files = 0
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f_out:
        for filename in sorted(os.listdir(SOURCE_DIR)):
            if not filename.endswith(".txt"):
                continue

            file_path = os.path.join(SOURCE_DIR, filename)
            
            try:
                with open(file_path, 'r', encoding='utf-8') as f_in:
                    lines = f_in.readlines()

                if not lines:
                    print(f"  - WARNING: Skipping empty file: {filename}")
                    continue

                full_text = ""
                spans = []
                current_pos = 0

                for line in lines:
                    # Aggiungiamo la riga al testo completo
                    full_text += line
                    
                    line_content = line.strip()
                    if not line_content:
                        current_pos = len(full_text)
                        continue

                    match = line_regex.match(line_content)
                    if not match:
                        print(f"  - WARNING: Skipping malformed line in {filename}: {line_content}")
                        current_pos = len(full_text)
                        continue
                    
                    label, content = match.groups()
                    
                    # Calcola la posizione di inizio e fine del contenuto
                    # L'inizio è la posizione corrente + la lunghezza del prefisso (es. "THOUGHT: ")
                    content_start = current_pos + (line_content.find(content))
                    content_end = content_start + len(content)

                    # Aggiungi lo span alla nostra lista
                    spans.append({
                        "label": label,
                        "start": content_start,
                        "end": content_end
                    })
                    
                    # Aggiorna la posizione corrente alla fine della riga nel testo completo
                    current_pos = len(full_text)

                # Scrivi l'esempio processato come una riga JSON
                if spans:
                    f_out.write(json.dumps({"full_text": full_text, "spans": spans}) + "\n")
                    processed_files += 1

            except Exception as e:
                print(f"  - ERROR: Failed to process file {filename}: {e}")

    print(f"\nDataset creation complete.")
    print(f"Processed {processed_files} files.")
    print(f"Output written to {OUTPUT_FILE}")

if __name__ == "__main__":
    create_dataset()