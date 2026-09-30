import os
import sys
import json
from datetime import datetime
import re
import requests
from openai import OpenAI
# ===================== Fixed Configuration (keep original parameters) =====================
LLM_API_KEY = ""
LLM_MODEL = "deepseek-v4-pro"
LLM_API_URL = "https://api.deepseek.com"
UTILITIES_DIR = ""
# Path for JSON cache file
CACHE_JSON_FILE = "/home/User/WorkSpace/LLMInliner/cache_llm_inline_polybench_v2_ds_pro_temp.json"
SUMMARYIZE_DIR = "/home/User/WorkSpace/LLMInliner/polybench_ds_pro"
# =========================================================================
client = OpenAI(
    api_key=LLM_API_KEY,
    base_url=LLM_API_URL,
    timeout=30.0
)
def load_json_cache():
    """Load JSON cache, return dict in format {(contract_file, call_relation): output}"""
    cache_dict = {}
    if not os.path.exists(CACHE_JSON_FILE):
        return cache_dict
    try:
        with open(CACHE_JSON_FILE, "r", encoding="utf-8") as f:
            records = json.load(f)
        for item in records:
            c_file = item["contract_c_file"]
            call_rel = item["call_relation_line"]
            output = item["origin_output"]
            # Core modification: keep only valid judgment results, filter invalid/redundant outputs
            valid_results = {"Inline", "NoInline","nodecision"}
            if output in valid_results:
                cache_key = (c_file, call_rel)
                cache_dict[cache_key] = output
    except Exception as e:
        print(f"Failed to load JSON cache: {e}")
    return cache_dict
def save_json_cache(cache_dict):
    """Write in‑memory cache dictionary to JSON file completely"""
    records = []
    for (c_file, call_rel), output in cache_dict.items():
        records.append({
            "contract_c_file": c_file,
            "call_relation_line": call_rel,
            "origin_output": output
        })
    try:
        with open(CACHE_JSON_FILE, "w", encoding="utf-8") as f:
            json.dump(records, f, ensure_ascii=False, indent=2)
    except Exception as e:
        print(f"Failed to write JSON cache: {e}")
def clean_source_comments(code):
    """
    Clean source code:
    1. Remove all /* $Header: ... */ CVS version comments
    2. Remove block copyright/license comment block at file header
    3. Preserve business code, functions, macros and variables
    """
    # 1. Remove any CVS Header comment block
    code = re.sub(r'/\*\s*\$Header:.*?\$.*?\*/\s*', '', code, flags=re.S)
    # 2. Remove block copyright comment at the very beginning of file (starting with first /*)
    # Only delete leading comment block, keep inline comments inside source code
    code = re.sub(r'^\s*/\*.*?\*/\s*', '', code, flags=re.S)
    return code.strip() + "\n"
def read_file_content(file_path):
    """Read content of single file (handle encoding exceptions)"""
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            return clean_source_comments(f.read())
    except UnicodeDecodeError:
        with open(file_path, "r", encoding="latin-1") as f:
            return clean_source_comments(f.read())
    except Exception as e:
        print(f"Failed to read file {file_path}: {e}")
        return ""
def collect_code_files(contract_c_file):
    """
    New path: SUMMARYIZE_DIR/bench_name
    Note: For large test cases, such as SPEC CPU 2017, only collect the code context of caller func and callee func.
          Use tree-sitter to collect.
    """
    code_content = {}
    if not os.path.exists(contract_c_file):
        print(f"Main file {contract_c_file} does not exist!")
        return None
    bench_dir_name = os.path.basename(os.path.dirname(contract_c_file))
    # Concatenate target reading directory
    target_util_dir = os.path.join(SUMMARYIZE_DIR, bench_dir_name)
    UTILITIES_DIR = target_util_dir
    if not os.path.exists(UTILITIES_DIR):
        print(f"Target utility directory {UTILITIES_DIR} does not exist!")
        return None
    # Recursively read all .c/.summary files under target directory
    for root, _, files in os.walk(UTILITIES_DIR):
        for file in files:
            if file.endswith((".c", ".summary")):
                file_path = os.path.join(root, file)
                code_content[file_path] = read_file_content(file_path)
    total_content = ""
    for file_path, content in code_content.items():
        total_content += f"{content}\n\n"
    return total_content,UTILITIES_DIR
def call_LLM(system_prompt, prompt):
    """Call LLM API to get judgment result (OpenAI SDK v1.x)"""
    try:
        response = client.chat.completions.create(
            model=LLM_MODEL,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ],
            temperature=0.1,
            max_tokens=2000,
            timeout=30.0
        )
        content = response.choices[0].message.content
        if content is None:
            return ""
        return content.strip()
    except Exception as e:
        print(f"LLM API call failed: {e}")
        return ""
