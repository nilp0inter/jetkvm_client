#!/usr/bin/env python3
import os
import sys
import stat
import argparse
import subprocess

def read_jetkvm_control_files(src_dir: str) -> dict:
    """
    Recursively reads all files in src_dir and returns a dictionary mapping
    relative paths (including the src_dir as prefix) to their contents.
    """
    file_dict = {}
    for root, dirs, files in os.walk(src_dir):
        # Exclude .git directory
        dirs[:] = [d for d in dirs if d != ".git"]
        for file in files:
            full_path = os.path.join(root, file)
            # Create a relative path that includes the top-level directory name
            rel_path = os.path.relpath(full_path, os.path.dirname(src_dir))
            try:
                with open(full_path, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                print(f"Warning: Could not read {full_path}: {e}")
                content = ""
            file_dict[rel_path] = content
    return file_dict

def generate_dynamic_script(file_dict: dict) -> str:
    """
    Generates the content of the dynamic script as a string.
    The generated script contains a FILES dictionary with the mapping and code
    to create the directory structure and write out the files.
    """
    # Use repr() to embed the dictionary as a valid Python literal.
    files_literal = repr(file_dict)
    
    # Build the dynamic script content.
    script = f'''#!/usr/bin/env python3
"""
This script dynamically recreates the "jetkvm_control" directory tree
from embedded static file contents, and then copies its own contents to
the clipboard.
Usage:
    python create_jetkvm_control_dynamic.py
"""

import os
import sys
import shutil
import subprocess

# Embedded file mapping: keys are relative paths, values are file contents.
FILES = {files_literal}

def create_files():
    for rel_path, content in FILES.items():
        # Create the directory if needed.
        dir_path = os.path.dirname(rel_path)
        if dir_path and not os.path.exists(dir_path):
            os.makedirs(dir_path, exist_ok=True)
            print(f"Created directory: {{dir_path}}")
        # Write the file.
        with open(rel_path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Created file: {{rel_path}}")

def copy_self_to_clipboard():
    try:
        with open(__file__, "r", encoding="utf-8") as f:
            content = f.read()
        subprocess.run(["pbcopy"], input=content, encoding="utf-8", check=True)
        print("The dynamic script's contents have been copied to the clipboard.")
    except Exception as e:
        print(f"Failed to copy self to clipboard: {{e}}")

def main():
    create_files()

if __name__ == "__main__":
    main()
    copy_self_to_clipboard()
'''
    return script

def main():
    parser = argparse.ArgumentParser(description="Generate a dynamic script for jetkvm_control.")
    parser.add_argument("--src-only", action="store_true", help="Only process the src folder inside jetkvm_control.")
    args = parser.parse_args()

    # Determine the directory to process
    base_dir = "../../jetkvm_control"
    src_dir = os.path.join(base_dir, "src") if args.src_only else base_dir

    if not os.path.isdir(src_dir):
        print(f"Error: Source directory '{src_dir}' does not exist.")
        sys.exit(1)
    
    # Read files from the selected directory
    file_dict = read_jetkvm_control_files(src_dir)
    
    # Generate the dynamic script content
    dynamic_script = generate_dynamic_script(file_dict)
    
    # Write the dynamic script to a file
    output_filename = "create_jetkvm_control_.py"
    with open(output_filename, "w", encoding="utf-8") as f:
        f.write(dynamic_script)
    
    # Make it executable
    st = os.stat(output_filename)
    os.chmod(output_filename, st.st_mode | stat.S_IEXEC)
    
    print(f"Generated '{output_filename}' successfully.")
    
    # Copy the generated dynamic script's content to the clipboard
    try:
        subprocess.run(["pbcopy"], input=dynamic_script, encoding="utf-8", check=True)
        print("The generated dynamic script has been copied to the clipboard.")
    except Exception as e:
        print(f"Failed to copy the generated script to the clipboard: {e}")

if __name__ == "__main__":
    main()

