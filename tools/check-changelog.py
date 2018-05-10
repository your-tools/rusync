import sys


def main():
    new_version = sys.argv[1]
    with open("Changelog.md") as stream:
        contents = stream.read()
    if not new_version in contents:
        sys.exit("new_version: %s not found in Changelog" % new_version)


if __name__ == "__main__":
    main()