def main():
    # 1. Load full cache into memory
    cache = load_json_cache()
    # 2. Validate input arguments
    if len(sys.argv) < 2:
        print("Invalid arguments! Usage: python3 get_inline_functions.py contractName [extra_prompt]")
        sys.exit(1)
    contract_name = sys.argv[1]
    contract_c_file = f"{contract_name}"
    extra_prompt = sys.argv[2] if len(sys.argv) >= 3 else ""
    # Extract Call Relation
    call_relation_line = ""
    if extra_prompt:
        for line in extra_prompt.split('\n'):
            if 'Call Relation:' in line:
                call_relation_line = line.strip()
                break
    code_content,UTILITIES_DIR = collect_code_files(contract_c_file)
    caller_name=""
    callee_name=""
    llm_result = ""
    if not code_content:
        llm_result="nodecision"
        # sys.exit(1)
    caller_in_code = False
    callee_in_code = False
    # Use rust_demangler to parse mangled function names
    if call_relation_line:
        call_relation_line=call_relation_line.replace("Call Relation: ","")
        # Extract caller and callee (format: Call Relation: caller -> callee)
        parts = call_relation_line.split(' -> ')
        if len(parts) == 2:
            # Parse real function names
            try:
                caller_name = parts[0].strip()
                caller_in_code = (caller_name in code_content)
                # If caller not present in code_content, directly output nodecision and exit
                if not caller_in_code and caller_name != "__main_argc_argv":
                    llm_result="nodecision"
                else:
                    caller_in_code=True
                # Check whether callee exists in code_content
                callee_name = parts[1].strip()
                callee_in_code = (callee_name in code_content)
                # If callee not present in code_content, output nodecision and exit
                if not callee_in_code:
                    llm_result="nodecision"
                else:
                    callee_in_code=True
            except Exception as e:
                llm_result="nodecision"
                sys.exit(1)
    cache_key = (contract_c_file, call_relation_line)
    cached=False
    # Cache hit, return cached result directly
    if llm_result=="" and cache_key in cache:
        llm_result = cache[cache_key]
        cached=True
    if llm_result=="" and cached==False:
        system_prompt = """
You are an engineer proficient in C‑language optimization and familiar with LLVM function inlining optimization rules.
You will judge whether a function call site is suitable for inlining according to function information obtained from LLVM.
LLVM will perform concrete inlining actions based on your decision to compile C source code into binary.

Analyze the given code (including main contract file and utility helper functions). Based on LLVM optimization best‑practices and provided reference information, judge whether this call site should be inlined.
Note that no macro definitions are provided in your compilation configuration.

Requirements:
1. The provided source contains per‑file code summaries in the format below:
    === File: XXX ===
    [File-level role]: Functional role and project positioning of this file
    [Global Variable Description]: All macros and global variables with their semantics; write "none" if there are none
    [Function Detail]: List every function inside this file item‑by‑item:
        [Function Name]: function_name
            [Usage]: What this function does and key logic (e.g. which other functions it invokes)
            [isCore]: Yes / No
            [shouldInline]: Yes / No (this is only a reference hint; final decision must combine compiler‑provided metadata)

2. Compiler provides call‑site metadata for your reference:
    a. Call Relation: <Caller Name> -> <Callee Name>
    b. Callee BB Count
    c. Callee Instruction Count
    d. Callee Total Uses
    e. Callers Total Uses
    f. HasCalls (whether callee invokes other functions)
    g. HasLoops (whether callee contains loop constructs)
    h. Callee IR
    i. Caller IR

3. Synthesize all above evidence: both source‑level summaries and compiler IR metadata. Not every call should be inlined; keep inlining conservative.
4. Critical constraint: output only one single verdict word: either Inline or NoInline. Do not output both.
5. Do NOT output reasoning steps. Output nothing except the single final verdict token required in rule‑4.
"""
        prompt = f"""
Source code content:
{code_content}
Additional compiler metadata for reference:
{extra_prompt}
"""
        llm_result = call_LLM(system_prompt, prompt)
        # llm_result="nodecision"
        if not llm_result:
            llm_result="nodecision"
            # sys.exit(1)
        # Update in‑memory cache and persist to JSON file
        cache[cache_key] = llm_result
        save_json_cache(cache)
    # Print final judgment result
    print(llm_result.strip())
    # Write raw log (preserve original logic)
    current_time = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    log_content = f"""
=== Execution Record ===
Time: {current_time}
contract_c_file: {contract_c_file}
UTILITIES_DIR: {UTILITIES_DIR}
call_relation_line: {call_relation_line}
code_content size: {len(code_content)} chars
extra_prompt size: {len(extra_prompt)} chars
origin_output: {llm_result}
caller_name: {caller_name}
callee_name: {callee_name}
caller_in_code: {caller_in_code}
callee_in_code: {callee_in_code}
if_cache_hit: {cached}
"""
    try:
        with open("/home/User/WorkSpace/LLMInliner/last_prompt_llvm_llm_polybench_v2_ds_pro_temp.txt", "a", encoding="utf-8") as f:
            f.write(log_content)
    except Exception as e:
        print(f"Failed writing last_prompt.txt: {e}")
    return 0
if __name__ == "__main__":
    main()
