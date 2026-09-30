# LLM-Inliner: A Large Language Model Guided Function Inlining Framework

This repository contains the implementation and experimental artifacts for our paper **LLM-Inliner: A Large Language Model Guided Function Inlining Framework**.
LLM-Inliner leverages large language models to capture source-level code context and make semantically aware function inlining decisions for LLVM-based compilers.

## Datasets

We use the following benchmark suites for evaluation in our paper:

1. **cBench V1.1**
2. **PolyBenchC-4.2.1**
3. **ChainMaker**
4. **SPEC CPU 2017**: This benchmark is closed-source and must be downloaded from the official website. https://www.spec.org/cpu2017/

## Code Overview

- `LLMInliner_summarize.py`: Extracts code context by invoking large language models for the LLM-Inliner framework.
- `LLMInliner_llvm_call.py`: Handles LLM-based function inlining decision logic.
- `InlineCost.cpp`: Modified LLVM source file to integrate LLM-Inliner into the LLVM inlining pass.
It extracts instruction-level features, invokes `LLMInliner_llvm_call.py` for inlining decisions, and performs the final inlining transformation.
> 
> This file can directly replace the original `InlineCost.cpp` from LLVM 22.1.6:
> [https://github.com/llvm/llvm-project/blob/llvmorg-22.1.6/llvm/lib/Analysis/InlineCost.cpp](https://github.com/llvm/llvm-project/blob/llvmorg-22.1.6/llvm/lib/Analysis/InlineCost.cpp)

## Experimental Results

The `results/` directory stores raw and processed experimental data, including execution time measurements for LLM-Inliner and all baseline methods used in our evaluation.