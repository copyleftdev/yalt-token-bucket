import os
import json

def collect_files(base_dir):
    collected_data = []

    for root, dirs, files in os.walk(base_dir):
        # Skip 'target' directory
        if 'target' in dirs:
            dirs.remove('target')

        for file in files:
            file_path = os.path.join(root, file)
            try:
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                collected_data.append({
                    'path': file_path,
                    'filename': file,
                    'content': content
                })
            except Exception as e:
                print(f"Could not read file {file_path}: {e}")

    return collected_data

def save_to_file(data, output_file):
    try:
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=4)
        print(f"Data saved to {output_file}")
    except Exception as e:
        print(f"Could not save data to {output_file}: {e}")

if __name__ == "__main__":
    cwd = os.getcwd()
    files_data = collect_files(cwd)
    
    output_file = 'collected_files.json'
    save_to_file(files_data, output_file)
