import os
from pathlib import Path
from dataclasses import dataclass
from tempfile import TemporaryDirectory
import subprocess
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("--program")
args = parser.parse_args()

programs = Path("programs")
compiler_bin = "../target/release/c-compiler"

@dataclass
class Program:
    source: str
    output: str

def parse_program(inp: Path) -> Program:
    c_source = ""
    expected_output = ""

    reading_output = False
    reading_source = False

    with open(inp) as f:
        for line in f:
            stripped = line.strip()
            if stripped == "// === Source ===":
                reading_source = True
                continue
            if stripped == "// === End Source ===":
                reading_source = False
                continue
            if stripped == "// === Output ===":
                reading_output = True
                continue
            if stripped == "// === End Output ===":
                reading_output = False
                continue
            if reading_output:
                expected_output += line.lstrip("// ")
            if reading_source:
                c_source += line
    return Program(c_source, expected_output)

def compile(source_file, asm_file) -> int:
    result = subprocess.run(
        [compiler_bin, "-i", source_file, "-o", asm_file],
        capture_output=True,
        text=True
    )
    return result.returncode

def compile_asm(asm_file, exe_file) -> int:
    result = subprocess.run(
        ["clang", asm_file, "-o", exe_file],
        capture_output=True,
        text=True
    )
    return result.returncode

def compare_output(exe_file, expected_output):
    try:
        result = subprocess.run(
            [exe_file],
            capture_output=True,
            text=True,
            timeout=10  # Prevent infinite loops
        )
        
        actual_output = result.stdout
        
        # Normalize newlines and compare
        if actual_output.replace('\r\n', '\n') == expected_output.replace('\r\n', '\n'):
            return True, "Output matches", actual_output
        else:
            return False, f"Output mismatch", actual_output
            
    except subprocess.TimeoutExpired:
        return False, "Execution timed out", ""
    except FileNotFoundError:
        return False, f"Executable not found: {exe_file}", ""
    except Exception as e:
        return False, f"Execution failed: {e}", ""

failed = 0
progs = os.listdir(programs)
progs.sort(key= lambda x: int(x.split('_')[0]))
for p in progs:
    if args.program is not None and args.program != p:
        continue
    program = programs / p
    program = parse_program(program)
    error = ""

    with TemporaryDirectory() as td:
        source_file = Path(td) / "source.c"
        out_file = Path(td) / "out.txt"
        asm_file = Path(td) / "out.asm"
        exe_file = Path(td) / "out.exe"
        with open(source_file, "w") as f: 
            f.write(program.source)
        with open(out_file, "w") as f: 
            f.write(program.source)

        if compile(source_file, asm_file) != 0:
            print(f"❌ Program {p} failed to compile c")
            failed += 1
            continue

        if compile_asm(asm_file, exe_file) != 0:
            print(f"❌ Program {p} failed to compile assembly")
            failed += 1
            continue

        ok, err, output = compare_output(exe_file, program.output)
        if not ok:
            print(f"❌ Program {p} failed to output, because", err)
            print("Expected output")
            print(program.output)
            print("Output")
            print(output)
            failed += 1
            continue
        print(f"✅ Test succeded: {p}")

if failed > 0:
    print("")