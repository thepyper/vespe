# -*- coding: utf-8 -*-
"""
Questo script implementa una pipeline per generare un dataset di addestramento.
Utilizza un modello LLM "grande" (insegnante) per creare prompt e per etichettare
le risposte di un modello LLM "piccolo" (studente).

Il processo è il seguente:
1. Il modello GRANDE crea un prompt per l'utente che spinga il modello PICCOLO
   a usare un tool specifico.
2. Il modello PICCOLO risponde al prompt.
3. Il modello GRANDE analizza la conversazione e la etichetta, producendo un
   file JSON strutturato che può essere usato per l'addestramento del parser.
"""

import argparse
import json
import os
import random
import requests

# --- COSTANTI E CONFIGURAZIONE ---

# Lista dei tool che vogliamo che il modello "piccolo" impari a usare.
# Lo script sceglierà un tool a caso da questa lista per ogni esempio generato.
TOOLS_TO_PRACTICE = [
    'read_file',
    'write_file',
    'list_directory',
    'run_shell_command',
    'search_file_content',
    'glob'
]

# --- FUNZIONI HELPER ---

def query_ollama(model_name, prompt, ollama_url, system_prompt=None):
    """
    Invia una richiesta a un modello LLM tramite l'API di Ollama e restituisce la risposta.

    Args:
        model_name (str): Il nome del modello da interrogare (es. "gemma3:1b").
        prompt (str): Il prompt da inviare al modello.
        ollama_url (str): L'URL dell'istanza di Ollama.
        system_prompt (str, optional): Un prompt di sistema da usare. Default a None.

    Returns:
        str: La risposta del modello come stringa, o None se si verifica un errore.
    """
    print(f"\n---\nQuery al modello: {model_name}")
    try:
        # Costruiamo il payload per la richiesta a Ollama
        payload = {
            "model": model_name,
            "prompt": prompt,
            "stream": False, # Vogliamo la risposta completa, non uno stream
        }
        if system_prompt:
            payload["system"] = system_prompt

        # Eseguiamo la chiamata POST all'API di Ollama
        response = requests.post(
            f"{ollama_url}/api/generate",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        # Controlliamo se la richiesta è andata a buon fine
        response.raise_for_status()

        # Estraiamo il contenuto della risposta
        response_data = response.json()
        return response_data.get("response", "").strip()

    except requests.exceptions.RequestException as e:
        print(f"ERRORE: Impossibile comunicare con Ollama: {e}")
        return None
    except json.JSONDecodeError:
        print(f"ERRORE: Risposta non valida (non JSON) da Ollama.")
        return None


def generate_student_prompt(teacher_model, ollama_url, tool_name):
    """
    Passo 1: Il modello "grande" (insegnante) genera un prompt per il modello "piccolo".

    Args:
        teacher_model (str): Il nome del modello insegnante.
        ollama_url (str): L'URL dell'istanza di Ollama.
        tool_name (str): Il nome del tool che il modello piccolo dovrebbe essere
                         incoraggiato a usare.

    Returns:
        str: Il prompt generato per lo studente, o None in caso di errore.
    """
    print(f"PASSO 1: Il modello '{teacher_model}' genera un prompt per il tool '{tool_name}'...")

    # Questo è il "meta-prompt": il prompt che diamo al modello grande per fargli
    # creare un prompt per il modello piccolo.
    meta_prompt = (
        f"Sei un generatore di prompt per un assistente AI. Il tuo compito è creare un "
        f"prompt utente che richieda all'assistente di usare un tool specifico. "
        f"Il prompt deve essere in linguaggio naturale e non deve contenere chiamate dirette a tool.\n"
        f"Genera un prompt che richieda l'uso del tool '{tool_name}'.\n"
        f"Output solo il prompt utente."
    )

    student_prompt = query_ollama(teacher_model, meta_prompt, ollama_url)

    if student_prompt:
        print(f"Prompt generato: '{student_prompt}'")
    else:
        print("ERRORE: Il modello insegnante non è riuscito a generare un prompt.")

    return student_prompt


def get_student_response(student_model, ollama_url, student_prompt):
    """
    Passo 2: Il modello "piccolo" (studente) risponde al prompt generato.

    Args:
        student_model (str): Il nome del modello studente.
        ollama_url (str): L'URL dell'istanza di Ollama.
        student_prompt (str): Il prompt a cui lo studente deve rispondere.

    Returns:
        str: La risposta grezza dello studente, o None in caso di errore.
    """
    print(f"PASSO 2: Il modello '{student_model}' risponde al prompt...")

    # In questo caso, non usiamo un system prompt specifico, lasciamo che il modello
    # si comporti con la sua configurazione di default.
    student_response = query_ollama(student_model, student_prompt, ollama_url)

    if student_response:
        print(f"Risposta generata:\n{student_response}")
    else:
        print("ERRORE: Il modello studente non ha fornito una risposta.")

    return student_response


def label_student_response(teacher_model, ollama_url, student_prompt, student_response):
    """
    Passo 3: Il modello "grande" etichetta la risposta del modello "piccolo".

    Args:
        teacher_model (str): Il nome del modello insegnante/etichettatore.
        ollama_url (str): L'URL dell'istanza di Ollama.
        student_prompt (str): Il prompt originale a cui lo studente ha risposto.
        student_response (str): La risposta grezza dello studente.

    Returns:
        dict: Un dizionario contenente il testo completo e le etichette (spans),
              o None in caso di errore o JSON non valido.
    """
    print(f"PASSO 3: Il modello '{teacher_model}' etichetta la risposta...")

    # Questo è il prompt per l'etichettatura. È molto specifico per ottenere
    # l'output JSON nel formato corretto.
    labeling_prompt = (
        f"Sei un etichettatore di output di assistenti AI. Ti verrà fornito un prompt utente "
        f"e la risposta grezza di un assistente AI. Il tuo compito è segmentare la risposta "
        f"dell'assistente nelle categorie funzionali: THOUGHT, TOOL_CODE, TEXT.\n"
        f"Per ogni segmento, devi fornire l'etichetta e le posizioni esatte (start/end) "
        f"a livello di carattere all'interno della risposta grezza.\n"
        f"L'output deve essere un oggetto JSON con due chiavi: \"full_text\" (la risposta "
        f"grezza completa) e \"spans\" (una lista di oggetti, ognuno con \"label\", \"start\", \"end\").\n\n"
        f"Prompt Utente:\n{student_prompt}\n\n"
        f"Risposta Assistente AI:\n{student_response}\n\n"
        f"Output JSON:"
    )

    labeled_json_str = query_ollama(teacher_model, labeling_prompt, ollama_url)

    if not labeled_json_str:
        print("ERRORE: Il modello etichettatore non ha fornito una risposta.")
        return None

    # Verifichiamo che l'output sia un JSON valido e contenga le chiavi attese.
    try:
        # A volte i modelli LLM avvolgono il JSON in blocchi di codice markdown.
        # Proviamo a pulire la stringa prima di fare il parsing.
        if labeled_json_str.startswith("```json"):
            labeled_json_str = labeled_json_str[7:]
        if labeled_json_str.endswith("```"):
            labeled_json_str = labeled_json_str[:-3]
        
        labeled_data = json.loads(labeled_json_str.strip())

        # Controllo di validità del JSON
        if "full_text" in labeled_data and "spans" in labeled_data:
            print("Etichettatura completata e valida.")
            return labeled_data
        else:
            print("ERRORE: Il JSON generato non contiene le chiavi 'full_text' e 'spans'.")
            return None
    except json.JSONDecodeError:
        print(f"ERRORE: L'output del modello non è un JSON valido.\nOutput ricevuto:\n{labeled_json_str}")
        return None


def save_labeled_example(output_dir, example_data, example_index):
    """
    Passo 4: Salva l'esempio etichettato in un file JSON.

    Args:
        output_dir (str): La cartella dove salvare il file.
        example_data (dict): I dati etichettati da salvare.
        example_index (int): L'indice dell'esempio, usato per il nome del file.
    """
    try:
        # Creiamo la cartella di output se non esiste
        os.makedirs(output_dir, exist_ok=True)

        # Definiamo il percorso del file
        file_path = os.path.join(output_dir, f"example_{example_index:04d}.json")

        # Scriviamo i dati nel file JSON
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(example_data, f, ensure_ascii=False, indent=4)

        print(f"PASSO 4: Esempio salvato in '{file_path}'")

    except IOError as e:
        print(f"ERRORE: Impossibile scrivere il file '{file_path}': {e}")


# --- FUNZIONE PRINCIPALE ---

def main():
    """
    Funzione principale che orchestra l'intera pipeline.
    """
    # 1. Configurazione del parser per gli argomenti da riga di comando
    parser = argparse.ArgumentParser(
        description="Genera un dataset di addestramento usando un modello 'insegnante' e uno 'studente'.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter # Mostra i valori di default nell'help
    )
    parser.add_argument(
        "--big-model",
        type=str,
        default="gpt-oss:20b",
        help="Nome del modello 'grande' (insegnante/etichettatore) da usare in Ollama."
    )
    parser.add_argument(
        "--small-model",
        type=str,
        default="gemma3:1b",
        help="Nome del modello 'piccolo' (studente) da usare in Ollama."
    )
    parser.add_argument(
        "--ollama-url",
        type=str,
        default="http://localhost:11434",
        help="URL base dell'API di Ollama."
    )
    parser.add_argument(
        "--num-examples",
        type=int,
        default=10,
        help="Numero di esempi di addestramento da generare."
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="buzz/training/generated_examples",
        help="Cartella dove salvare i file JSON generati."
    )

    args = parser.parse_args()

    # 2. Stampa della configurazione iniziale
    print("--- Inizio Pipeline di Generazione Dati ---")
    print(f"Modello Insegnante: {args.big_model}")
    print(f"Modello Studente:   {args.small_model}")
    print(f"URL Ollama:         {args.ollama_url}")
    print(f"Num. Esempi:        {args.num_examples}")
    print(f"Output Dir:         {args.output_dir}")
    print("-------------------------------------------")

    # 3. Ciclo di generazione degli esempi
    for i in range(1, args.num_examples + 1):
        print(f"\n========== Inizio Esempio {i}/{args.num_examples} ==========")

        # Scegliamo un tool a caso dalla nostra lista
        tool_to_practice = random.choice(TOOLS_TO_PRACTICE)

        # PASSO 1: Genera prompt per lo studente
        student_prompt = generate_student_prompt(args.big_model, args.ollama_url, tool_to_practice)
        if not student_prompt:
            print(f"ERRORE: Saltando l'esempio {i} a causa di un fallimento nel Passo 1.")
            continue

        # PASSO 2: Ottieni la risposta dello studente
        student_response = get_student_response(args.small_model, args.ollama_url, student_prompt)
        if not student_response:
            print(f"ERRORE: Saltando l'esempio {i} a causa di un fallimento nel Passo 2.")
            continue

        # PASSO 3: Etichetta la risposta
        labeled_data = label_student_response(args.big_model, args.ollama_url, student_prompt, student_response)
        if not labeled_data:
            print(f"ERRORE: Saltando l'esempio {i} a causa di un fallimento nel Passo 3.")
            continue

        # PASSO 4: Salva l'esempio
        save_labeled_example(args.output_dir, labeled_data, i)

        print(f"========== Fine Esempio {i}/{args.num_examples} ==========")

    print("\n--- Pipeline completata ---")


# Questo blocco di codice viene eseguito solo se lo script è lanciato direttamente
# (es. "python nome_script.py") e non se viene importato come modulo.
if __name__ == "__main__":
    main()
