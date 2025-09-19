import os
import json
import re

# Definiamo le directory di input e il file di output
SOURCE_DIR = "2025_09_19_gemini"
OUTPUT_FILE = "dataset.jsonl"

# Regex per identificare l'inizio di un nuovo segmento
# Cerca "LABEL:" all'inizio di una riga
SEGMENT_START_REGEX = re.compile(r"^(THOUGHT|TOOL_CALL|TOOL_RESPONSE|TEXT):\s?(.*)$")

def create_dataset():
    print(f"Starting dataset creation with robust multi-line handling...")
    print(f"Source directory: {SOURCE_DIR}")

    processed_files = 0
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f_out:
        for filename in sorted(os.listdir(SOURCE_DIR)):
            if not filename.endswith(".txt"):
                continue

            file_path = os.path.join(SOURCE_DIR, filename)
            
            try:
                with open(file_path, 'r', encoding='utf-8') as f_in:
                    raw_lines = f_in.readlines()

                if not raw_lines:
                    print(f"  - WARNING: Skipping empty file: {filename}")
                    continue

                # --- Pass 1: Consolidate Multi-line Segments ---
                consolidated_segments = []
                current_segment_lines = []
                
                for line_idx, line in enumerate(raw_lines):
                    stripped_line = line.strip()
                    if not stripped_line: # Skip empty lines
                        continue

                    match = SEGMENT_START_REGEX.match(stripped_line)
                    
                    if match: # This line starts a new segment
                        if current_segment_lines: # Save the previous segment if it exists
                            consolidated_segments.append("".join(current_segment_lines))
                        current_segment_lines = [line] # Start new segment with current line
                    else: # This line is a continuation of the previous segment
                        if current_segment_lines:
                            current_segment_lines.append(line)
                        else: # Handle case where first line doesn't start with a label
                            print(f"  - WARNING: First non-empty line in {filename} does not start with a label. Skipping: {line.strip()}")
                            # Decide how to handle: skip, or add to a dummy segment. For now, skip.
                
                # Add the last segment after the loop
                if current_segment_lines:
                    consolidated_segments.append("".join(current_segment_lines))

                # --- Pass 2: Extract full_text and spans from Consolidated Segments ---
                full_text = ""
                spans = []
                current_char_pos = 0

                for segment_text in consolidated_segments:
                    # The segment_text already includes its own newlines
                    full_text += segment_text
                    
                    # Find the actual content after the LABEL: prefix
                    match = SEGMENT_START_REGEX.match(segment_text.strip())
                    if not match:
                        # This should not happen if Pass 1 worked correctly, but for safety
                        print(f"  - ERROR: Consolidated segment does not match regex: {segment_text.strip()}")
                        continue
                    
                    label, content = match.groups()
                    
                    # Calculate start and end character positions of the content within full_text
                    # Need to find the start of the content relative to the start of the segment_text
                    # Then add current_char_pos to get absolute position in full_text
                    
                    # Find the index of the content within the segment_text (after the label prefix)
                    # Example: "THOUGHT: My thought\n" -> content is "My thought"
                    # The content starts after "THOUGHT: "
                    content_relative_start = segment_text.find(content) 
                    
                    if content_relative_start == -1: # Should not happen if regex matched
                        print(f"  - ERROR: Content not found in segment: {segment_text.strip()}")
                        continue

                    span_start = current_char_pos + content_relative_start
                    span_end = span_start + len(content)

                    spans.append({
                        "label": label,
                        "start": span_start,
                        "end": span_end
                    })
                    
                    # Update current_char_pos to the end of the current segment in full_text
                    current_char_pos = len(full_text)

                # Write the processed example as a JSON line
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