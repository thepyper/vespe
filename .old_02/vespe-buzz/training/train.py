# This script fine-tunes a DistilBERT model for token classification on a custom dataset.

import json
import torch
import numpy as np
from datasets import load_dataset
from transformers import (
    AutoTokenizer,
    AutoModelForTokenClassification,
    DataCollatorForTokenClassification,
    TrainingArguments,
    Trainer,
)
import evaluate

# --- CONFIGURATION ---
MODEL_CHECKPOINT = "distilbert-base-uncased"
DATASET_FILE = "dataset.jsonl"
OUTPUT_DIR = "buzz-parser-v2"

# Define all possible labels for our BIO scheme
LABELS = [
    "O", # Outside
    "B-THOUGHT", "I-THOUGHT",
    "B-TOOL_CALL", "I-TOOL_CALL",
    "B-TOOL_RESPONSE", "I-TOOL_RESPONSE",
    "B-TEXT", "I-TEXT",
]

# Create mappings between labels and integer IDs
label2id = {label: i for i, label in enumerate(LABELS)}
id2label = {i: label for i, label in enumerate(LABELS)}

# --- DATA PREPARATION ---

def main():
    print("Loading dataset...")
    raw_datasets = load_dataset('json', data_files=DATASET_FILE, split="train")
    raw_datasets = raw_datasets.train_test_split(test_size=0.1, seed=42)

    print("\nDataset loaded and split:")
    print(raw_datasets)

    tokenizer = AutoTokenizer.from_pretrained(MODEL_CHECKPOINT)

    def tokenize_and_align_labels(examples):
        """
        This function tokenizes the full text and aligns labels using character spans.
        This version robustly handles token-span overlaps.
        """
        # Tokenize the full text, getting the offset mapping
        tokenized_inputs = tokenizer(
            examples["full_text"], 
            truncation=True, 
            return_offsets_mapping=True
        )

        labels = []
        for i, offset_mapping in enumerate(tokenized_inputs["offset_mapping"]):
            # Initialize all labels to 'O'
            doc_labels = [label2id['O']] * len(offset_mapping)
            spans = examples["spans"][i]

            for span in spans:
                start_char = span["start"]
                end_char = span["end"]
                label = span["label"]
                
                # Find all tokens that are within the character span
                span_token_indices = []
                for token_idx, (token_start, token_end) in enumerate(offset_mapping):
                    # Skip special tokens which have an offset of (0, 0)
                    if token_start == token_end:
                        continue
                    
                    # Add token to span if it overlaps with the character span
                    if token_start < end_char and token_end > start_char:
                        span_token_indices.append(token_idx)
                
                if span_token_indices:
                    # Mark the first token in the sequence as Begin
                    doc_labels[span_token_indices[0]] = label2id[f"B-{label}"]
                    # Mark all subsequent tokens as Inside
                    for token_idx in span_token_indices[1:]:
                        doc_labels[token_idx] = label2id[f"I-{label}"]
            
            labels.append(doc_labels)

        tokenized_inputs["labels"] = labels

        # --- DEBUGGING LOG (Write to file) ---
        # Only log for the first few examples to avoid huge files
        if examples["full_text"][0] and i < 5: 
            debug_output_path = "debug_token_alignment.jsonl"
            with open(debug_output_path, 'a', encoding='utf-8') as debug_f:
                debug_entry = {
                    "example_index": i,
                    "full_text": examples['full_text'][i].strip(),
                    "tokens": tokenizer.convert_ids_to_tokens(tokenized_inputs["input_ids"][i]),
                    "labels": [id2label[l_id] if l_id != -100 else "(ignored)" for l_id in tokenized_inputs["labels"][i]],
                    "offset_mapping": tokenized_inputs["offset_mapping"][i].tolist() # Convert numpy array to list
                }
                debug_f.write(json.dumps(debug_entry, indent=2) + ",\n")

        return tokenized_inputs

    print("\nTokenizing and aligning labels with new span-based method...")
    tokenized_datasets = raw_datasets.map(tokenize_and_align_labels, batched=True)

    # --- METRICS ---
    metric = evaluate.load("seqeval")

    def compute_metrics(p):
        predictions, labels = p
        predictions = np.argmax(predictions, axis=2)

        true_predictions = [
            [LABELS[p] for (p, l) in zip(prediction, label) if l != -100]
            for prediction, label in zip(predictions, labels)
        ]
        true_labels = [
            [LABELS[l] for (p, l) in zip(prediction, label) if l != -100]
            for prediction, label in zip(predictions, labels)
        ]

        results = metric.compute(predictions=true_predictions, references=true_labels)
        return {
            "precision": results["overall_precision"],
            "recall": results["overall_recall"],
            "f1": results["overall_f1"],
            "accuracy": results["overall_accuracy"],
        }

    # --- TRAINING ---
    print("\nSetting up model and trainer...")
    data_collator = DataCollatorForTokenClassification(tokenizer=tokenizer)

    model = AutoModelForTokenClassification.from_pretrained(
        MODEL_CHECKPOINT,
        num_labels=len(LABELS),
        id2label=id2label,
        label2id=label2id,
    )

    training_args = TrainingArguments(
        output_dir=OUTPUT_DIR,
        learning_rate=2e-5,
        per_device_train_batch_size=16,
        per_device_eval_batch_size=16,
        num_train_epochs=3,
        weight_decay=0.01,
        evaluation_strategy="epoch",
        save_strategy="epoch",
        load_best_model_at_end=True,
        save_safetensors=False, # Keep compatibility with ONNX exporter
        push_to_hub=False,
    )

    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=tokenized_datasets["train"],
        eval_dataset=tokenized_datasets["test"],
        tokenizer=tokenizer,
        data_collator=data_collator,
        compute_metrics=compute_metrics,
    )

    print("\nStarting training...")
    trainer.train()

    print("\nTraining complete. Saving the best model...")
    trainer.save_model(f"{OUTPUT_DIR}/final_model")

    print(f"Model saved to {OUTPUT_DIR}/final_model")
    print("\nTo export to ONNX, run the following command:")
    print(f"python export_to_onnx.py --model_path {OUTPUT_DIR}/final_model")

if __name__ == "__main__":
    main()