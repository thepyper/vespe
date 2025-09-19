# This script fine-tunes a DistilBERT model for token classification on a custom dataset.
# It assumes the dataset is a JSONL file where each line has "words" and "labels".

# 1. Install and Import necessary libraries
# ------------------------------------------

# In a Colab/Jupyter notebook, run this cell first:
# !pip install transformers datasets torch evaluate seqeval

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

# 2. Define Constants and Configuration
# -------------------------------------

MODEL_CHECKPOINT = "distilbert-base-uncased"
DATASET_FILE = "dataset.jsonl"
OUTPUT_DIR = "buzz-parser-v1"

# Define all possible labels from our BIO scheme
# This must be consistent with the prepare_dataset.py script
LABELS = [
    "O", # Outside
    "B-THOUGHT", "I-THOUGHT",
    "B-TOOLCALL", "I-TOOLCALL",
    "B-TOOLRESPONSE", "I-TOOLRESPONSE",
    "B-TEXT", "I-TEXT",
]

# Create mappings between labels and integer IDs
label2id = {label: i for i, label in enumerate(LABELS)}
id2label = {i: label for i, label in enumerate(LABELS)}

# 3. Load and Prepare the Dataset
# --------------------------------

def main():
    print("Loading dataset...")
    # Load the dataset from our JSONL file
    raw_datasets = load_dataset('json', data_files=DATASET_FILE, split="train")

    # Split the dataset into training and testing sets (e.g., 90% train, 10% test)
    raw_datasets = raw_datasets.train_test_split(test_size=0.1, seed=42)

    print("\nDataset loaded and split:")
    print(raw_datasets)

    # Load the tokenizer for our chosen model
    tokenizer = AutoTokenizer.from_pretrained(MODEL_CHECKPOINT)

    def tokenize_and_align_labels(examples):
        """
        This is the core preprocessing function.
        It tokenizes the words and aligns the labels with the new subword tokens.
        """
        tokenized_inputs = tokenizer(examples["words"], truncation=True, is_split_into_words=True)

        labels = []
        for i, label_list in enumerate(examples["labels"]):
            word_ids = tokenized_inputs.word_ids(batch_index=i)
            previous_word_idx = None
            label_ids = []
            for word_idx in word_ids:
                if word_idx is None:
                    label_ids.append(-100)
                elif word_idx != previous_word_idx:
                    label_ids.append(label2id[label_list[word_idx]])
                else:
                    label_ids.append(-100)
                previous_word_idx = word_idx
            labels.append(label_ids)

        tokenized_inputs["labels"] = labels
        return tokenized_inputs

    print("\nTokenizing and aligning labels...")
    tokenized_datasets = raw_datasets.map(tokenize_and_align_labels, batched=True)

    # 4. Define Metrics for Evaluation
    # ---------------------------------

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

    # 5. Configure and Run the Training
    # ----------------------------------

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

    # 6. Save the Final Model
    # ------------------------

    print("\nTraining complete. Saving the best model...")
    trainer.save_model(f"{OUTPUT_DIR}/final_model")

    print(f"Model saved to {OUTPUT_DIR}/final_model")
    print("\nTo export to ONNX, run the following command in your terminal:")
    print(f"pip install transformers[onnx]")
    print(f"python -m transformers.onnx --model={OUTPUT_DIR}/final_model --feature=token-classification onnx/")

if __name__ == "__main__":
    main()
