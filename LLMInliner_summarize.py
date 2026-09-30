import os
import sys
import re
import requests
from datetime import datetime
from openai import OpenAI
# ===================== Fixed Configuration (keep original parameters) =====================
LLM_API_KEY = ""
LLM_MODEL = "deepseek-v4-pro"
LLM_API_URL = "https://api.deepseek.com"
BASE_SAVE_DIR = "/home/User/WorkSpace/LLMInliner/polybench_ds_pro"
UTILITIES_DIR = "/home/User/WorkSpace/PolyBenchC-4.2.1/utilities"
# =========================================================================
client = OpenAI(
    api_key=LLM_API_KEY,
    base_url=LLM_API_URL,
    timeout=30.0
)

def clean_source_comments(code):
    """
    Clean source code:
    1. Remove all /* $Header: ... */ CVS version comments
    2. Remove block copyright/license comment block at file header
    3. Preserve business logic code, functions, macros and variables
    """
    # 1. Remove any CVS Header comment lines
    code = re.sub(r'/\*\s*\$Header:.*?\$.*?\*/\s*', '', code, flags=re.S)
    # 2. Remove leading copyright comment block starting with the first /*
    # Only delete the top‑most comment block, keep inline comments in source body
    code = re.sub(r'^\s*/\*.*?\*/\s*', '', code, flags=re.S)
    return code.strip() + "\n"

def read_file_content(file_path):
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            raw = f.read()
    except UnicodeDecodeError:
        with open(file_path, "r", encoding="latin-1") as f:
            raw = f.read()
    except Exception as e:
        print(f"Failed to read {file_path}: {e}")
        return ""
    # Strip source comments
    return clean_source_comments(raw)

def get_all_c_h_files(folder):
    file_list = []
    # for root, _, files in os.walk(UTILITIES_DIR):
    #     for fname in files:
    #         if fname.endswith(".c") or fname.endswith(".cpp"):
    #             file_list.append(os.path.abspath(os.path.join(root, fname)))
    for root, _, files in os.walk(folder):
        for fname in files:
            if fname.endswith(".c") or fname.endswith(".cpp"):
                file_list.append(os.path.abspath(os.path.join(root, fname)))
    return file_list

def concat_all_files(files):
    total = ""
    file_origin_map = {}
    for fpath in files:
        content = read_file_content(fpath)
        pure_name = os.path.basename(fpath)
        block = f"=== File: {pure_name} ===\n{content}\n\n"
        total += block
        file_origin_map[fpath] = {"name": pure_name, "content": content}
    return total, file_origin_map

def call_llm_batch_analysis(all_code_text):
    system_prompt = """
You are a senior C source‑code analysis engineer. Your job is to parse C source files, extract file‑level purpose, macro/global variable semantics, and detailed function descriptions for later LLVM function‑inlining optimization when compiling C to WASM.

You will receive a batch of C source files. Each file begins with a header marker === File: XXX ===.
Note that no custom macro definitions are provided; compilation uses default settings only.

Task:
Output summaries for every input file. **Do NOT output original source code**. Follow the exact format below. Separate summaries of different files with blank lines.

=== File: XXX ===
[File-level role]: Functional role and project positioning of this file
    [Global Variable Description]: All macros and global variables with their semantics; write "none" if there are none
    [Function Detail]: List every function inside this file item‑by‑item:
        [Function Name]: function_name
            [Usage]: What this function does and key logic (e.g. which other functions it invokes)
            [isCore]: Yes / No
            [shouldInline]: Yes / No (this is only a reference hint; final decision must combine compiler‑provided metadata)

Requirements:
1. Cover all input files without omission
2. Each file summary must start with === File: XXX ===
3. Keep output concise and professional; never echo source code
4. If the file contains main(), explicitly mark it as the program entry point inside [File Purpose]
5. For each function entry, strictly include: function name, purpose & logic, whether it is a core function, and whether it is a candidate for inlining (Yes / No).
"""
    user_prompt = f"Below are the batch source‑code files:\n{all_code_text}"
    try:
        # Reuse the globally initialized OpenAI client instance
        response = client.chat.completions.create(
            model=LLM_MODEL,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            temperature=0.1,
            max_tokens=100000,
            timeout=120.0
        )
        content = response.choices[0].message.content
        if content is None:
            return "Batch LLM analysis failed: returned content is None"
        return content.strip()
    except Exception as e:
        return f"Batch LLM analysis failed: {str(e)}"

def parse_llm_result_to_dict(llm_raw):
    summary_dict = {}
    pattern = re.compile(r"=== File: (.*?) ===", re.MULTILINE)
    splits = pattern.split(llm_raw)
    for i in range(1, len(splits), 2):
        file_name = splits[i].strip()
        summary = splits[i + 1].strip()
        summary_dict[file_name] = summary
    return summary_dict

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 LLMInliner_summarize.py source_directory_path")
        sys.exit(1)
    target_dir = sys.argv[1]
    files = get_all_c_h_files(target_dir)
    if not files:
        print("No .c/.cpp files found")
        return
    case_name = os.path.basename(target_dir.rstrip("/"))
    final_save_dir = os.path.join(BASE_SAVE_DIR, case_name)
    os.makedirs(final_save_dir, exist_ok=True)
    print(f"Loaded {len(files)} source files, case directory: {case_name}")
    all_code, file_origin_map = concat_all_files(files)
    print(f"all_code size: {len(all_code)} characters")
    # Uncomment to enable real LLM invocation
    llm_raw_result = call_llm_batch_analysis(all_code)
    # llm_raw_result=""
    summary_map = parse_llm_result_to_dict(llm_raw_result)
    print(f"llm_raw_result size: {len(llm_raw_result)} characters")
    print(f"llm_raw_result: {llm_raw_result} characters")
    for fpath in files:
        info = file_origin_map[fpath]
        pure_name = info["name"]
        src_content = info["content"]
        sum_text = summary_map.get(pure_name, "[No corresponding summary]")
        out_content = (
            f"=== File: {pure_name} ===\n"
            f"# File Summary:\n{sum_text}\n\n"
            f"{src_content}"
        )
        save_path = os.path.join(final_save_dir, f"{pure_name}.summary")
        with open(save_path, "w", encoding="utf-8") as f:
            f.write(out_content)
    print(f"\n✅ All tasks finished! Output directory: {final_save_dir}")

if __name__ == "__main__":
    main()
