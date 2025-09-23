# Workflow di Training per il Parser `buzz`

Questo documento descrive il processo completo per addestrare un nuovo modello di parsing, dalla preparazione dei dati all'esportazione del modello finale per l'uso in produzione.

Il processo è diviso in 3 passi principali, ognuno gestito da uno script Python.

## Prerequisiti

Assicurati di avere Python installato e di aver creato un ambiente virtuale. Installa tutte le dipendenze necessarie:

```bash
pip install torch
pip install transformers datasets evaluate seqeval
pip install transformers[onnx]
```

## Workflow

### Passo 1: Preparare il Dataset

Questo passo converte tutti i file di esempio `.txt` (che si trovano in `2025_09_19_gemini/`) in un unico file `dataset.jsonl` formattato per il training.

**Comando:**
```bash
python prepare_dataset.py
```

**Output:** Un file `dataset.jsonl` verrà creato in questa directory.

---

### Passo 2: Addestrare il Modello

Questo passo usa il file `dataset.jsonl` per eseguire il fine-tuning di un modello `distilbert-base-uncased`.

**Nota:** Questo script è pensato per essere eseguito su un ambiente con GPU, come **Google Colab**. Per eseguirlo, carica sia questo script (o copia il suo contenuto) sia il file `dataset.jsonl` nel tuo notebook Colab.

**Comando (se eseguito localmente):**
```bash
python train.py
```

**Output:** Una nuova directory verrà creata (es. `buzz-parser-v1/`), contenente i file del modello addestrato. Il percorso del modello migliore (es. `buzz-parser-v1/checkpoint-XYZ` o `buzz-parser-v1/final_model`) sarà stampato alla fine del processo.

---

### Passo 3: Esportare il Modello in ONNX

Questo passo finale prende il modello addestrato e lo converte nel formato `.onnx`, che è un formato portabile e performante per l'inferenza, ideale per essere usato da Rust.

**Comando:**
```bash
# Sostituisci <path_to_your_best_model> con il percorso stampato alla fine del training
python export_to_onnx.py --model_path <path_to_your_best_model>
```

**Esempio:**
```bash
python export_to_onnx.py --model_path buzz-parser-v1/final_model
```

**Output:** Una nuova directory `onnx/` verrà creata, contenente il file `model.onnx`. Questo è l'artefatto finale da usare nel tuo applicativo Rust.
