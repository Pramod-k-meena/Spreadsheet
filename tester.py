import subprocess
import filecmp

# Test files and dimensions
input_files = [
    # "textfiles/test1.txt","textfiles/test2.txt", "textfiles/test3.txt", "textfiles/test4.txt", "textfiles/test5.txt", "textfiles/test6.txt",
    # "textfiles/test7.txt", "textfiles/test8.txt", "textfiles/test9.txt", 
    "textfiles/test10.txt", "textfiles/test11.txt", "textfiles/test12.txt",
]
output_files = [
    # "textfiles/output1.txt","textfiles/output2.txt", "textfiles/output3.txt", "textfiles/output4.txt", "textfiles/output5.txt", "textfiles/output6.txt",
    # "textfiles/output7.txt", "textfiles/output8.txt", "textfiles/output9.txt",
    "textfiles/output10.txt", "textfiles/output11.txt", "textfiles/output12.txt",
]
expected_files = [
    # "textfiles/expected_output1.txt","textfiles/expected_output2.txt", "textfiles/expected_output3.txt", "textfiles/expected_output4.txt", "textfiles/expected_output5.txt", "textfiles/expected_output6.txt",
    # "textfiles/expected_output7.txt", "textfiles/expected_output8.txt", "textfiles/expected_output9.txt",
    "textfiles/expected_output10.txt",  "textfiles/expected_output11.txt", "textfiles/expected_output12.txt",
]
rows = 999
cols = 18278

def run_test(input_file, output_file, expected_file, row, col):
    with open(input_file, "r") as infile, open(output_file, "w") as outfile:
        result = subprocess.run(
            ["./target/release/spreadsheet", str(row), str(col)],
            stdin=infile,
            stdout=outfile
        )
        if result.returncode != 0:
            print(f"ERROR: Spreadsheet binary failed on {input_file}")
            return False

    return filecmp.cmp(output_file, expected_file, shallow=False)

def main():
    all_passed = True

    for i in range(len(input_files)):
        if run_test(input_files[i], output_files[i], expected_files[i], rows, cols):
            # print(f"{output_files[i]} matches expected output!")
            pass
        else:
            print(f"ERROR: {output_files[i]} does not match expected output!")
            all_passed = False

    if all_passed:
        print("Test successful!")
    else:
        print("Some tests failed. Check output files.")

if __name__ == "__main__":
    main()
