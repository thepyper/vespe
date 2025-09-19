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
    "O", "B-THOUGHT", "I-THOUGHT", "B-TOOLCALL", "I-TOOLCALL",
    "B-TOOLRESPONSE", "I-TOOLRESPONSE", "B-TEXT", "I-TEXT",
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
                span_label = span["label"]
                start_char = span["start"]
                end_char = span["end"]

                # Find the first token of the span
                token_start_index = -1
                for idx, (start, end) in enumerate(offset_mapping):
                    if start >= start_char and end <= end_char:
                        token_start_index = idx
                        break
                
                if token_start_index != -1:
                    # Assign the B- tag to the first token
                    doc_labels[token_start_index] = label2id[f"B-{span_label}"]

                    # Assign I- tags to subsequent tokens in the same span
                    for idx in range(token_start_index + 1, len(offset_mapping)):
                        start, end = offset_mapping[idx]
                        if start < end_char and end <= end_char:
                            doc_labels[idx] = label2id[f"I-{span_label}"]
                        else:
                            break # Exit when the token is outside the span
            
            labels.append(doc_labels)

        tokenized_inputs["labels"] = labels
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