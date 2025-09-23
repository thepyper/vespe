import os
import argparse
import subprocess
import sys

# Descrizione dello script per l'help
parser = argparse.ArgumentParser(
    description="Export a fine-tuned Hugging Face Transformer model to ONNX format."
)

# Argomento per specificare il percorso del modello da convertire
parser.add_argument(
    "--model_path",
    type=str,
    required=True,
    help="Path to the directory containing the fine-tuned model (e.g., 'buzz-parser-v1/final_model')."
)

# Argomento per specificare la directory di output per il modello ONNX
parser.add_argument(
    "--output_dir",
    type=str,
    default="onnx",
    help="The directory where the ONNX model will be saved."
)

parser.add_argument(
    "--opset",
    type=int,
    default=14,
    help="The ONNX opset version to use for the export."
)

def main():
    args = parser.parse_args()

    print(f"Starting ONNX export for model at: {args.model_path}")

    # Verifica che la directory del modello esista
    if not os.path.isdir(args.model_path):
        print(f"Error: Model directory not found at '{args.model_path}'", file=sys.stderr)
        sys.exit(1)

    # Comando da eseguire
    command = [
        sys.executable, # Usa l'interprete python corrente
        "-m",
        "transformers.onnx",
        "--model",
        args.model_path,
        f"--opset={args.opset}",
        "--feature=token-classification",
        args.output_dir,
    ]

    print(f"\nRunning command:\n{' '.join(command)}\n")

    try:
        # Esegui il comando
        subprocess.run(command, check=True)
        print("\nONNX export completed successfully.")
        print(f"Model saved in: {args.output_dir}/model.onnx")
    except FileNotFoundError:
        print("Error: `transformers` or `onnx` may not be installed.", file=sys.stderr)
        print("Please make sure you have run: pip install transformers[onnx]", file=sys.stderr)
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print(f"An error occurred during the export process: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
