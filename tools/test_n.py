import sys
import subprocess

def main():
    failed = []
    n = int(sys.argv[1])
    test_args = sys.argv[2:]
    for i in range(n):
        print(i +1, "on", n, end=" ")
        process = subprocess.run(
            ["cargo", "test", *test_args],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        if process.returncode == 0:
            print("ok")
        else:
            print("!!! FAILED")
            failed.append(process.stdout.decode())

    if not failed:
        return

    for fail in failed:
        print(fail)
        print("-" * 80)
        print()


if __name__ == "__main__":
    main()
