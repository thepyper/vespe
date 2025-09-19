import os
import json
import re

# Definiamo le directory di input e il file di output
SOURCE_DIR = "2025_09_19_gemini"
OUTPUT_FILE = "dataset.jsonl"

# Mapping delle nostre etichette di alto livello a quelle BIO (Begin, Inside)
# "O" (Outside) non ci serve qui, perché ogni parola avrà un'etichetta.
LABEL_MAP = {
    "THOUGHT": ("B-THOUGHT", "I-THOUGHT"),
    "TOOL_CALL": ("B-TOOLCALL", "I-TOOLCALL"),
    "TOOL_RESPONSE": ("B-TOOLRESPONSE", "I-TOOLRESPONSE"),
    "TEXT": ("B-TEXT", "I-TEXT"),
}

def create_dataset():
    """
    Legge tutti i file .txt dalla directory sorgente, li processa,
    e scrive il risultato nel file di output JSONL.
    """
    print(f"Starting dataset creation...")
    print(f"Source directory: {SOURCE_DIR}")

    # Regex per estrarre LABEL e contenuto da una riga
    line_regex = re.compile(r"^([A-Z_]+):\s?(.*)$")

    processed_files = 0
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f_out:
        # Itera su tutti i file nella directory sorgente
        for filename in sorted(os.listdir(SOURCE_DIR)):
            if filename.endswith(".txt"):
                file_path = os.path.join(SOURCE_DIR, filename)
                
                all_words = []
                all_labels = []

                try:
                    with open(file_path, 'r', encoding='utf-8') as f_in:
                        lines = f_in.readlines()

                    if not lines:
                        print(f"  - WARNING: Skipping empty file: {filename}")
                        continue

                    # Processa ogni riga del file di esempio
                    for line in lines:
                        line = line.strip()
                        if not line:
                            continue

                        match = line_regex.match(line)
                        if not match:
                            print(f"  - WARNING: Skipping malformed line in {filename}: {line}")
                            continue
                        
                        label, content = match.groups()

                        if label not in LABEL_MAP:
                            print(f"  - WARNING: Skipping unknown label '{label}' in {filename}")
                            continue
                        
                        # Dividi il contenuto in parole (semplice split su spazi)
                        words = content.split()
                        if not words:
                            continue

                        # Assegna le etichette BIO
                        b_tag, i_tag = LABEL_MAP[label]
                        line_labels = [b_tag] + [i_tag] * (len(words) - 1)
                        
                        all_words.extend(words)
                        all_labels.extend(line_labels)

                    # Scrivi l'esempio processato come una riga JSON nel file di output
                    if all_words:
                        f_out.write(json.dumps({"words": all_words, "labels": all_labels}) + "\n")
                        processed_files += 1

                except Exception as e:
                    print(f"  - ERROR: Failed to process file {filename}: {e}")

    print(f"\nDataset creation complete.")
    print(f"Processed {processed_files} files.")
    print(f"Output written to {OUTPUT_FILE}")

if __name__ == "__main__":
    create_dataset()
