import os
import re

PROJ_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
CARGO_TOML_PATH = os.path.join(PROJ_DIR, "Cargo.toml")
CORE_CARGO_TOML_PATH = os.path.join(
    PROJ_DIR, "crates", "git-flow-tui-core", "Cargo.toml"
)
TUI_CARGO_TOML_PATH = os.path.join(PROJ_DIR, "crates", "git-flow-tui", "Cargo.toml")
CLI_CARGO_TOML_PATH = os.path.join(PROJ_DIR, "crates", "git-flow", "Cargo.toml")

if __name__ == "__main__":
    while True:
        value = input("Enter new version (x.y.z): ")

        parts = value.split(".")

        if len(parts) == 3 and all(p.isdigit() for p in parts):
            break

        print("Invalid format (ej: 1.2.3)")

    version_re = re.compile(r'^(\s*version\s*=\s*")[^"]*(".*)$')
    for file in [
        CARGO_TOML_PATH,
        CORE_CARGO_TOML_PATH,
        CLI_CARGO_TOML_PATH,
        TUI_CARGO_TOML_PATH,
    ]:
        with open(file, "r", encoding="utf-8") as f:
            lines = f.readlines()

        for i, line in enumerate(lines):
            if version_re.match(line):
                lines[i] = version_re.sub(rf"\g<1>{value}\2", line)
                break

        with open(file, "w", encoding="utf-8") as f:
            f.writelines(lines)
