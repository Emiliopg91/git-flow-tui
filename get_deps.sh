SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

cargo build --release

OLD_LANG=$LANG
export LANG=C
pkgs=$(ldd "$SCRIPT_DIR/target/release/git-flow-tui" | awk '/=>/ { print $(NF-1) }' | xargs pacman -Qo | rev | cut -d' ' -f2 | rev | sort -u)

#for pkg in $pkgs; do
#    desc=$(pacman -Qi "$pkg" | grep "Description" | cut -d':' -f 2)
#    echo -e "$pkg\n  $desc"
#done

echo $pkgs

export LANG=$OLD_LANG