import os
import json

# Definiamo le directory di input e il file di output (relativi alla root del progetto)
SOURCE_DIR = "buzz/training/generated_examples_rust"
OUTPUT_FILE = "buzz/training/dataset.jsonl"

def create_dataset():
    """
    Crea un dataset in formato JSONL leggendo file JSON pre-processati.
    Ogni file JSON di input deve contenere 'full_text' e 'spans'.
    """
    print(f"Starting dataset creation from pre-processed JSON examples...")
    print(f"Source directory: {SOURCE_DIR}")

    processed_files = 0
    
    if not os.path.isdir(SOURCE_DIR):
        print(f"  - ERROR: Source directory not found: {SOURCE_DIR}")
        return

    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f_out:
        for filename in sorted(os.listdir(SOURCE_DIR)):
            if not filename.endswith(".json"):
                continue

            file_path = os.path.join(SOURCE_DIR, filename)
            
            try:
                with open(file_path, 'r', encoding='utf-8') as f_in:
                    data = json.load(f_in)

                # Estrai full_text e spans direttamente dal JSON
                full_text = data.get("full_text")
                spans_raw = data.get("spans")

                # Valida che i campi necessari esistano
                if full_text is None or spans_raw is None:
                    print(f"  - WARNING: Skipping {filename} due to missing 'full_text' or 'spans' key.")
                    continue

                # Adatta gli spans: la chiave "category" viene rinominata in "label"
                # per compatibilità con gli script di training.
                adapted_spans = [
                    {
                        "label": span.get("category"), 
                        "start": span.get("start"), 
                        "end": span.get("end")
                    } 
                    for span in spans_raw
                ]

                # Scrivi l'esempio processato come una riga JSON nel file di output
                f_out.write(json.dumps({"full_text": full_text, "spans": adapted_spans}) + "\n")
                processed_files += 1

            except json.JSONDecodeError:
                print(f"  - WARNING: Skipping invalid JSON file: {filename}")
            except Exception as e:
                print(f"  - ERROR: Failed to process file {filename}: {e}")

    print(f"\nDataset creation complete.")
    print(f"Processed {processed_files} files.")
    print(f"Output written to {OUTPUT_FILE}")

if __name__ == "__main__":
    create_dataset()